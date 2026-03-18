use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    #[error("invalid_time_range")]
    InvalidTimeRange { day_id: u8 },
    #[error("break_exceeds_day")]
    BreakExceedsDay { day_id: u8 },
    #[error("empty_label")]
    EmptyLabel { day_id: u8 },
    #[error("too_many_intervals")]
    TooManyIntervals { day_id: u8 },
    #[error("missing_time_input")]
    MissingTimeInput { day_id: u8 },
    #[error("invalid_threshold")]
    InvalidThreshold,
    #[error("invalid_week_start")]
    InvalidWeekStart,
    #[error("invalid_time_format")]
    InvalidTimeFormat,
    #[error("invalid_day_configuration")]
    InvalidDayConfiguration,
}

#[derive(Debug, Error, Clone)]
pub enum StorageError {
    #[error("storage_unavailable")]
    StorageUnavailable,
    #[error("migration_failed")]
    MigrationFailed,
    #[error("query_failed")]
    QueryFailed,
    #[error("serialization_failed")]
    SerializationFailed,
    #[error("entity_not_found")]
    EntityNotFound,
}

#[derive(Debug, Error, Clone)]
pub enum ConfigError {
    #[error("config_invalid")]
    Invalid,
    #[error("serialization_failed")]
    Serialization { details: String },
}

#[derive(Debug, Error, Clone)]
pub enum ApplicationError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Config(#[from] ConfigError),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicError {
    pub code: String,
    pub message: String,
    pub correlation_id: String,
    pub retryable: bool,
}

impl ApplicationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(error) => match error {
                ValidationError::InvalidTimeRange { .. } => "validation.invalid_time_range",
                ValidationError::BreakExceedsDay { .. } => "validation.break_exceeds_day",
                ValidationError::EmptyLabel { .. } => "validation.empty_label",
                ValidationError::TooManyIntervals { .. } => "validation.too_many_intervals",
                ValidationError::MissingTimeInput { .. } => "validation.missing_time_input",
                ValidationError::InvalidThreshold => "validation.invalid_threshold",
                ValidationError::InvalidWeekStart => "validation.invalid_week_start",
                ValidationError::InvalidTimeFormat => "validation.invalid_time_format",
                ValidationError::InvalidDayConfiguration => "validation.invalid_day_configuration",
            },
            Self::Storage(error) => match error {
                StorageError::StorageUnavailable => "storage.unavailable",
                StorageError::MigrationFailed => "storage.migration_failed",
                StorageError::QueryFailed => "storage.query_failed",
                StorageError::SerializationFailed => "storage.serialization_failed",
                StorageError::EntityNotFound => "storage.entity_not_found",
            },
            Self::Config(ConfigError::Invalid) => "config.invalid",
            Self::Config(ConfigError::Serialization { .. }) => "config.serialization_failed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Validation(error) => match error {
                ValidationError::InvalidTimeRange { .. } => {
                    "L'heure de fin doit être après l'heure de début."
                }
                ValidationError::BreakExceedsDay { .. } => {
                    "La pause ne peut pas être plus longue que la journée."
                }
                ValidationError::EmptyLabel { .. } => "Chaque jour doit avoir un libellé valide.",
                ValidationError::TooManyIntervals { .. } => {
                    "La version actuelle n'autorise qu'une seule plage horaire par jour."
                }
                ValidationError::MissingTimeInput { .. } => {
                    "Les heures de début et de fin sont requises pour un jour actif."
                }
                ValidationError::InvalidThreshold => {
                    "Le seuil d'heures supplémentaires doit être supérieur à zéro."
                }
                ValidationError::InvalidWeekStart => {
                    "La date de semaine fournie est invalide."
                }
                ValidationError::InvalidTimeFormat => {
                    "Le format horaire attendu est HH:MM."
                }
                ValidationError::InvalidDayConfiguration => {
                    "La configuration des jours est invalide."
                }
            },
            Self::Storage(StorageError::StorageUnavailable) => {
                "Le stockage local est momentanément indisponible."
            }
            Self::Storage(StorageError::MigrationFailed) => {
                "La base locale n'a pas pu être initialisée correctement."
            }
            Self::Storage(StorageError::QueryFailed) => {
                "Une opération locale a échoué."
            }
            Self::Storage(StorageError::SerializationFailed) => {
                "Une erreur de sérialisation locale s'est produite."
            }
            Self::Storage(StorageError::EntityNotFound) => {
                "L'élément demandé n'existe plus."
            }
            Self::Config(ConfigError::Invalid) => {
                "La configuration chargée n'est pas valide."
            }
            Self::Config(ConfigError::Serialization { .. }) => {
                "Erreur lors de la sérialisation des données."
            }
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::Storage(StorageError::StorageUnavailable | StorageError::QueryFailed)
        )
    }
}

impl From<ApplicationError> for PublicError {
    fn from(value: ApplicationError) -> Self {
        Self {
            code: value.code().to_string(),
            message: value.user_message().to_string(),
            correlation_id: Uuid::new_v4().to_string(),
            retryable: value.retryable(),
        }
    }
}
