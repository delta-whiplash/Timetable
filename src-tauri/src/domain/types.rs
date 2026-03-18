use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[doc(alias = "week identifier")]
pub struct WeekId(pub String);

impl WeekId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for WeekId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DayId(pub u8);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[doc(alias = "week date")]
pub struct WeekStartDate(pub NaiveDate);

impl WeekStartDate {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        let parsed =
            NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| ValidationError::InvalidWeekStart)?;
        Ok(Self::normalized(parsed))
    }

    pub fn today() -> Self {
        Self::normalized(Utc::now().date_naive())
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
        let (hours, minutes) = value
            .split_once(':')
            .ok_or(ValidationError::InvalidTimeFormat)?;
        let hours: u16 = hours.parse().map_err(|_| ValidationError::InvalidTimeFormat)?;
        let minutes: u16 = minutes.parse().map_err(|_| ValidationError::InvalidTimeFormat)?;
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
pub struct WorkedMinutes(pub u16);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefaultBreakMinutes(pub u16);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultWorkInterval {
    pub start: TimeOfDay,
    pub end: TimeOfDay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkInterval {
    pub start: TimeOfDay,
    pub end: TimeOfDay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayEntry {
    pub day_id: DayId,
    pub label: DayLabel,
    pub intervals: Vec<WorkInterval>,
    pub break_minutes: BreakMinutes,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredDay {
    pub day_id: DayId,
    pub label: DayLabel,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekSheet {
    pub week_id: WeekId,
    pub week_start: WeekStartDate,
    pub entries: Vec<DayEntry>,
    pub overtime_threshold: OvertimeThresholdMinutes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayWorkSummary {
    pub day_id: DayId,
    pub label: DayLabel,
    pub worked_minutes: WorkedMinutes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekSummary {
    pub total_minutes: WorkedMinutes,
    pub overtime_minutes: WorkedMinutes,
    pub average_minutes: WorkedMinutes,
    pub longest_day: Option<DayWorkSummary>,
    pub shortest_day: Option<DayWorkSummary>,
    pub worked_days: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub overtime_threshold: OvertimeThresholdMinutes,
    pub theme: ThemePreference,
    pub default_work_interval: DefaultWorkInterval,
    pub default_break_minutes: DefaultBreakMinutes,
    pub configured_days: Vec<ConfiguredDay>,
    pub active_week_id: Option<WeekId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticSnapshot {
    pub snapshot_id: String,
    pub created_at: String,
    pub reason: String,
    pub correlation_id: String,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppMetadata {
    pub latest_migration_status: String,
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
