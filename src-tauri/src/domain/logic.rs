use super::{
    errors::ValidationError,
    types::{
        default_configured_days, BreakMinutes, ConfiguredDay, DayEntry, ThemePreference,
        WeekSheet, WeekSummary, WorkInterval,
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

pub fn default_interval() -> WorkInterval {
    WorkInterval {
        start: super::types::TimeOfDay(8 * 60),
        end: super::types::TimeOfDay(18 * 60),
    }
}

pub fn default_break() -> BreakMinutes {
    BreakMinutes(60)
}

pub fn default_theme() -> ThemePreference {
    ThemePreference::Dark
}

pub fn default_entries(configured_days: &[ConfiguredDay]) -> Vec<DayEntry> {
    configured_days
        .iter()
        .cloned()
        .map(|day| DayEntry {
            day_id: day.day_id,
            label: day.label,
            interval: if day.enabled { Some(default_interval()) } else { None },
            break_minutes: default_break(),
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

pub fn default_settings() -> super::types::AppSettings {
    super::types::AppSettings {
        overtime_threshold: super::types::OvertimeThresholdMinutes(35 * 60),
        theme: default_theme(),
        default_work_interval: default_interval(),
        default_break_minutes: default_break(),
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
        WeekId, WeekSheet, WeekStartDate, WorkInterval,
    };

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
