use serde::{Deserialize, Serialize};

use crate::domain::{
    logic::{
        calculate_day_minutes, minutes_to_human_label, minutes_to_label, summarize_week,
        threshold_percentage,
    },
    types::{AppSettings, ConfiguredDay, DayType, TimeOfDay, WeekSheet},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWeekDayEntryInput {
    pub day_id: u8,
    pub label: String,
    pub enabled: bool,
    pub start: Option<String>,
    pub end: Option<String>,
    pub break_time: String,
    pub has_departure_deduction: bool,
    pub has_return_deduction: bool,
    pub day_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWeekInput {
    pub week_id: Option<String>,
    pub week_start: String,
    pub overtime_threshold_minutes: u16,
    pub travel_deduction_minutes: u16,
    pub entries: Vec<SaveWeekDayEntryInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekSelectorInput {
    pub week_start: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWeekInput {
    pub week_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguredDayView {
    pub day_id: u8,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSettingsInput {
    pub overtime_threshold_minutes: u16,
    pub default_start: String,
    pub default_end: String,
    pub default_break: String,
    pub configured_days: Vec<ConfiguredDayView>,
    pub enable_travel_deduction: bool,
    pub travel_deduction_minutes: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayEntryView {
    pub day_id: u8,
    pub label: String,
    pub enabled: bool,
    pub start: Option<String>,
    pub end: Option<String>,
    pub break_time: String,
    pub has_departure_deduction: bool,
    pub has_return_deduction: bool,
    pub total_minutes: u16,
    pub total_label: String,
    pub day_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekSummaryView {
    pub total_label: String,
    pub percentage: u8,
    pub cumulative_balance_minutes: i32,
    pub cumulative_balance_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekSheetView {
    pub week_id: Option<String>,
    pub week_start: String,
    pub entries: Vec<DayEntryView>,
    pub overtime_threshold_minutes: u16,
    pub travel_deduction_minutes: u16,
    pub summary: WeekSummaryView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekListItem {
    pub week_id: String,
    pub week_start: String,
    pub total_minutes: u16,
    pub total_label: String,
    pub worked_days: u8,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub overtime_threshold_minutes: u16,
    pub overtime_threshold_label: String,
    pub theme: String,
    pub default_start: String,
    pub default_end: String,
    pub default_break: String,
    pub configured_days: Vec<ConfiguredDayView>,
    pub active_week_id: Option<String>,
    pub enable_travel_deduction: bool,
    pub travel_deduction_minutes: u16,
    pub travel_deduction_label: String,
    pub vacation_day_hours: u16,
    pub vacation_day_hours_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    pub active_week: WeekSheetView,
}

/// Point de données pour les courbes analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekAnalyticsPoint {
    /// Date de début de la semaine (format ISO)
    pub week_start: String,
    /// Numéro de semaine ISO
    pub week_number: u32,
    /// Temps de présence effective (minutes réelles avec déductions)
    pub effective_minutes: u32,
    /// Heures sup consommées (excès au-dessus du seuil)
    pub consumed_overtime_minutes: u32,
}

/// Vue principale des données analytiques
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDataView {
    /// Statistiques par jour de la semaine (0=Lundi, 1=Mardi, etc.)
    pub day_of_week_stats: Vec<DayOfWeekStats>,
    /// Tendance des heures travaillées sur les dernières semaines
    pub weekly_trends: Vec<WeeklyTrendPoint>,
    /// Statistiques mensuelles
    pub monthly_stats: Vec<MonthlyStatsView>,
    /// Nombre total de semaines enregistrées
    pub total_weeks: u32,
    /// Courbes comparatives pour les 12 dernières semaines
    pub weekly_curves: Vec<WeekAnalyticsPoint>,
}

/// Statistiques pour un jour de la semaine spécifique
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayOfWeekStats {
    /// Nom du jour
    pub day_name: String,
    /// Heures moyennes travaillées pour ce jour (en minutes)
    pub average_minutes: u32,
}

/// Point de données pour la tendance hebdomadaire
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyTrendPoint {
    /// Date de début de la semaine (format ISO)
    pub week_start: String,
    /// Heures totales travaillées cette semaine (en minutes)
    pub total_minutes: u32,
}

/// Statistiques mensuelles agrégées
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyStatsView {
    /// Année et mois (format: YYYY-MM)
    pub month: String,
    /// Libellé formaté des heures totales
    pub total_label: String,
    /// Libellé formaté de la moyenne hebdomadaire
    pub weekly_average_label: String,
}

pub fn settings_to_view(settings: &AppSettings) -> SettingsView {
    SettingsView {
        overtime_threshold_minutes: settings.overtime_threshold.0,
        overtime_threshold_label: minutes_to_human_label(settings.overtime_threshold.0),
        theme: settings.theme.to_string(),
        default_start: settings.default_work_interval.start.to_hhmm(),
        default_end: settings.default_work_interval.end.to_hhmm(),
        default_break: TimeOfDay(settings.default_break_minutes.0).to_hhmm(),
        configured_days: settings
            .configured_days
            .iter()
            .map(configured_day_to_view)
            .collect(),
        active_week_id: settings.active_week_id.as_ref().map(|week_id| week_id.0.clone()),
        enable_travel_deduction: settings.enable_travel_deduction,
        travel_deduction_minutes: settings.travel_deduction_minutes.0,
        travel_deduction_label: format!("{} min", settings.travel_deduction_minutes.0),
        vacation_day_hours: settings.vacation_day_hours,
        vacation_day_hours_label: format!("{:.1}h", settings.vacation_day_hours as f64 / 60.0),
    }
}

pub fn configured_day_to_view(day: &ConfiguredDay) -> ConfiguredDayView {
    ConfiguredDayView {
        day_id: day.day_id.0,
        label: day.label.0.clone(),
        enabled: day.enabled,
    }
}

pub fn week_to_view(
    week: &WeekSheet,
    cumulative_balance_minutes: Option<i32>,
) -> Result<WeekSheetView, crate::domain::errors::ValidationError> {
    let summary = summarize_week(week)?;
    let entries = week
        .entries
        .iter()
        .map(|entry| {
            let total_minutes = calculate_day_minutes(entry, week.travel_deduction_minutes, week.vacation_day_hours)?;
            Ok(DayEntryView {
                day_id: entry.day_id.0,
                label: entry.label.0.clone(),
                enabled: entry.enabled,
                start: entry.interval.as_ref().map(|interval| interval.start.to_hhmm()),
                end: entry.interval.as_ref().map(|interval| interval.end.to_hhmm()),
                break_time: crate::domain::types::TimeOfDay(entry.break_minutes.0).to_hhmm(),
                has_departure_deduction: entry.has_departure_deduction,
                has_return_deduction: entry.has_return_deduction,
                total_minutes,
                total_label: minutes_to_label(total_minutes),
                day_type: match entry.day_type {
                    DayType::Work => "work".to_string(),
                    DayType::Vacation => "vacation".to_string(),
                    DayType::PublicHoliday => "public_holiday".to_string(),
                    DayType::Disabled => "disabled".to_string(),
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(WeekSheetView {
        week_id: week.week_id.clone().map(|id| id.0),
        week_start: week.week_start.as_string(),
        entries,
        overtime_threshold_minutes: week.overtime_threshold.0,
        travel_deduction_minutes: week.travel_deduction_minutes.0,
        summary: WeekSummaryView {
            total_label: minutes_to_label(summary.total_minutes),
            percentage: threshold_percentage(summary.total_minutes, week.overtime_threshold.0),
            cumulative_balance_minutes: cumulative_balance_minutes.unwrap_or(0),
            cumulative_balance_label: cumulative_balance_minutes
                .map(|balance| crate::domain::logic::signed_minutes_to_label(balance))
                .unwrap_or_else(|| "--".to_string()),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test de non-régression : day_type doit être correctement désérialisé
    /// Issue : le frontend ne passait pas dayType dans buildSaveInput, ce qui
    /// faisait que les changements vacation/congé n'étaient pas persistés.
    #[test]
    fn save_week_day_entry_input_deserializes_day_type() {
        let json = r#"{
            "dayId": 0,
            "label": "Lundi",
            "enabled": true,
            "start": "09:00",
            "end": "17:00",
            "breakTime": "01:00",
            "hasDepartureDeduction": false,
            "hasReturnDeduction": false,
            "dayType": "vacation"
        }"#;

        let input: SaveWeekDayEntryInput = serde_json::from_str(json).expect("should parse");
        assert_eq!(input.day_type, Some("vacation".to_string()));
    }

    #[test]
    fn save_week_day_entry_input_handles_missing_day_type() {
        // Pour la rétro-compatibilité, day_type peut être absent
        let json = r#"{
            "dayId": 0,
            "label": "Lundi",
            "enabled": true,
            "start": "09:00",
            "end": "17:00",
            "breakTime": "01:00",
            "hasDepartureDeduction": false,
            "hasReturnDeduction": false
        }"#;

        let input: SaveWeekDayEntryInput = serde_json::from_str(json).expect("should parse");
        assert_eq!(input.day_type, None);
    }

    #[test]
    fn save_week_input_includes_day_type_in_entries() {
        // Vérifie que le JSON complet avec dayType pour chaque entrée est valide
        let json = r#"{
            "weekId": "test-week",
            "weekStart": "2026-01-05",
            "overtimeThresholdMinutes": 2100,
            "travelDeductionMinutes": 30,
            "entries": [
                {
                    "dayId": 0,
                    "label": "Lundi",
                    "enabled": true,
                    "start": "09:00",
                    "end": "17:00",
                    "breakTime": "01:00",
                    "hasDepartureDeduction": false,
                    "hasReturnDeduction": false,
                    "dayType": "vacation"
                },
                {
                    "dayId": 1,
                    "label": "Mardi",
                    "enabled": true,
                    "start": "08:00",
                    "end": "18:00",
                    "breakTime": "01:00",
                    "hasDepartureDeduction": true,
                    "hasReturnDeduction": true,
                    "dayType": "work"
                }
            ]
        }"#;

        let input: SaveWeekInput = serde_json::from_str(json).expect("should parse");
        assert_eq!(input.entries.len(), 2);
        assert_eq!(input.entries[0].day_type, Some("vacation".to_string()));
        assert_eq!(input.entries[1].day_type, Some("work".to_string()));
    }
}
