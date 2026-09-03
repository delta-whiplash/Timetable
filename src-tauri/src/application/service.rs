use std::sync::Arc;

use tracing::info;

use crate::{
    application::{
        dto::{
            settings_to_view, week_to_view, AnalyticsDataView, BootstrapState, DeleteWeekInput,
            SaveSettingsInput, SaveWeekDayEntryInput, SaveWeekInput, SettingsView, WeekAnalyticsPoint, WeekListItem,
            WeekSelectorInput, WeekSheetView,
        },
        export::{build_export_sheet, sheet_to_xlsx},
    },
    domain::{
        errors::{ApplicationError, StorageError, ValidationError},
        logic::{default_entries, minutes_to_label, summarize_week},
        types::{
            AppSettings, BreakMinutes, ConfiguredDay, DayEntry, DayId, DayLabel, DayType,
            OvertimeThresholdMinutes, TimeOfDay, TravelDeductionMinutes, WeekId, WeekSheet,
            WeekStartDate, WorkInterval,
        },
    },
    infrastructure::duckdb::DuckDb,
};

/// Fichier prêt à écrire : nom de sortie (pour la boîte de dialogue) +
/// contenu bytes (XLSX déjà sérialisé).
pub struct ExportedFile {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

pub struct ApplicationService {
    store: Arc<DuckDb>,
}

impl ApplicationService {
    pub fn new(store: Arc<DuckDb>) -> Self {
        Self { store }
    }

    fn get_settings(&self) -> Result<AppSettings, ApplicationError> {
        Ok(self.store.load_settings()?)
    }

    fn week_to_view_with_balance(&self, week: &WeekSheet) -> Result<WeekSheetView, ApplicationError> {
        // Issue #1 : pas de solde cumulé si la semaine n'est pas persistée (week_id: None)
        let balance = match &week.week_id {
            Some(_) => {
                match self.store.get_cumulative_balance(&week.week_start) {
                    Ok(value) => Some(value),
                    Err(error) => {
                        tracing::error!(week_start = %week.week_start.as_string(), ?error, "Failed to compute cumulative balance");
                        Some(0)
                    }
                }
            }
            None => None,
        };
        week_to_view(week, balance).map_err(ApplicationError::from)
    }

    fn persist_settings(&self, settings: &AppSettings) -> Result<(), ApplicationError> {
        Ok(self.store.save_settings(settings)?)
    }

    pub fn load_bootstrap(&self) -> Result<BootstrapState, ApplicationError> {
        let settings = self.get_settings()?;
        let active_week = self.resolve_active_week(&settings)?;
        Ok(BootstrapState {
            active_week,
        })
    }

    pub fn save_week(&self, input: SaveWeekInput) -> Result<WeekSheetView, ApplicationError> {
        let mut week = self.parse_week_input(input)?;

        // Issue #1 : générer un ID si week_id est None (nouvelle semaine)
        if week.week_id.is_none() {
            week.week_id = Some(WeekId::new());
        }

        // Id obsolète (fenêtre périmée, double instance) : adopte la ligne
        // existante de la même semaine au lieu de violer l'index unique.
        if let Some(existing) = self.store.get_week_by_start(&week.week_start)? {
            week.week_id = Some(existing.week_id.expect("persisted week must have id"));
        }

        // Valide la semaine avant écriture (fail-closed si invalide)
        summarize_week(&week)?;

        self.store.save_week(&week)?;
        let mut settings = self.get_settings()?;
        settings.active_week_id = week.week_id.clone();
        self.persist_settings(&settings)?;
        info!(week_id = ?week.week_id, "week saved");

        // Construit la vue APRÈS sauvegarde pour avoir un solde cumulé frais
        let view = self.week_to_view_with_balance(&week)?;
        Ok(view)
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
                    week_id: week.week_id.expect("persisted week must have id").0,
                    week_start: week.week_start.as_string(),
                    total_minutes: summary.total_minutes,
                    total_label: minutes_to_label(summary.total_minutes),
                    worked_days: summary.worked_days,
                    updated_at: week.updated_at,
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
            self.persist_settings(&settings)?;
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

        // Nouvelles options de déduction trajet
        settings.enable_travel_deduction = input.enable_travel_deduction;
        settings.travel_deduction_minutes = TravelDeductionMinutes::new(input.travel_deduction_minutes)?;

        self.persist_settings(&settings)?;

        // Rétro-écrire le seuil UNIQUEMENT si la semaine active est la semaine courante
        // Sinon on modifie l'historique passé, ce qui invalide les exports précédents
        if let Some(active_week_id) = settings.active_week_id.clone() {
            if let Some(week) = self.store.get_week_by_id(&active_week_id)? {
                let today = WeekStartDate::today();
                if week.week_start == today {
                    let mut updated_week = week;
                    updated_week.overtime_threshold = settings.overtime_threshold;
                    self.store.save_week(&updated_week)?;
                }
            }
        }

        Ok(settings_to_view(&settings))
    }

