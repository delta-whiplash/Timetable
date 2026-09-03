use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeekId(pub String);

impl WeekId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DayId(pub u8);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeekStartDate(pub NaiveDate);

impl WeekStartDate {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        let parsed =
            NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| ValidationError::InvalidWeekStart)?;
        Ok(Self::normalized(parsed))
    }

    pub fn today() -> Self {
        // Heure locale : la semaine active doit suivre le calendrier de
        // l'utilisateur, pas UTC (sinon lundi 00h30 a Paris = dimanche en UTC).
        Self::normalized(Local::now().date_naive())
    }

    pub fn normalized(date: NaiveDate) -> Self {
        let delta = i64::from(date.weekday().num_days_from_monday());
        Self(date - chrono::Duration::days(delta))
    }

    pub fn as_string(&self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DayLabel(pub String);

impl DayLabel {
    pub fn parse(day_id: DayId, value: &str) -> Result<Self, ValidationError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyLabel { day_id: day_id.0 });
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeOfDay(pub u16);

impl TimeOfDay {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(ValidationError::InvalidTimeFormat);
        }

        // Normaliser H/h en séparateur :
        let value = value.replace('h', ":").replace('H', ":");

        let (hours, minutes) = if let Some((h, m)) = value.split_once(':') {
            // Format HH:MM ou HH: (minutes optionnelles)
            let hours: u16 = h.parse().map_err(|_| ValidationError::InvalidTimeFormat)?;
            let minutes: u16 = if m.is_empty() {
                0
            } else {
                m.parse().map_err(|_| ValidationError::InvalidTimeFormat)?
            };
            (hours, minutes)
        } else if value.len() == 4 && value.chars().all(|c| c.is_ascii_digit()) {
            // Format compact 4 chiffres (1830 -> 18:30)
            let hours: u16 = value[0..2].parse().unwrap();
            let minutes: u16 = value[2..4].parse().unwrap();
            (hours, minutes)
        } else if value.chars().all(|c| c.is_ascii_digit()) {
            // Heures seules (18 -> 18:00)
            let hours: u16 = value
                .parse()
                .map_err(|_| ValidationError::InvalidTimeFormat)?;
            (hours, 0)
        } else {
            return Err(ValidationError::InvalidTimeFormat);
        };

        if hours > 23 || minutes > 59 {
            return Err(ValidationError::InvalidTimeFormat);
        }
        Ok(Self(hours * 60 + minutes))
    }

    pub fn to_hhmm(self) -> String {
        format!("{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BreakMinutes(pub u16);

impl BreakMinutes {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        Ok(Self(TimeOfDay::parse(value)?.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OvertimeThresholdMinutes(pub u16);

impl OvertimeThresholdMinutes {
    pub fn new(value: u16) -> Result<Self, ValidationError> {
        if value == 0 {
            return Err(ValidationError::InvalidThreshold);
        }
        Ok(Self(value))
    }
}

/// Durée de déduction pour trajet (en minutes)
/// Valeur configurable entre 15 et 180 minutes (default: 30)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TravelDeductionMinutes(pub u16);

impl TravelDeductionMinutes {
    pub const DEFAULT: u16 = 30;
    pub const MIN: u16 = 15;
    pub const MAX: u16 = 180;

    pub fn new(value: u16) -> Result<Self, ValidationError> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(ValidationError::InvalidTravelDeductionMinutes { value });
        }
        Ok(Self(value))
    }
}

impl Default for TravelDeductionMinutes {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkInterval {
    pub start: TimeOfDay,
    pub end: TimeOfDay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayEntry {
    pub day_id: DayId,
    pub label: DayLabel,
    pub interval: Option<WorkInterval>,
    pub break_minutes: BreakMinutes,
    pub enabled: bool,
    /// Déduction 30min sur l'heure de départ (déplacement domicile-travail)
    pub has_departure_deduction: bool,
    /// Déduction 30min sur l'heure de retour (déplacement travail-domicile)
    pub has_return_deduction: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredDay {
    pub day_id: DayId,
    pub label: DayLabel,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekSheet {
    pub week_id: Option<WeekId>,
    pub week_start: WeekStartDate,
    pub entries: Vec<DayEntry>,
    pub overtime_threshold: OvertimeThresholdMinutes,
    /// Snapshot de la déduction trajet au moment de la création de la semaine
    pub travel_deduction_minutes: TravelDeductionMinutes,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekSummary {
    pub total_minutes: u16,
    pub worked_days: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Light,
    Dark,
}

impl std::fmt::Display for ThemePreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        })
    }
}

impl std::str::FromStr for ThemePreference {
    type Err = super::errors::ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "light" => Ok(ThemePreference::Light),
            "dark" => Ok(ThemePreference::Dark),
            _ => Err(super::errors::ConfigError::Invalid),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub overtime_threshold: OvertimeThresholdMinutes,
    pub theme: ThemePreference,
    pub default_work_interval: WorkInterval,
    pub default_break_minutes: BreakMinutes,
    pub configured_days: Vec<ConfiguredDay>,
    pub active_week_id: Option<WeekId>,
    /// Active/désactive la fonctionnalité de déduction trajet
    pub enable_travel_deduction: bool,
    /// Durée de déduction par case cochée (snapshot au moment de la création de semaine)
    pub travel_deduction_minutes: TravelDeductionMinutes,
}

pub fn default_configured_days() -> Vec<ConfiguredDay> {
    [
        (0, "Lundi", true),
        (1, "Mardi", true),
        (2, "Mercredi", true),
        (3, "Jeudi", true),
        (4, "Vendredi", true),
        (5, "Samedi", false),
        (6, "Dimanche", false),
    ]
    .into_iter()
    .map(|(day_id, label, enabled)| ConfiguredDay {
        day_id: DayId(day_id),
        label: DayLabel(label.to_string()),
        enabled,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_est_toujours_un_lundi() {
        // Garantit que today() reste normalisée au lundi de la semaine locale
        let today = WeekStartDate::today();
        assert_eq!(today.0.weekday().num_days_from_monday(), 0);
    }

    #[test]
    fn test_parse_time_formats() {
        // Formats valides
        assert_eq!(TimeOfDay::parse("18:00").unwrap().0, 18 * 60);
        assert_eq!(TimeOfDay::parse("8:30").unwrap().0, 8 * 60 + 30);
        assert_eq!(TimeOfDay::parse("18H").unwrap().0, 18 * 60);
        assert_eq!(TimeOfDay::parse("18h").unwrap().0, 18 * 60);
        assert_eq!(TimeOfDay::parse("18H30").unwrap().0, 18 * 60 + 30);
        assert_eq!(TimeOfDay::parse("18h30").unwrap().0, 18 * 60 + 30);
        assert_eq!(TimeOfDay::parse("18").unwrap().0, 18 * 60);
        assert_eq!(TimeOfDay::parse("8").unwrap().0, 8 * 60);
        assert_eq!(TimeOfDay::parse("1830").unwrap().0, 18 * 60 + 30);
        assert_eq!(TimeOfDay::parse("0830").unwrap().0, 8 * 60 + 30);

        // Formats invalides
        assert!(TimeOfDay::parse("25:00").is_err());
        assert!(TimeOfDay::parse("18:60").is_err());
        assert!(TimeOfDay::parse("").is_err());
        assert!(TimeOfDay::parse("abc").is_err());
    }
}
