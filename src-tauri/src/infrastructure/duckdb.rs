use std::path::PathBuf;

use chrono::Utc;
use duckdb::{params, Connection};

use crate::{
    application::dto::{DayOfWeekStats, MonthlyStatsView, WeeklyTrendPoint},
    domain::{
        errors::StorageError,
        logic::{default_settings, minutes_to_label},
        types::{
            AppSettings, BreakMinutes, DayEntry, DayId, DayLabel, OvertimeThresholdMinutes,
            ThemePreference, TimeOfDay, WeekId, WeekSheet, WeekStartDate, WorkInterval,
        },
    },
};

fn map_storage_error<T>(result: Result<T, duckdb::Error>) -> Result<T, StorageError> {
    result.map_err(|_| StorageError::QueryFailed)
}

fn open_connection(path: &PathBuf) -> Result<Connection, StorageError> {
    Connection::open(path).map_err(|_| StorageError::StorageUnavailable)
}

fn parse_interval(start_minutes: Option<u16>, end_minutes: Option<u16>) -> Option<WorkInterval> {
    match (start_minutes, end_minutes) {
        (Some(start), Some(end)) => Some(WorkInterval {
            start: TimeOfDay(start),
            end: TimeOfDay(end),
        }),
        _ => None,
    }
}

/// Stockage DuckDB : semaines, jours et paramètres de l'application.
#[derive(Clone)]
pub struct DuckDb {
    database_path: PathBuf,
}

impl DuckDb {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    fn load_week(&self, connection: &Connection, week_id: &WeekId) -> Result<Option<WeekSheet>, StorageError> {
        let mut statement = map_storage_error(connection.prepare(
            "SELECT id, week_start, overtime_threshold_minutes FROM weeks WHERE id = ?1",
        ))?;

        let mut rows = map_storage_error(statement.query(params![week_id.0]))?;
        let Some(row) = map_storage_error(rows.next())? else {
            return Ok(None);
        };

        let stored_week_id: String = map_storage_error(row.get(0))?;
        let week_start: String = map_storage_error(row.get(1))?;
        let overtime_threshold_minutes: u16 = map_storage_error(row.get(2))?;

        let entries = self.load_entries(connection, &stored_week_id)?;

        Ok(Some(WeekSheet {
            week_id: WeekId(stored_week_id),
            week_start: WeekStartDate::parse(&week_start).map_err(|_| StorageError::SerializationFailed)?,
            entries,
            overtime_threshold: OvertimeThresholdMinutes(overtime_threshold_minutes),
        }))
    }

    fn load_entries(&self, connection: &Connection, week_id: &str) -> Result<Vec<DayEntry>, StorageError> {
        let mut statement = map_storage_error(connection.prepare(
            "SELECT day_id, label, enabled, start_minutes, end_minutes, break_minutes
             FROM day_entries
             WHERE week_id = ?1
             ORDER BY day_id ASC",
        ))?;
        let mut rows = map_storage_error(statement.query(params![week_id]))?;

        let mut entries = Vec::new();
        while let Some(day_row) = map_storage_error(rows.next())? {
            let day_id: u8 = map_storage_error(day_row.get(0))?;
            let label: String = map_storage_error(day_row.get(1))?;
            let enabled: bool = map_storage_error(day_row.get(2))?;
            let start_minutes: Option<u16> = map_storage_error(day_row.get(3))?;
            let end_minutes: Option<u16> = map_storage_error(day_row.get(4))?;
            let break_minutes: u16 = map_storage_error(day_row.get(5))?;

            // Les horaires sont préservés même si le jour est désactivé
            // pour pouvoir les restaurer si l'utilisateur réactive le jour
            entries.push(DayEntry {
                day_id: DayId(day_id),
                label: DayLabel(label),
                interval: parse_interval(start_minutes, end_minutes),
                break_minutes: BreakMinutes(break_minutes),
                enabled,
            });
        }

        Ok(entries)
    }

    pub fn migrate(&self) -> Result<(), StorageError> {
        let connection = open_connection(&self.database_path)?;

        map_storage_error(connection.execute(
            "CREATE TABLE IF NOT EXISTS weeks (
                id TEXT PRIMARY KEY,
                week_start TEXT NOT NULL,
                overtime_threshold_minutes INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        ))?;

        map_storage_error(connection.execute(
            "CREATE TABLE IF NOT EXISTS day_entries (
                week_id TEXT NOT NULL,
                day_id INTEGER NOT NULL,
                label TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                start_minutes INTEGER,
                end_minutes INTEGER,
                break_minutes INTEGER NOT NULL,
                PRIMARY KEY (week_id, day_id)
            )",
            [],
        ))?;