    pub fn set_theme(&self, theme: String) -> Result<(), ApplicationError> {
        let mut settings = self.get_settings()?;
        settings.theme = theme.parse()?;
        self.persist_settings(&settings)
    }

    /// Récupère les données analytiques agrégées
    /// Statistiques par jour de la semaine, tendances hebdomadaires et mensuelles
    pub fn get_analytics(&self) -> Result<AnalyticsDataView, ApplicationError> {
        let day_of_week_stats = self.store.get_day_of_week_stats()?;
        let weekly_trends = self.store.get_weekly_trends()?;
        let monthly_stats = self.store.get_monthly_stats()?;
        let weekly_curves = self.store.get_weekly_curves()?;

        let total_weeks = self.store.list_weeks()?.len() as u32;

        Ok(AnalyticsDataView {
            day_of_week_stats,
            weekly_trends,
            monthly_stats,
            total_weeks,
            weekly_curves,
        })
    }

    pub fn export_week(&self, week_start: String) -> Result<ExportedFile, ApplicationError> {
        let start = WeekStartDate::parse(&week_start)?;
        let week = self
            .store
            .get_week_by_start(&start)?
            .ok_or(ApplicationError::Storage(StorageError::EntityNotFound))?;

        let balance = self.store.get_cumulative_balance(&week.week_start)?;
        let sheet = build_export_sheet(&week, balance).map_err(ApplicationError::from)?;
        let bytes = sheet_to_xlsx(&sheet);

        Ok(ExportedFile {
            file_name: format!("timetable-{}.xlsx", week_start),
            bytes,
        })
    }

    fn resolve_active_week(&self, settings: &AppSettings) -> Result<WeekSheetView, ApplicationError> {
        if let Some(active_week_id) = &settings.active_week_id {
            if let Ok(Some(week)) = self.store.get_week_by_id(active_week_id) {
                // Si la semaine est invalide (corrompue), fallback sur today
                if let Ok(view) = self.week_to_view_with_balance(&week) {
                    return Ok(view);
                }
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
            next_settings.active_week_id = Some(week.week_id.clone().expect("persisted week must have id"));
            self.persist_settings(&next_settings)?;
            return Ok(week);
        }

        // Issue #1 : template en mémoire avec week_id: None, pas de persistance
        // tant que l'utilisateur ne sauvegarde pas explicitement.
        // Le solde cumulé affichera "--" jusqu'à la première sauvegarde.
        // Snapshot de travel_deduction_minutes au moment de la création
        let week = WeekSheet {
            week_id: None,
            week_start,
            entries: default_entries(settings),
            overtime_threshold: settings.overtime_threshold,
            travel_deduction_minutes: settings.travel_deduction_minutes,
            updated_at: String::new(),
        };

        // Ne pas mettre à jour active_week_id tant que la semaine n'est pas sauvegardée
        // (sinon une semaine vide deviendrait la semaine active par défaut)
        Ok(week)
    }

    fn parse_week_input(&self, input: SaveWeekInput) -> Result<WeekSheet, ApplicationError> {
        let overtime_threshold = OvertimeThresholdMinutes::new(input.overtime_threshold_minutes)?;
        let travel_deduction_minutes = TravelDeductionMinutes::new(input.travel_deduction_minutes)?;
        let entries = input
            .entries
            .into_iter()
            .map(parse_day_entry_input)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(WeekSheet {
            week_id: input.week_id.map(WeekId),
            week_start: WeekStartDate::parse(&input.week_start)?,
            entries,
            overtime_threshold,
            travel_deduction_minutes,
            updated_at: String::new(),
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
        has_departure_deduction: input.has_departure_deduction,
        has_return_deduction: input.has_return_deduction,
        day_type: if input.enabled { DayType::Work } else { DayType::Disabled },
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
