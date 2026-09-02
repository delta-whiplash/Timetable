use std::sync::{Arc, RwLock};

use tracing::info;

use crate::{
    application::dto::{
        settings_to_view, week_to_view, AnalyticsDataView, BootstrapState, DeleteWeekInput,
        SaveSettingsInput, SaveWeekDayEntryInput, SaveWeekInput, SettingsView, WeekListItem,
        WeekSelectorInput, WeekSheetView,
    },
    domain::{
        errors::{ApplicationError, ConfigError, StorageError, ValidationError},
        logic::{default_entries, minutes_to_label, summarize_week},
        types::{
            AppSettings, BreakMinutes, ConfiguredDay, DayEntry, DayId, DayLabel,
            OvertimeThresholdMinutes, ThemePreference, TimeOfDay, WeekId, WeekSheet,
            WeekStartDate, WorkInterval,
        },
    },
    infrastructure::duckdb::DuckDb,
};

pub struct ApplicationService {
    store: Arc<DuckDb>,
    cached_settings: RwLock<Option<AppSettings>>,
}

impl ApplicationService {
    pub fn new(store: Arc<DuckDb>) -> Self {
        Self {
            store,
            cached_settings: RwLock::new(None),
        }
    }

    fn get_settings(&self) -> Result<AppSettings, ApplicationError> {
        {
            let cached = self.cached_settings.read()
                .map_err(|_| ApplicationError::Storage(StorageError::StorageUnavailable))?;
            if let Some(settings) = cached.as_ref() {
                return Ok(settings.clone());
            }
        }

        let settings = self.store.load_settings()?;
        {
            let mut cache = self.cached_settings.write()
                .map_err(|_| ApplicationError::Storage(StorageError::StorageUnavailable))?;
            *cache = Some(settings.clone());
        }
        Ok(settings)
    }

