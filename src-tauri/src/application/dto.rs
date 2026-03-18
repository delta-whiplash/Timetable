use serde::{Deserialize, Serialize};

use crate::domain::{
    logic::{calculate_day_minutes, minutes_to_human_label, minutes_to_label, quick_read, summarize_week},
    types::{
        AppSettings, ConfiguredDay, DayWorkSummary, ThemePreference, WeekSheet,
        TimeOfDay,
    },
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
pub struct ThemeInput {
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeView {
    pub theme: String,
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
pub struct DaySummaryView {
    pub day_id: u8,
    pub label: String,
    pub worked_minutes: u16,
    pub worked_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekSummaryView {
    pub total_minutes: u16,
    pub total_label: String,
    pub overtime_minutes: u16,
    pub overtime_label: String,
    pub average_minutes: u16,
    pub average_label: String,
    pub worked_days: u8,
    pub longest_day: Option<DaySummaryView>,
    pub shortest_day: Option<DaySummaryView>,
    pub quick_read: String,
    pub percentage: u8,
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
    pub theme: String,
    pub overtime_threshold_minutes: u16,
    pub active_week: WeekSheetView,
    pub config_checksum: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatusView {
    pub version: String,
    pub config_checksum: String,
    pub storage_status: String,
    pub latest_migration_status: String,
    pub active_week_id: Option<String>,
    pub latest_diagnostic_snapshot_id: Option<String>,
}

/// Export complet des données utilisateur
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataExport {
    pub version: String,
    pub exported_at: String,
    pub settings: SettingsView,
    pub weeks: Vec<WeekSheetView>,
}

/// Import complet des données utilisateur
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataImport {
    pub settings: SettingsView,
    pub weeks: Vec<WeekSheetView>,
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
    /// Index du jour (0=Lundi, 1=Mardi, ..., 6=Dimanche)
    pub day_index: u8,
    /// Nom du jour
    pub day_name: String,
    /// Nombre total d'entrées pour ce jour
    pub entry_count: u32,
    /// Heures moyennes travaillées pour ce jour (en minutes)
    pub average_minutes: u32,
    /// Libellé formaté des heures moyennes
    pub average_label: String,
    /// Heures totales travaillées pour ce jour (en minutes)
    pub total_minutes: u32,
    /// Libellé formaté des heures totales
    pub total_label: String,
}

/// Point de données pour la tendance hebdomadaire
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyTrendPoint {
    /// Date de début de la semaine (format ISO)
    pub week_start: String,
    /// Heures totales travaillées cette semaine (en minutes)
    pub total_minutes: u32,
    /// Libellé formaté des heures
    pub total_label: String,
    /// Nombre de jours travaillés cette semaine
    pub worked_days: u8,
    /// Heures supplémentaires (en minutes)
    pub overtime_minutes: u32,
    /// Libellé formaté des heures supplémentaires
    pub overtime_label: String,
}

/// Statistiques mensuelles agrégées
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyStatsView {
    /// Année et mois (format: YYYY-MM)
    pub month: String,
    /// Nombre de semaines avec des données ce mois-ci
    pub weeks_count: u32,
    /// Heures totales travaillées ce mois (en minutes)
    pub total_minutes: u32,
    /// Libellé formaté des heures totales
    pub total_label: String,
    /// Moyenne hebdomadaire ce mois (en minutes)
    pub weekly_average_minutes: u32,
    /// Libellé formaté de la moyenne hebdomadaire
    pub weekly_average_label: String,
}

fn theme_to_string(theme: ThemePreference) -> String {
    match theme {
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
    }
    .to_string()
}

fn day_summary_to_view(day_summary: Option<DayWorkSummary>) -> Option<DaySummaryView> {
    day_summary.map(|item| DaySummaryView {
        day_id: item.day_id.0,
        label: item.label.0,
        worked_minutes: item.worked_minutes.0,
        worked_label: minutes_to_label(item.worked_minutes.0),
    })
}

pub fn settings_to_view(settings: &AppSettings) -> SettingsView {
    SettingsView {
        overtime_threshold_minutes: settings.overtime_threshold.0,
        overtime_threshold_label: minutes_to_human_label(settings.overtime_threshold.0),
        theme: theme_to_string(settings.theme),
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

pub fn week_to_view(week: &WeekSheet) -> Result<WeekSheetView, crate::domain::errors::ValidationError> {
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
                start: entry.intervals.first().map(|interval| interval.start.to_hhmm()),
                end: entry.intervals.first().map(|interval| interval.end.to_hhmm()),
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
            total_minutes: summary.total_minutes.0,
            total_label: minutes_to_label(summary.total_minutes.0),
            overtime_minutes: summary.overtime_minutes.0,
            overtime_label: minutes_to_label(summary.overtime_minutes.0),
            average_minutes: summary.average_minutes.0,
            average_label: minutes_to_label(summary.average_minutes.0),
            worked_days: summary.worked_days,
            longest_day: day_summary_to_view(summary.longest_day.clone()),
            shortest_day: day_summary_to_view(summary.shortest_day.clone()),
            quick_read: quick_read(&summary),
            percentage: if week.overtime_threshold.0 > 0 {
                ((summary.total_minutes.0 as f64 / week.overtime_threshold.0 as f64) * 100.0) as u8
            } else {
                0
            },
        },
    })
}
