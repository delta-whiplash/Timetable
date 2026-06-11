use crate::application::dto::{DayOfWeekStats, MonthlyStatsView, WeeklyTrendPoint};
use crate::domain::{
    errors::StorageError,
    types::{AppMetadata, AppSettings, DiagnosticSnapshot, WeekId, WeekSheet, WeekStartDate},
};

/// Trait pour la récupération des données analytiques
pub trait AnalyticsRepository: Send + Sync {
    /// Récupère les statistiques par jour de la semaine
    fn get_day_of_week_stats(&self) -> Result<Vec<DayOfWeekStats>, StorageError>;

    /// Récupère les tendances hebdomadaires (dernières 12 semaines)
    fn get_weekly_trends(&self) -> Result<Vec<WeeklyTrendPoint>, StorageError>;

    /// Récupère les statistiques mensuelles
    fn get_monthly_stats(&self) -> Result<Vec<MonthlyStatsView>, StorageError>;
}

pub trait WeekRepository: Send + Sync {
    fn migrate(&self) -> Result<(), StorageError>;
    fn get_week_by_id(&self, week_id: &WeekId) -> Result<Option<WeekSheet>, StorageError>;
    fn get_week_by_start(&self, week_start: &WeekStartDate) -> Result<Option<WeekSheet>, StorageError>;
    fn save_week(&self, week: &WeekSheet) -> Result<(), StorageError>;
    fn list_weeks(&self) -> Result<Vec<WeekSheet>, StorageError>;
    fn delete_week(&self, week_id: &WeekId) -> Result<(), StorageError>;
    fn get_cumulative_balance(&self, up_to_week_start: &WeekStartDate) -> Result<i32, StorageError>;
    fn metadata(&self) -> Result<AppMetadata, StorageError>;
    fn ping(&self) -> Result<(), StorageError>;
}

pub trait SettingsRepository: Send + Sync {
    fn ensure_default_settings(&self) -> Result<(), StorageError>;
    fn load_settings(&self) -> Result<AppSettings, StorageError>;
    fn save_settings(&self, settings: &AppSettings) -> Result<(), StorageError>;
}

pub trait DiagnosticsStore: Send + Sync {
    fn save_snapshot(&self, snapshot: &DiagnosticSnapshot) -> Result<(), StorageError>;
    fn latest_snapshot_id(&self) -> Result<Option<String>, StorageError>;
}