    fn week_to_view_with_balance(&self, week: &WeekSheet) -> Result<WeekSheetView, ApplicationError> {
        let balance = match self.store.get_cumulative_balance(&week.week_start) {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(week_start = %week.week_start.as_string(), ?error, "Failed to compute cumulative balance");
                0
            }
        };
        week_to_view(week, balance).map_err(ApplicationError::from)
    }

    fn save_settings_and_cache(&self, settings: &AppSettings) -> Result<(), ApplicationError> {
        self.store.save_settings(settings)?;
        {
            let mut cache = self.cached_settings.write()
                .map_err(|_| ApplicationError::Storage(StorageError::StorageUnavailable))?;
            *cache = Some(settings.clone());
        }
        Ok(())
    }

    pub fn load_bootstrap(&self) -> Result<BootstrapState, ApplicationError> {
        let settings = self.get_settings()?;
        let active_week = self.resolve_active_week(&settings)?;
        Ok(BootstrapState {
            active_week,
        })
    }

    pub fn save_week(&self, input: SaveWeekInput) -> Result<WeekSheetView, ApplicationError> {
        let week = self.parse_week_input(input)?;
        summarize_week(&week)?;
        self.store.save_week(&week)?;

        let mut settings = self.get_settings()?;
        settings.active_week_id = Some(week.week_id.clone());
        self.save_settings_and_cache(&settings)?;
        info!(week_id = %week.week_id.0, "week saved");
        self.week_to_view_with_balance(&week)
    }

    pub fn create_or_switch_week(
        &self,
        input: WeekSelectorInput,
    ) -> Result<WeekSheetView, ApplicationError> {
        let settings = self.get_settings()?;
        let week_start = WeekStartDate::parse(&input.week_start)?;
        let week = self.ensure_week_for_date(week_start, &settings)?;
        self.week_to_view_with_balance(&week)
    }

    pub fn list_weeks(&self) -> Result<Vec<WeekListItem>, ApplicationError> {
        let weeks = self.store.list_weeks()?;
        weeks.into_iter()
            .map(|week| {
                let summary = summarize_week(&week)?;
                Ok(WeekListItem {
                    week_id: week.week_id.0,
                    week_start: week.week_start.as_string(),
                    total_minutes: summary.total_minutes,
                    total_label: minutes_to_label(summary.total_minutes),
                    worked_days: summary.worked_days,
                    updated_at: format!("{} 00:00", week.week_start.as_string()),
                })
            })
            .collect::<Result<Vec<_>, ValidationError>>()
            .map_err(ApplicationError::from)
    }

    pub fn delete_week(&self, input: DeleteWeekInput) -> Result<(), ApplicationError> {
        let week_id = WeekId(input.week_id);
        self.store.delete_week(&week_id)?;
        let mut settings = self.get_settings()?;
        if settings.active_week_id.as_ref() == Some(&week_id) {
            settings.active_week_id = None;
            self.save_settings_and_cache(&settings)?;
        }
        Ok(())
    }

    pub fn load_settings(&self) -> Result<SettingsView, ApplicationError> {
        let settings = self.get_settings()?;
        Ok(settings_to_view(&settings))
    }

    pub fn save_settings(&self, input: SaveSettingsInput) -> Result<SettingsView, ApplicationError> {
        let mut settings = self.get_settings()?;
        settings.overtime_threshold = OvertimeThresholdMinutes::new(input.overtime_threshold_minutes)?;
        settings.configured_days = parse_configured_days(input.configured_days)?;

        // Parser et sauvegarder les nouvelles valeurs par défaut
        settings.default_work_interval = WorkInterval {
            start: TimeOfDay::parse(&input.default_start)?,
            end: TimeOfDay::parse(&input.default_end)?,
        };
        settings.default_break_minutes = BreakMinutes::parse(&input.default_break)?;

        self.save_settings_and_cache(&settings)?;

        if let Some(active_week_id) = settings.active_week_id.clone() {
            if let Some(mut week) = self.store.get_week_by_id(&active_week_id)? {
                week.overtime_threshold = settings.overtime_threshold;
                self.store.save_week(&week)?;
            }
        }

        Ok(settings_to_view(&settings))
    }

    pub fn set_theme(&self, theme: String) -> Result<(), ApplicationError> {
        let mut settings = self.get_settings()?;
        settings.theme = match theme.as_str() {
            "light" => ThemePreference::Light,
            "dark" => ThemePreference::Dark,
            _ => return Err(ApplicationError::Config(ConfigError::Invalid)),
        };
        self.save_settings_and_cache(&settings)
    }

    /// Récupère les données analytiques agrégées
    /// Statistiques par jour de la semaine, tendances hebdomadaires et mensuelles
    pub fn get_analytics(&self) -> Result<AnalyticsDataView, ApplicationError> {
        let day_of_week_stats = self.store.get_day_of_week_stats()?;
        let weekly_trends = self.store.get_weekly_trends()?;
        let monthly_stats = self.store.get_monthly_stats()?;

        let total_weeks = self.store.list_weeks()?.len() as u32;

        Ok(AnalyticsDataView {
            day_of_week_stats,
            weekly_trends,
            monthly_stats,
            total_weeks,
        })
    }

    fn resolve_active_week(&self, settings: &AppSettings) -> Result<WeekSheetView, ApplicationError> {
        if let Some(active_week_id) = &settings.active_week_id {
            if let Some(week) = self.store.get_week_by_id(active_week_id)? {
                return self.week_to_view_with_balance(&week);
            }
        }

        let week = self.ensure_week_for_date(WeekStartDate::today(), settings)?;
        self.week_to_view_with_balance(&week)
    }

    fn ensure_week_for_date(
        &self,
        week_start: WeekStartDate,
        settings: &AppSettings,
    ) -> Result<WeekSheet, ApplicationError> {
        if let Some(week) = self.store.get_week_by_start(&week_start)? {
            let mut next_settings = settings.clone();
            next_settings.active_week_id = Some(week.week_id.clone());
            self.save_settings_and_cache(&next_settings)?;
            return Ok(week);
        }

        let week = WeekSheet {
            week_id: WeekId::new(),
            week_start,
            entries: default_entries(&settings.configured_days),
            overtime_threshold: settings.overtime_threshold,
        };
        self.store.save_week(&week)?;

        let mut next_settings = settings.clone();
        next_settings.active_week_id = Some(week.week_id.clone());
        self.save_settings_and_cache(&next_settings)?;
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

    let interval = if input.enabled {
        let start = input
            .start
            .as_deref()
            .ok_or(ValidationError::MissingTimeInput { day_id: day_id.0 })?;
        let end = input
            .end
            .as_deref()
            .ok_or(ValidationError::MissingTimeInput { day_id: day_id.0 })?;
        Some(WorkInterval {
            start: TimeOfDay::parse(start)?,
            end: TimeOfDay::parse(end)?,
        })
    } else {
        None
    };

    Ok(DayEntry {
        day_id,
        label,
        interval,
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
