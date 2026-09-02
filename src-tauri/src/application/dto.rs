use serde::{Deserialize, Serialize};

use crate::domain::{
    logic::{
        calculate_day_minutes, minutes_to_human_label, minutes_to_label, summarize_week,
        threshold_percentage,
    },
    types::{AppSettings, ConfiguredDay, TimeOfDay, WeekSheet},
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWeekInput {
    pub week_id: String,
    pub week_start: String,
    pub overtime_threshold_minutes: u16,
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
    pub total_minutes: u16,
    pub total_label: String,
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
    pub week_id: String,
    pub week_start: String,
    pub entries: Vec<DayEntryView>,
    pub overtime_threshold_minutes: u16,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    pub active_week: WeekSheetView,
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
    cumulative_balance_minutes: i32,
) -> Result<WeekSheetView, crate::domain::errors::ValidationError> {
    let summary = summarize_week(week)?;
    let entries = week
        .entries
        .iter()
        .map(|entry| {
            let total_minutes = calculate_day_minutes(entry)?;
            Ok(DayEntryView {
                day_id: entry.day_id.0,
                label: entry.label.0.clone(),
                enabled: entry.enabled,
                start: entry.interval.as_ref().map(|interval| interval.start.to_hhmm()),
                end: entry.interval.as_ref().map(|interval| interval.end.to_hhmm()),
                break_time: crate::domain::types::TimeOfDay(entry.break_minutes.0).to_hhmm(),
                total_minutes,
                total_label: minutes_to_label(total_minutes),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(WeekSheetView {
        week_id: week.week_id.0.clone(),
        week_start: week.week_start.as_string(),
        entries,
        overtime_threshold_minutes: week.overtime_threshold.0,
        summary: WeekSummaryView {
            total_label: minutes_to_label(summary.total_minutes),
            percentage: threshold_percentage(summary.total_minutes, week.overtime_threshold.0),
            cumulative_balance_minutes,
            cumulative_balance_label: crate::domain::logic::signed_minutes_to_label(
                cumulative_balance_minutes,
            ),
        },
    })
}
