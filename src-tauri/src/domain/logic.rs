use super::{
    errors::ValidationError,
    types::{
        default_configured_days, BreakMinutes, DayEntry, ThemePreference, WeekSheet,
        WeekSummary, WorkInterval,
    },
};

pub fn minutes_to_label(minutes: u16) -> String {
    format!("{}h{:02}", minutes / 60, minutes % 60)
}

pub fn minutes_to_human_label(minutes: u16) -> String {
    if minutes < 60 {
        format!("{} min", minutes)
    } else {
        format!("{}h {}min", minutes / 60, minutes % 60)
    }
}

pub fn default_theme() -> ThemePreference {
    ThemePreference::Dark
}

pub fn default_entries(settings: &super::types::AppSettings) -> Vec<DayEntry> {
    settings
        .configured_days
        .iter()
        .cloned()
        .map(|day| DayEntry {
            day_id: day.day_id,
            label: day.label,
            interval: if day.enabled {
                Some(settings.default_work_interval)
            } else {
                None
            },
            break_minutes: settings.default_break_minutes,
            enabled: day.enabled,
        })
        .collect()
}

pub fn validate_day(entry: &DayEntry) -> Result<(), ValidationError> {
    if entry.label.0.trim().is_empty() {
        return Err(ValidationError::EmptyLabel {
            day_id: entry.day_id.0,
        });
    }

    if !entry.enabled {
        return Ok(());
    }

    let interval = entry
        .interval
        .ok_or(ValidationError::MissingTimeInput {
            day_id: entry.day_id.0,
        })?;

    if interval.end.0 <= interval.start.0 {
        return Err(ValidationError::InvalidTimeRange {
            day_id: entry.day_id.0,
        });
    }

    if entry.break_minutes.0 >= interval.end.0 - interval.start.0 {
        return Err(ValidationError::BreakExceedsDay {
            day_id: entry.day_id.0,
        });
    }

    Ok(())
}

pub fn calculate_day_minutes(entry: &DayEntry) -> Result<u16, ValidationError> {
    validate_day(entry)?;
    if !entry.enabled {
        return Ok(0);
    }

    let Some(interval) = entry.interval else {
        return Ok(0);
    };

    Ok(interval.end.0 - interval.start.0 - entry.break_minutes.0)
}

pub fn summarize_week(sheet: &WeekSheet) -> Result<WeekSummary, ValidationError> {
    let mut total = 0_u16;
    let mut worked_days = 0_u8;

    for entry in &sheet.entries {
        let minutes = calculate_day_minutes(entry)?;
        if minutes > 0 {
            total = total.saturating_add(minutes);
            worked_days = worked_days.saturating_add(1);
        }
    }

    Ok(WeekSummary {
        total_minutes: total,
        worked_days,
    })
}

pub fn signed_minutes_to_label(minutes: i32) -> String {
    let abs = minutes.unsigned_abs();
    let label = format!("{}h{:02}", abs / 60, abs % 60);
    if minutes > 0 {
        format!("+{label}")
    } else if minutes < 0 {
        format!("-{label}")
    } else {
        label
    }
}

/// Pourcentage de l'objectif atteint (saturé, jamais de division par zéro).
pub fn threshold_percentage(total_minutes: u16, threshold_minutes: u16) -> u8 {
    if threshold_minutes == 0 {
        return 0;
    }
    (u32::from(total_minutes) * 100 / u32::from(threshold_minutes)).min(255) as u8
}

