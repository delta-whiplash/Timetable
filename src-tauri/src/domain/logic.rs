use super::{
    errors::ValidationError,
    types::{
        default_configured_days, BreakMinutes, DayEntry, DayType, ThemePreference, TravelDeductionMinutes,
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
            has_departure_deduction: false,
            has_return_deduction: false,
            day_type: if day.enabled {
                DayType::Work
            } else {
                DayType::Disabled
            },
        })
        .collect()
}

pub fn validate_day(entry: &DayEntry) -> Result<(), ValidationError> {
    if entry.label.0.trim().is_empty() {
        return Err(ValidationError::EmptyLabel {
            day_id: entry.day_id.0,
        });
    }

    // Vacation, public holidays and disabled days don't need time validation
    if !entry.enabled || matches!(entry.day_type, DayType::Vacation | DayType::PublicHoliday) {
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

pub fn calculate_day_minutes(
    entry: &DayEntry,
    travel_deduction_minutes: TravelDeductionMinutes,
    vacation_day_hours: u16,
) -> Result<u16, ValidationError> {
    validate_day(entry)?;

    // Vacation days count as configured vacation hours (default 7.8h)
    if entry.day_type == DayType::Vacation {
        return Ok(vacation_day_hours);
    }

    // Public holidays don't count toward totals
    if entry.day_type == DayType::PublicHoliday {
        return Ok(0);
    }

    if !entry.enabled {
        return Ok(0);
    }

    let Some(interval) = entry.interval else {
        return Ok(0);
    };

    let mut net = i32::from(interval.end.0 - interval.start.0 - entry.break_minutes.0);

    // Déductions déplacement (montant configurable par case cochée)
    if entry.has_departure_deduction {
        net -= i32::from(travel_deduction_minutes.0);
    }
    if entry.has_return_deduction {
        net -= i32::from(travel_deduction_minutes.0);
    }

    // Saturer à 0 (pas de temps négatif)
    Ok(net.max(0) as u16)
}

pub fn summarize_week(sheet: &WeekSheet) -> Result<WeekSummary, ValidationError> {
    let mut total = 0_u16;
    let mut worked_days = 0_u8;

    for entry in &sheet.entries {
        let minutes = calculate_day_minutes(entry, sheet.travel_deduction_minutes, sheet.vacation_day_hours)?;
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
        enable_travel_deduction: true,
        travel_deduction_minutes: TravelDeductionMinutes::default(),
        vacation_day_hours: 468, // 7.8h par défaut (39h / 5 jours)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::domain::types::{
        AppSettings, BreakMinutes, DayEntry, DayId, DayLabel, OvertimeThresholdMinutes, TimeOfDay,
        ThemePreference, TravelDeductionMinutes, WeekId, WeekSheet, WeekStartDate, WorkInterval,
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
            enable_travel_deduction: true,
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            vacation_day_hours: 468,
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
            has_departure_deduction: false,
            has_return_deduction: false,
            day_type: DayType::Work,
        }
    }

    #[test]
    fn computes_week_summary() {
        let sheet = WeekSheet {
            week_id: Some(WeekId::new()),
            week_start: WeekStartDate::today(),
            entries: vec![
                build_day(0, 480, 1080, 60),
                build_day(1, 480, 1020, 30),
            ],
            overtime_threshold: OvertimeThresholdMinutes(35 * 60),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            vacation_day_hours: 468,
            updated_at: String::new(),
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
            let total = calculate_day_minutes(&entry, TravelDeductionMinutes::default(), 468).expect("valid day");
            prop_assert!(total <= 1439);
        }
    }

    #[test]
    fn defaults_provide_working_configuration() {
        let settings: AppSettings = default_settings();
        assert_eq!(settings.overtime_threshold.0, 2100);
        assert_eq!(settings.configured_days.len(), 7);
        assert!(settings.enable_travel_deduction);
        assert_eq!(settings.travel_deduction_minutes.0, 30);
    }

    #[test]
    fn departure_deduction_reduces_time_by_30min() {
        let mut entry = build_day(0, 8 * 60, 18 * 60, 60); // 08:00-18:00, 60min break
        let deduction = TravelDeductionMinutes::default(); // 30 min
        let vacation_hours = 468; // 7.8h default

        // Sans déduction: 600 - 60 = 540min
        assert_eq!(calculate_day_minutes(&entry, deduction, vacation_hours).unwrap(), 540);

        // Avec déduction départ: 540 - 30 = 510min
        entry.has_departure_deduction = true;
        assert_eq!(calculate_day_minutes(&entry, deduction, vacation_hours).unwrap(), 510);
    }

    #[test]
    fn return_deduction_reduces_time_by_30min() {
        let mut entry = build_day(0, 8 * 60, 18 * 60, 60);
        let deduction = TravelDeductionMinutes::default(); // 30 min
        let vacation_hours = 468; // 7.8h default

        // Avec déduction retour: 540 - 30 = 510min
        entry.has_return_deduction = true;
        assert_eq!(calculate_day_minutes(&entry, deduction, vacation_hours).unwrap(), 510);
    }

    #[test]
    fn both_deductions_reduce_time_by_60min() {
        let mut entry = build_day(0, 8 * 60, 18 * 60, 60);
        let deduction = TravelDeductionMinutes::default(); // 30 min
        let vacation_hours = 468; // 7.8h default

        // Avec les deux déductions: 540 - 60 = 480min (8h net)
        entry.has_departure_deduction = true;
        entry.has_return_deduction = true;
        assert_eq!(calculate_day_minutes(&entry, deduction, vacation_hours).unwrap(), 480);
    }

    #[test]
    fn deduction_saturates_at_zero() {
        let mut entry = build_day(0, 8 * 60, 9 * 60, 0); // 08:00-09:00, 0 break = 60min
        let deduction = TravelDeductionMinutes::default(); // 30 min
        let vacation_hours = 468; // 7.8h default

        entry.has_departure_deduction = true;
        entry.has_return_deduction = true;
        // 60 - 30 - 30 = 0 (saturé)
        assert_eq!(calculate_day_minutes(&entry, deduction, vacation_hours).unwrap(), 0);
    }

    #[test]
    fn configurable_deduction_amount() {
        let mut entry = build_day(0, 8 * 60, 18 * 60, 60); // 08:00-18:00, 60min break = 540min net
        let vacation_hours = 468; // 7.8h default

        // Avec 20min de déduction
        let deduction_20 = TravelDeductionMinutes::new(20).unwrap();
        entry.has_departure_deduction = true;
        assert_eq!(calculate_day_minutes(&entry, deduction_20, vacation_hours).unwrap(), 520); // 540 - 20 = 520

        // Avec 45min de déduction
        let deduction_45 = TravelDeductionMinutes::new(45).unwrap();
        entry.has_return_deduction = true;
        assert_eq!(calculate_day_minutes(&entry, deduction_45, vacation_hours).unwrap(), 450); // 540 - 45 - 45 = 450
    }

    #[test]
    fn vacation_day_counts_as_configured_hours() {
        let mut entry = build_day(0, 8 * 60, 18 * 60, 60);
        entry.day_type = DayType::Vacation;

        let deduction = TravelDeductionMinutes::default();
        // Vacation day should return configured hours (468 min = 7.8h by default)
        assert_eq!(calculate_day_minutes(&entry, deduction, 468).unwrap(), 468);
        // With custom 8h (480 min)
        assert_eq!(calculate_day_minutes(&entry, deduction, 480).unwrap(), 480);
    }

    #[test]
    fn vacation_days_count_in_week_summary() {
        let sheet = WeekSheet {
            week_id: Some(WeekId::new()),
            week_start: WeekStartDate::today(),
            entries: vec![
                build_day(0, 480, 1080, 60), // Work day: 540 min
                {
                    let mut day = build_day(1, 480, 1080, 60); // Would be 540 min
                    day.day_type = DayType::Vacation; // But counts as 468 min (7.8h)
                    day
                },
            ],
            overtime_threshold: OvertimeThresholdMinutes(35 * 60),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            vacation_day_hours: 468, // 7.8h default
            updated_at: String::new(),
        };

        let summary = summarize_week(&sheet).expect("summary should be valid");

        // Work day (540) + Vacation day (468) = 1008 min
        assert_eq!(summary.total_minutes, 1008);
        assert_eq!(summary.worked_days, 2);
    }

    #[test]
    fn vacation_day_does_not_require_time_validation() {
        let mut entry = build_day(0, 8 * 60, 18 * 60, 60);
        entry.day_type = DayType::Vacation;
        entry.interval = None; // No time set

        // Should validate successfully without time
        assert!(validate_day(&entry).is_ok());
    }
}