        map_storage_error(connection.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY,
                overtime_threshold_minutes INTEGER NOT NULL,
                theme TEXT NOT NULL,
                configured_days_json TEXT NOT NULL,
                active_week_id TEXT,
                default_start TEXT NOT NULL DEFAULT '08:00',
                default_end TEXT NOT NULL DEFAULT '18:00',
                default_break TEXT NOT NULL DEFAULT '01:00',
                updated_at TEXT NOT NULL
            )",
            [],
        ))?;

        Ok(())
    }

    pub fn get_week_by_id(&self, week_id: &WeekId) -> Result<Option<WeekSheet>, StorageError> {
        let connection = open_connection(&self.database_path)?;
        self.load_week(&connection, week_id)
    }

    pub fn get_week_by_start(&self, week_start: &WeekStartDate) -> Result<Option<WeekSheet>, StorageError> {
        let connection = open_connection(&self.database_path)?;
        let mut statement = map_storage_error(connection.prepare(
            "SELECT id, week_start, overtime_threshold_minutes FROM weeks WHERE week_start = ?1",
        ))?;
        let mut rows = map_storage_error(statement.query(params![week_start.as_string()]))?;
        let Some(row) = map_storage_error(rows.next())? else {
            return Ok(None);
        };

        let week_id: String = map_storage_error(row.get(0))?;
        let stored_week_start: String = map_storage_error(row.get(1))?;
        let overtime_threshold_minutes: u16 = map_storage_error(row.get(2))?;
        let entries = self.load_entries(&connection, &week_id)?;

        Ok(Some(WeekSheet {
            week_id: WeekId(week_id),
            week_start: WeekStartDate::parse(&stored_week_start)
                .map_err(|_| StorageError::SerializationFailed)?,
            entries,
            overtime_threshold: OvertimeThresholdMinutes(overtime_threshold_minutes),
        }))
    }

    pub fn save_week(&self, week: &WeekSheet) -> Result<(), StorageError> {
        let mut connection = open_connection(&self.database_path)?;
        let transaction = map_storage_error(connection.transaction())?;

        map_storage_error(transaction.execute(
            "INSERT INTO weeks (id, week_start, overtime_threshold_minutes, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (id) DO UPDATE SET week_start = ?2, overtime_threshold_minutes = ?3, updated_at = ?4",
            params![
                week.week_id.0,
                week.week_start.as_string(),
                week.overtime_threshold.0,
                Utc::now().to_rfc3339()
            ],
        ))?;

        map_storage_error(transaction.execute(
            "DELETE FROM day_entries WHERE week_id = ?1",
            params![week.week_id.0],
        ))?;

        for entry in &week.entries {
            let (start_minutes, end_minutes) = entry
                .interval
                .map(|interval| (Some(interval.start.0), Some(interval.end.0)))
                .unwrap_or((None, None));

            map_storage_error(transaction.execute(
                "INSERT INTO day_entries (week_id, day_id, label, enabled, start_minutes, end_minutes, break_minutes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    week.week_id.0,
                    entry.day_id.0,
                    entry.label.0,
                    entry.enabled,
                    start_minutes,
                    end_minutes,
                    entry.break_minutes.0
                ],
            ))?;
        }

        map_storage_error(transaction.commit())?;
        Ok(())
    }

    pub fn list_weeks(&self) -> Result<Vec<WeekSheet>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                w.id, w.week_start, w.overtime_threshold_minutes,
                de.day_id, de.label, de.enabled, de.start_minutes, de.end_minutes, de.break_minutes
             FROM weeks w
             LEFT JOIN day_entries de ON w.id = de.week_id
             ORDER BY w.week_start DESC, de.day_id ASC"
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;

        let mut weeks: Vec<WeekSheet> = Vec::new();
        while let Some(row) = map_storage_error(rows.next())? {
            let week_id_str: String = map_storage_error(row.get(0))?;

            if weeks.last().map(|week| week.week_id.0.as_str()) != Some(week_id_str.as_str()) {
                let week_start_str: String = map_storage_error(row.get(1))?;
                let overtime_threshold: u16 = map_storage_error(row.get(2))?;
                weeks.push(WeekSheet {
                    week_id: WeekId(week_id_str.clone()),
                    week_start: WeekStartDate::parse(&week_start_str)
                        .map_err(|_| StorageError::SerializationFailed)?,
                    entries: Vec::new(),
                    overtime_threshold: OvertimeThresholdMinutes(overtime_threshold),
                });
            }

            // LEFT JOIN can have NULL day columns for a week without entries
            if let Ok(Some(day_id)) = row.get::<_, Option<u8>>(3) {
                let label: String = map_storage_error(row.get(4))?;
                let enabled: bool = map_storage_error(row.get(5))?;
                let start_minutes: Option<u16> = map_storage_error(row.get(6))?;
                let end_minutes: Option<u16> = map_storage_error(row.get(7))?;
                let break_minutes: u16 = map_storage_error(row.get(8))?;

                weeks.last_mut().expect("week pushed above").entries.push(DayEntry {
                    day_id: DayId(day_id),
                    label: DayLabel(label),
                    interval: parse_interval(start_minutes, end_minutes),
                    break_minutes: BreakMinutes(break_minutes),
                    enabled,
                });
            }
        }

        Ok(weeks)
    }

    pub fn delete_week(&self, week_id: &WeekId) -> Result<(), StorageError> {
        let connection = open_connection(&self.database_path)?;
        map_storage_error(connection.execute(
            "DELETE FROM day_entries WHERE week_id = ?1",
            params![week_id.0],
        ))?;
        map_storage_error(connection.execute("DELETE FROM weeks WHERE id = ?1", params![week_id.0]))?;
        Ok(())
    }

    pub fn get_cumulative_balance(&self, up_to_week_start: &WeekStartDate) -> Result<i32, StorageError> {
        let connection = open_connection(&self.database_path)?;
        let mut statement = map_storage_error(connection.prepare(
            "SELECT COALESCE(SUM(
                COALESCE(week_totals.total_minutes, 0) - w.overtime_threshold_minutes
            ), 0)
            FROM weeks w
            LEFT JOIN (
                SELECT week_id,
                    SUM(CASE WHEN enabled = 1
                        AND start_minutes IS NOT NULL
                        AND end_minutes IS NOT NULL
                    THEN end_minutes - start_minutes - break_minutes
                    ELSE 0 END) as total_minutes
                FROM day_entries
                GROUP BY week_id
            ) week_totals ON w.id = week_totals.week_id
            WHERE w.week_start <= ?1",
        ))?;
        let balance: Result<i32, duckdb::Error> =
            statement.query_row(params![up_to_week_start.as_string()], |row| row.get(0));
        map_storage_error(balance)
    }

    pub fn ensure_default_settings(&self) -> Result<(), StorageError> {
        let settings = default_settings();
        let configured_days_json =
            serde_json::to_string(&settings.configured_days).map_err(|_| StorageError::SerializationFailed)?;

        let connection = open_connection(&self.database_path)?;
        map_storage_error(connection.execute(
            "INSERT INTO settings
             (id, overtime_threshold_minutes, theme, configured_days_json, active_week_id, default_start, default_end, default_break, updated_at)
             SELECT 1, ?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7
             WHERE NOT EXISTS (SELECT 1 FROM settings WHERE id = 1)",
            params![
                settings.overtime_threshold.0,
                settings.theme.to_string(),
                configured_days_json,
                settings.default_work_interval.start.to_hhmm(),
                settings.default_work_interval.end.to_hhmm(),
                TimeOfDay(settings.default_break_minutes.0).to_hhmm(),
                Utc::now().to_rfc3339()
            ],
        ))?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<AppSettings, StorageError> {
        let connection = open_connection(&self.database_path)?;
        let mut statement = map_storage_error(connection.prepare(
            "SELECT overtime_threshold_minutes, theme, configured_days_json, active_week_id, default_start, default_end, default_break
             FROM settings
             WHERE id = 1",
        ))?;
        let mut rows = map_storage_error(statement.query([]))?;
        let Some(row) = map_storage_error(rows.next())? else {
            return Err(StorageError::EntityNotFound);
        };

        let overtime_threshold_minutes: u16 = map_storage_error(row.get(0))?;
        let theme: String = map_storage_error(row.get(1))?;
        let configured_days_json: String = map_storage_error(row.get(2))?;
        let active_week_id: Option<String> = map_storage_error(row.get(3))?;
        let default_start: String = map_storage_error(row.get(4))?;
        let default_end: String = map_storage_error(row.get(5))?;
        let default_break: String = map_storage_error(row.get(6))?;
        let configured_days = serde_json::from_str(&configured_days_json)
            .map_err(|_| StorageError::SerializationFailed)?;

        Ok(AppSettings {
            overtime_threshold: OvertimeThresholdMinutes(overtime_threshold_minutes),
            theme: theme.parse().unwrap_or(ThemePreference::Dark),
            default_work_interval: WorkInterval {
                start: TimeOfDay::parse(&default_start).map_err(|_| StorageError::SerializationFailed)?,
                end: TimeOfDay::parse(&default_end).map_err(|_| StorageError::SerializationFailed)?,
            },
            default_break_minutes: BreakMinutes::parse(&default_break)
                .map_err(|_| StorageError::SerializationFailed)?,
            configured_days,
            active_week_id: active_week_id.map(WeekId),
        })
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), StorageError> {
        let connection = open_connection(&self.database_path)?;
        let configured_days_json =
            serde_json::to_string(&settings.configured_days).map_err(|_| StorageError::SerializationFailed)?;
        map_storage_error(connection.execute(
            "INSERT INTO settings
             (id, overtime_threshold_minutes, theme, configured_days_json, active_week_id, default_start, default_end, default_break, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT (id) DO UPDATE SET
               overtime_threshold_minutes = ?1, theme = ?2, configured_days_json = ?3, active_week_id = ?4,
               default_start = ?5, default_end = ?6, default_break = ?7, updated_at = ?8",
            params![
                settings.overtime_threshold.0,
                settings.theme.to_string(),
                configured_days_json,
                settings.active_week_id.as_ref().map(|week_id| week_id.0.clone()),
                settings.default_work_interval.start.to_hhmm(),
                settings.default_work_interval.end.to_hhmm(),
                TimeOfDay(settings.default_break_minutes.0).to_hhmm(),
                Utc::now().to_rfc3339()
            ],
        ))?;
        Ok(())
    }

    pub fn get_day_of_week_stats(&self) -> Result<Vec<DayOfWeekStats>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                de.day_id,
                de.label,
                COUNT(*) as entry_count,
                COALESCE(SUM(
                    CASE
                        WHEN de.enabled = 1
                            AND de.start_minutes IS NOT NULL
                            AND de.end_minutes IS NOT NULL
                        THEN de.end_minutes - de.start_minutes - de.break_minutes
                        ELSE 0
                    END
                ), 0) as total_minutes
            FROM day_entries de
            GROUP BY de.day_id, de.label
            ORDER BY de.day_id ASC",
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;
        let mut stats = Vec::new();

        let day_names = ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche"];

        while let Some(row) = map_storage_error(rows.next())? {
            let day_index: u8 = map_storage_error(row.get(0))?;
            let day_name: String = map_storage_error(row.get(1))?;
            let entry_count: i64 = map_storage_error(row.get(2))?;
            let total_minutes: i64 = map_storage_error(row.get(3))?;

            let average_minutes = if entry_count > 0 {
                (total_minutes / entry_count).max(0) as u32
            } else {
                0
            };

            stats.push(DayOfWeekStats {
                day_name: day_names.get(day_index as usize).copied().unwrap_or(day_name.as_str()).to_string(),
                average_minutes,
            });
        }

        Ok(stats)
    }

    pub fn get_weekly_trends(&self) -> Result<Vec<WeeklyTrendPoint>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                w.week_start,
                COALESCE(SUM(
                    CASE
                        WHEN de.enabled = 1
                            AND de.start_minutes IS NOT NULL
                            AND de.end_minutes IS NOT NULL
                        THEN GREATEST(0, de.end_minutes - de.start_minutes - de.break_minutes)
                        ELSE 0
                    END
                ), 0) as total_minutes
            FROM weeks w
            LEFT JOIN day_entries de ON w.id = de.week_id
            GROUP BY w.id, w.week_start
            ORDER BY w.week_start DESC
            LIMIT 12",
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;
        let mut trends = Vec::new();

        while let Some(row) = map_storage_error(rows.next())? {
            let week_start: String = map_storage_error(row.get(0))?;
            let total_minutes: i64 = map_storage_error(row.get(1))?;

            trends.push(WeeklyTrendPoint {
                week_start,
                total_minutes: total_minutes.max(0) as u32,
            });
        }

        // Inverser pour avoir l'ordre chronologique
        trends.reverse();
        Ok(trends)
    }

    pub fn get_monthly_stats(&self) -> Result<Vec<MonthlyStatsView>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                SUBSTR(w.week_start, 1, 7) as month,
                COUNT(DISTINCT w.id) as weeks_count,
                COALESCE(SUM(
                    CASE
                        WHEN de.enabled = 1
                            AND de.start_minutes IS NOT NULL
                            AND de.end_minutes IS NOT NULL
                        THEN GREATEST(0, de.end_minutes - de.start_minutes - de.break_minutes)
                        ELSE 0
                    END
                ), 0) as total_minutes
            FROM weeks w
            LEFT JOIN day_entries de ON w.id = de.week_id
            GROUP BY month
            ORDER BY month DESC",
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;
        let mut stats = Vec::new();

        while let Some(row) = map_storage_error(rows.next())? {
            let month: String = map_storage_error(row.get(0))?;
            let weeks_count: i64 = map_storage_error(row.get(1))?;
            let total_minutes: i64 = map_storage_error(row.get(2))?;

            let total_minutes = total_minutes.max(0) as u32;
            let weekly_average_minutes = if weeks_count > 0 {
                total_minutes / weeks_count as u32
            } else {
                0
            };

            stats.push(MonthlyStatsView {
                month,
                total_label: minutes_to_label(total_minutes as u16),
                weekly_average_label: minutes_to_label(weekly_average_minutes as u16),
            });
        }

        // Inverser pour avoir l'ordre chronologique
        stats.reverse();
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        domain::{
            logic::{default_entries, default_settings, summarize_week},
            types::{OvertimeThresholdMinutes, WeekSheet, WeekStartDate},
        },
        infrastructure::duckdb::DuckDb,
    };

    #[test]
    fn persists_and_loads_week() {
        let temp_dir = tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("integration.duckdb");
        let store = DuckDb::new(db_path);

        store.migrate().expect("migrations");
        store.ensure_default_settings().expect("default settings");

        let week = WeekSheet {
            week_id: crate::domain::types::WeekId::new(),
            week_start: WeekStartDate::today(),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
        };

        store.save_week(&week).expect("save");
        let loaded = store
            .get_week_by_id(&week.week_id)
            .expect("load")
            .expect("week should exist");

        assert_eq!(loaded.entries.len(), 7);
        assert_eq!(summarize_week(&loaded).expect("summary").worked_days, 5);
    }

    #[test]
    fn cumulative_balance_no_weeks() {
        let temp_dir = tempdir().expect("temp dir");
        let store = DuckDb::new(temp_dir.path().join("balance_test.duckdb"));

        store.migrate().expect("migrations");

        let balance = store
            .get_cumulative_balance(&WeekStartDate::today())
            .expect("balance query");

        assert_eq!(balance, 0);
    }

    #[test]
    fn cumulative_balance_single_week_positive() {
        let temp_dir = tempdir().expect("temp dir");
        let store = DuckDb::new(temp_dir.path().join("balance_test.duckdb"));

        store.migrate().expect("migrations");
        store.ensure_default_settings().expect("default settings");

        let week = WeekSheet {
            week_id: crate::domain::types::WeekId::new(),
            week_start: WeekStartDate::today(),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
        };

        store.save_week(&week).expect("save");

        let balance = store
            .get_cumulative_balance(&week.week_start)
            .expect("balance query");

        let summary = summarize_week(&week).expect("summary");
        assert_eq!(balance, i32::from(summary.total_minutes) - 2100);
    }

    #[test]
    fn cumulative_balance_multiple_weeks() {
        let temp_dir = tempdir().expect("temp dir");
        let store = DuckDb::new(temp_dir.path().join("balance_test.duckdb"));

        store.migrate().expect("migrations");
        store.ensure_default_settings().expect("default settings");

        let week1 = WeekSheet {
            week_id: crate::domain::types::WeekId::new(),
            week_start: WeekStartDate::today(),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
        };
        store.save_week(&week1).expect("save week1");

        let week2 = WeekSheet {
            week_id: crate::domain::types::WeekId::new(),
            week_start: WeekStartDate::parse("2024-01-15").expect("parse date"),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
        };
        store.save_week(&week2).expect("save week2");

        let balance = store
            .get_cumulative_balance(&WeekStartDate::today())
            .expect("balance query");

        let summary1 = summarize_week(&week1).expect("summary1");
        let summary2 = summarize_week(&week2).expect("summary2");
        let expected = (i32::from(summary1.total_minutes) - 2100) + (i32::from(summary2.total_minutes) - 2100);
        assert_eq!(balance, expected);
    }
}
