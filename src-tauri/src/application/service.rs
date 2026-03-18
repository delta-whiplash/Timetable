use std::sync::Arc;

use chrono::Utc;
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    application::{
        dto::{
            settings_to_view, week_to_view, AnalyticsDataView, AppStatusView, BootstrapState,
            DataExport, DeleteWeekInput, SaveSettingsInput, SaveWeekDayEntryInput, SaveWeekInput,
            SettingsView, ThemeInput, ThemeView, WeekListItem, WeekSelectorInput, WeekSheetView,
        },
        ports::{AnalyticsRepository, DiagnosticsStore, SettingsRepository, WeekRepository},
    },
    domain::{
        errors::{ApplicationError, ConfigError, ValidationError},
        logic::{default_entries, minutes_to_label, summarize_week},
        types::{
            AppSettings, BreakMinutes, ConfiguredDay, DayEntry, DayId, DayLabel, DefaultBreakMinutes,
            DefaultWorkInterval, DiagnosticSnapshot, OvertimeThresholdMinutes, ThemePreference,
            TimeOfDay, WeekId, WeekSheet, WeekStartDate, WorkInterval,
        },
    },
    infrastructure::config::AppRuntimeConfig,
};

pub struct ApplicationService {
    week_repository: Arc<dyn WeekRepository>,
    settings_repository: Arc<dyn SettingsRepository>,
    diagnostics_store: Arc<dyn DiagnosticsStore>,
    analytics_repository: Arc<dyn AnalyticsRepository>,
    runtime_config: AppRuntimeConfig,
}

impl ApplicationService {
    pub fn new(
        week_repository: Arc<dyn WeekRepository>,
        settings_repository: Arc<dyn SettingsRepository>,
        diagnostics_store: Arc<dyn DiagnosticsStore>,
        analytics_repository: Arc<dyn AnalyticsRepository>,
        runtime_config: AppRuntimeConfig,
    ) -> Self {
        Self {
            week_repository,
            settings_repository,
            diagnostics_store,
            analytics_repository,
            runtime_config,
        }
    }

    pub fn capture_error(&self, context: &str, error: &ApplicationError) {
        let snapshot = DiagnosticSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            created_at: Utc::now().to_rfc3339(),
            reason: context.to_string(),
            correlation_id: Uuid::new_v4().to_string(),
            payload_json: json!({
                "context": context,
                "code": error.code(),
                "retryable": error.retryable(),
                "version": self.runtime_config.version,
            })
            .to_string(),
        };