pub fn default_settings() -> super::types::AppSettings {
    super::types::AppSettings {
        overtime_threshold: super::types::OvertimeThresholdMinutes(35 * 60),
        theme: default_theme(),
        default_work_interval: WorkInterval {
            start: super::types::TimeOfDay(8 * 60),
            end: super::types::TimeOfDay(18 * 60),
        },
        default_break_minutes: BreakMinutes(60),
        configured_days: default_configured_days(),
        active_week_id: None,
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::domain::types::{
        AppSettings, BreakMinutes, DayEntry, DayId, DayLabel, OvertimeThresholdMinutes, TimeOfDay,
        ThemePreference, WeekId, WeekSheet, WeekStartDate, WorkInterval,
    };

    #[test]
    fn default_entries_honorent_les_parametres_utilisateur() {
        // Un utilisateur qui configure 07:00-15:00 / 45 min de pause
        // doit voir SES valeurs dans les nouvelles semaines, pas 08:00-18:00 / 60.
        let settings = AppSettings {
            overtime_threshold: OvertimeThresholdMinutes(35 * 60),
            theme: ThemePreference::Dark,
            default_work_interval: WorkInterval {
                start: TimeOfDay(7 * 60),
                end: TimeOfDay(15 * 60),
            },
            default_break_minutes: BreakMinutes(45),
            configured_days: default_configured_days(),
            active_week_id: None,
        };

        let entries = default_entries(&settings);

        let lundi = &entries[0];
        assert_eq!(lundi.interval.expect("jour actif").start.0, 7 * 60);
        assert_eq!(lundi.interval.expect("jour actif").end.0, 15 * 60);
        assert_eq!(lundi.break_minutes.0, 45);
        // Samedi désactivé : pas d'intervalle
        assert!(entries[5].interval.is_none());
    }

    fn build_day(day_id: u8, start: u16, end: u16, break_minutes: u16) -> DayEntry {
        DayEntry {
            day_id: DayId(day_id),
            label: DayLabel(format!("Jour {day_id}")),
            interval: Some(WorkInterval {
                start: TimeOfDay(start),
                end: TimeOfDay(end),
            }),
            break_minutes: BreakMinutes(break_minutes),
            enabled: true,
        }
    }

    #[test]
    fn computes_week_summary() {
        let sheet = WeekSheet {
            week_id: WeekId::new(),
            week_start: WeekStartDate::today(),
            entries: vec![
                build_day(0, 480, 1080, 60),
                build_day(1, 480, 1020, 30),
            ],
            overtime_threshold: OvertimeThresholdMinutes(35 * 60),
        };

        let summary = summarize_week(&sheet).expect("summary should be valid");

        assert_eq!(summary.total_minutes, 1050);
        assert_eq!(summary.worked_days, 2);
    }

    #[test]
    fn signed_minutes_to_label_zero() {
        assert_eq!(signed_minutes_to_label(0), "0h00");
    }

    #[test]
    fn signed_minutes_to_label_positive() {
        assert_eq!(signed_minutes_to_label(150), "+2h30");
        assert_eq!(signed_minutes_to_label(60), "+1h00");
        assert_eq!(signed_minutes_to_label(59), "+0h59");
        assert_eq!(signed_minutes_to_label(1), "+0h01");
    }

    #[test]
    fn signed_minutes_to_label_negative() {
        assert_eq!(signed_minutes_to_label(-150), "-2h30");
        assert_eq!(signed_minutes_to_label(-60), "-1h00");
        assert_eq!(signed_minutes_to_label(-59), "-0h59");
        assert_eq!(signed_minutes_to_label(-1), "-0h01");
    }

    proptest! {
        #[test]
        fn total_minutes_never_negative(start in 0u16..1200, duration in 1u16..360, break_minutes in 0u16..120) {
            let end = start.saturating_add(duration).min(1439);
            let entry = build_day(0, start, end.max(start + 1), break_minutes.min(duration.saturating_sub(1)));
            let total = calculate_day_minutes(&entry).expect("valid day");
            prop_assert!(total <= 1439);
        }
    }

    #[test]
    fn defaults_provide_working_configuration() {
        let settings: AppSettings = default_settings();
        assert_eq!(settings.overtime_threshold.0, 2100);
        assert_eq!(settings.configured_days.len(), 7);
    }
}