        if let Err(snapshot_error) = self.diagnostics_store.save_snapshot(&snapshot) {
            error!(
                code = error.code(),
                snapshot_code = ?snapshot_error,
                "unable to save diagnostic snapshot"
            );
        }
    }

    pub fn load_bootstrap(&self) -> Result<BootstrapState, ApplicationError> {
        let settings = self.settings_repository.load_settings()?;
        let active_week = self.resolve_active_week(&settings)?;
        Ok(BootstrapState {
            theme: settings_to_view(&settings).theme,
            overtime_threshold_minutes: settings.overtime_threshold.0,
            active_week,
            config_checksum: self.runtime_config.config_checksum.clone(),
            version: self.runtime_config.version.clone(),
        })
    }

    pub fn get_active_week(&self) -> Result<WeekSheetView, ApplicationError> {
        let settings = self.settings_repository.load_settings()?;
        self.resolve_active_week(&settings)
    }

    pub fn save_week(&self, input: SaveWeekInput) -> Result<WeekSheetView, ApplicationError> {
        let week = self.parse_week_input(input)?;
        summarize_week(&week)?;
        self.week_repository.save_week(&week)?;

        let mut settings = self.settings_repository.load_settings()?;
        settings.active_week_id = Some(week.week_id.clone());
        self.settings_repository.save_settings(&settings)?;
        info!(week_id = %week.week_id.0, "week saved");
        week_to_view(&week).map_err(ApplicationError::from)
    }

    pub fn create_or_switch_week(
        &self,
        input: WeekSelectorInput,
    ) -> Result<WeekSheetView, ApplicationError> {
        let settings = self.settings_repository.load_settings()?;
        let week_start = WeekStartDate::parse(&input.week_start)?;
        let week = self.ensure_week_for_date(week_start, &settings)?;
        week_to_view(&week).map_err(ApplicationError::from)
    }

    pub fn list_weeks(&self) -> Result<Vec<WeekListItem>, ApplicationError> {
        let weeks = self.week_repository.list_weeks()?;
        weeks.into_iter()
            .map(|week| {
                let summary = summarize_week(&week)?;
                Ok(WeekListItem {
                    week_id: week.week_id.0,
                    week_start: week.week_start.as_string(),
                    total_minutes: summary.total_minutes.0,
                    total_label: minutes_to_label(summary.total_minutes.0),
                    worked_days: summary.worked_days,
                    updated_at: format!("{} 00:00", week.week_start.as_string()),
                })
            })
            .collect::<Result<Vec<_>, ValidationError>>()
            .map_err(ApplicationError::from)
    }

    pub fn delete_week(&self, input: DeleteWeekInput) -> Result<(), ApplicationError> {
        let week_id = WeekId(input.week_id);
        self.week_repository.delete_week(&week_id)?;
        let mut settings = self.settings_repository.load_settings()?;
        if settings.active_week_id.as_ref() == Some(&week_id) {
            settings.active_week_id = None;
            self.settings_repository.save_settings(&settings)?;
        }
        Ok(())
    }

    pub fn load_settings(&self) -> Result<SettingsView, ApplicationError> {
        let settings = self.settings_repository.load_settings()?;
        Ok(settings_to_view(&settings))
    }

    pub fn save_settings(&self, input: SaveSettingsInput) -> Result<SettingsView, ApplicationError> {
        let mut settings = self.settings_repository.load_settings()?;
        settings.overtime_threshold = OvertimeThresholdMinutes::new(input.overtime_threshold_minutes)?;
        settings.configured_days = parse_configured_days(input.configured_days)?;

        // Parser et sauvegarder les nouvelles valeurs par défaut
        settings.default_work_interval = DefaultWorkInterval {
            start: TimeOfDay::parse(&input.default_start)?,
            end: TimeOfDay::parse(&input.default_end)?,
        };
        settings.default_break_minutes = DefaultBreakMinutes(BreakMinutes::parse(&input.default_break)?.0);

        self.settings_repository.save_settings(&settings)?;

        if let Some(active_week_id) = settings.active_week_id.clone() {
            if let Some(mut week) = self.week_repository.get_week_by_id(&active_week_id)? {
                week.overtime_threshold = settings.overtime_threshold;
                self.week_repository.save_week(&week)?;
            }
        }

        Ok(settings_to_view(&settings))
    }

    pub fn set_theme(&self, input: ThemeInput) -> Result<ThemeView, ApplicationError> {
        let mut settings = self.settings_repository.load_settings()?;
        settings.theme = match input.theme.as_str() {
            "light" => ThemePreference::Light,
            "dark" => ThemePreference::Dark,
            _ => return Err(ApplicationError::Config(ConfigError::Invalid)),
        };
        self.settings_repository.save_settings(&settings)?;
        Ok(ThemeView { theme: input.theme })
    }

    pub fn get_app_status(&self) -> Result<AppStatusView, ApplicationError> {
        let metadata = self.week_repository.metadata()?;
        let settings = self.settings_repository.load_settings()?;
        let storage_status = match self.week_repository.ping() {
            Ok(()) => "healthy",
            Err(error) => {
                self.capture_error("get_app_status.ping", &ApplicationError::Storage(error));
                "degraded"
            }
        };
        let latest_snapshot = self.diagnostics_store.latest_snapshot_id()?;

        Ok(AppStatusView {
            version: self.runtime_config.version.clone(),
            config_checksum: self.runtime_config.config_checksum.clone(),
            storage_status: storage_status.to_string(),
            latest_migration_status: metadata.latest_migration_status,
            active_week_id: settings.active_week_id.map(|item| item.0),
            latest_diagnostic_snapshot_id: latest_snapshot,
        })
    }

    /// Récupère les données analytiques agrégées
    /// Statistiques par jour de la semaine, tendances hebdomadaires et mensuelles
    pub fn get_analytics(&self) -> Result<AnalyticsDataView, ApplicationError> {
        let day_of_week_stats = self.analytics_repository.get_day_of_week_stats()?;
        let weekly_trends = self.analytics_repository.get_weekly_trends()?;
        let monthly_stats = self.analytics_repository.get_monthly_stats()?;

        let total_weeks = self.week_repository.list_weeks()?.len() as u32;

        Ok(AnalyticsDataView {
            day_of_week_stats,
            weekly_trends,
            monthly_stats,
            total_weeks,
        })
    }

    /// Export toutes les données de l'utilisateur (paramètres + semaines)
    pub fn export_data(&self) -> Result<String, ApplicationError> {
        let settings = self.settings_repository.load_settings()?;
        let settings_view = settings_to_view(&settings);

        let weeks = self.week_repository.list_weeks()?;
        let weeks_view = weeks
            .iter()
            .map(week_to_view)
            .collect::<Result<Vec<_>, _>>()
            .map_err(ApplicationError::from)?;

        let export = DataExport {
            version: self.runtime_config.version.clone(),
            exported_at: Utc::now().to_rfc3339(),
            settings: settings_view,
            weeks: weeks_view,
        };

        serde_json::to_string_pretty(&export)
            .map_err(|error| ApplicationError::Config(ConfigError::Serialization {
                details: error.to_string(),
            }))
    }

    /// Import des données utilisateur (paramètres + semaines)
    pub fn import_data(&self, json_data: String) -> Result<BootstrapState, ApplicationError> {
        let import: DataExport = serde_json::from_str(&json_data).map_err(|_| {
            ApplicationError::Config(ConfigError::Serialization {
                details: "Format de fichier invalide".to_string(),
            })
        })?;

        // Importer les paramètres
        let settings = self.settings_repository.load_settings()?;

        // Mettre à jour les paramètres depuis l'import
        let mut updated_settings = settings.clone();
        updated_settings.overtime_threshold =
            OvertimeThresholdMinutes::new(import.settings.overtime_threshold_minutes)?;
        updated_settings.theme = match import.settings.theme.as_str() {
            "light" => ThemePreference::Light,
            "dark" => ThemePreference::Dark,
            _ => return Err(ApplicationError::Config(ConfigError::Invalid)),
        };
        updated_settings.configured_days = import
            .settings
            .configured_days
            .into_iter()
            .map(|day| -> Result<ConfiguredDay, ValidationError> {
                Ok(ConfiguredDay {
                    day_id: DayId(day.day_id),
                    label: DayLabel::parse(DayId(day.day_id), &day.label)?,
                    enabled: day.enabled,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Importer les valeurs par défaut
        updated_settings.default_work_interval = DefaultWorkInterval {
            start: TimeOfDay::parse(&import.settings.default_start)?,
            end: TimeOfDay::parse(&import.settings.default_end)?,
        };
        updated_settings.default_break_minutes = DefaultBreakMinutes(BreakMinutes::parse(&import.settings.default_break)?.0);

        self.settings_repository.save_settings(&updated_settings)?;

        // Importer les semaines
        for week_view in import.weeks {
            let week = self.parse_week_input(SaveWeekInput {
                week_id: week_view.week_id.clone(),
                week_start: week_view.week_start.clone(),
                overtime_threshold_minutes: week_view.overtime_threshold_minutes,
                entries: week_view
                    .entries
                    .into_iter()
                    .map(|entry| SaveWeekDayEntryInput {
                        day_id: entry.day_id,
                        label: entry.label,
                        enabled: entry.enabled,
                        start: entry.start,
                        end: entry.end,
                        break_time: entry.break_time,
                    })
                    .collect(),
            })?;

            // Sauvegarder ou mettre à jour la semaine
            self.week_repository.save_week(&week)?;
        }

        // Retourner l'état bootstrap pour rafraîchir l'interface
        let active_week = self.resolve_active_week(&updated_settings)?;
        Ok(BootstrapState {
            theme: import.settings.theme,
            overtime_threshold_minutes: import.settings.overtime_threshold_minutes,
            active_week,
            config_checksum: self.runtime_config.config_checksum.clone(),
            version: self.runtime_config.version.clone(),
        })
    }

    fn resolve_active_week(&self, settings: &AppSettings) -> Result<WeekSheetView, ApplicationError> {
        if let Some(active_week_id) = &settings.active_week_id {
            if let Some(week) = self.week_repository.get_week_by_id(active_week_id)? {
                return week_to_view(&week).map_err(ApplicationError::from);
            }
        }

        let week = self.ensure_week_for_date(WeekStartDate::today(), settings)?;
        week_to_view(&week).map_err(ApplicationError::from)
    }

    fn ensure_week_for_date(
        &self,
        week_start: WeekStartDate,
        settings: &AppSettings,
    ) -> Result<WeekSheet, ApplicationError> {
        if let Some(week) = self.week_repository.get_week_by_start(&week_start)? {
            let mut next_settings = settings.clone();
            next_settings.active_week_id = Some(week.week_id.clone());
            self.settings_repository.save_settings(&next_settings)?;
            return Ok(week);
        }

        let week = WeekSheet {
            week_id: WeekId::new(),
            week_start,
            entries: default_entries(&settings.configured_days),
            overtime_threshold: settings.overtime_threshold,
        };
        self.week_repository.save_week(&week)?;

        let mut next_settings = settings.clone();
        next_settings.active_week_id = Some(week.week_id.clone());
        self.settings_repository.save_settings(&next_settings)?;
        Ok(week)
    }

    fn parse_week_input(&self, input: SaveWeekInput) -> Result<WeekSheet, ApplicationError> {
        let overtime_threshold = OvertimeThresholdMinutes::new(input.overtime_threshold_minutes)?;
        let entries = input
            .entries
            .into_iter()
            .map(parse_day_entry_input)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(WeekSheet {
            week_id: WeekId(input.week_id),
            week_start: WeekStartDate::parse(&input.week_start)?,
            entries,
            overtime_threshold,
        })
    }
}

fn parse_day_entry_input(input: SaveWeekDayEntryInput) -> Result<DayEntry, ValidationError> {
    let day_id = DayId(input.day_id);
    let label = DayLabel::parse(day_id, &input.label)?;
    let break_minutes = crate::domain::types::BreakMinutes::parse(&input.break_time)?;

    let intervals = if input.enabled {
        let start = input
            .start
            .as_deref()
            .ok_or(ValidationError::MissingTimeInput { day_id: day_id.0 })?;
        let end = input
            .end
            .as_deref()
            .ok_or(ValidationError::MissingTimeInput { day_id: day_id.0 })?;
        vec![WorkInterval {
            start: TimeOfDay::parse(start)?,
            end: TimeOfDay::parse(end)?,
        }]
    } else {
        Vec::new()
    };

    Ok(DayEntry {
        day_id,
        label,
        intervals,
        break_minutes,
        enabled: input.enabled,
    })
}

fn parse_configured_days(input: Vec<crate::application::dto::ConfiguredDayView>) -> Result<Vec<ConfiguredDay>, ValidationError> {
    if input.is_empty() {
        return Err(ValidationError::InvalidDayConfiguration);
    }

    input
        .into_iter()
        .map(|day| {
            Ok(ConfiguredDay {
                day_id: DayId(day.day_id),
                label: DayLabel::parse(DayId(day.day_id), &day.label)?,
                enabled: day.enabled,
            })
        })
        .collect()
}
