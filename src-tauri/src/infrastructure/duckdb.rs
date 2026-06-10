use std::path::PathBuf;

use chrono::Utc;
use duckdb::{params, Connection};

use crate::{
    application::ports::{AnalyticsRepository, DiagnosticsStore, SettingsRepository, WeekRepository},
    application::dto::{DayOfWeekStats, MonthlyStatsView, WeeklyTrendPoint},
    domain::{
        errors::StorageError,
        logic::{default_settings, minutes_to_label},
        types::{
            AppMetadata, AppSettings, BreakMinutes, DayEntry, DayId, DayLabel, DefaultBreakMinutes,
            DefaultWorkInterval, DiagnosticSnapshot, OvertimeThresholdMinutes, ThemePreference,
            TimeOfDay, WeekId, WeekSheet, WeekStartDate, WorkInterval,
        },
    },
};

fn map_storage_error<T>(result: Result<T, duckdb::Error>) -> Result<T, StorageError> {
    result.map_err(|_| StorageError::QueryFailed)
}

fn open_connection(path: &PathBuf) -> Result<Connection, StorageError> {
    Connection::open(path).map_err(|_| StorageError::StorageUnavailable)
}

#[derive(Clone)]
pub struct DuckDbWeekRepository {
    database_path: PathBuf,
}

impl DuckDbWeekRepository {
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

        let mut day_statement = map_storage_error(connection.prepare(
            "SELECT day_id, label, enabled, start_minutes, end_minutes, break_minutes
             FROM day_entries
             WHERE week_id = ?1
             ORDER BY day_id ASC",
        ))?;
        let mut day_rows = map_storage_error(day_statement.query(params![stored_week_id.clone()]))?;

        let mut entries = Vec::new();
        while let Some(day_row) = map_storage_error(day_rows.next())? {
            let day_id: u8 = map_storage_error(day_row.get(0))?;
            let label: String = map_storage_error(day_row.get(1))?;
            let enabled: bool = map_storage_error(day_row.get(2))?;
            let start_minutes: Option<u16> = map_storage_error(day_row.get(3))?;
            let end_minutes: Option<u16> = map_storage_error(day_row.get(4))?;
            let break_minutes: u16 = map_storage_error(day_row.get(5))?;

            // Les horaires sont préservés même si le jour est désactivé
            // pour pouvoir les restaurer si l'utilisateur réactive le jour
            let intervals = match (start_minutes, end_minutes) {
                (Some(start), Some(end)) => vec![WorkInterval {
                    start: TimeOfDay(start),
                    end: TimeOfDay(end),
                }],
                _ => Vec::new(),
            };

            entries.push(DayEntry {
                day_id: DayId(day_id),
                label: DayLabel(label),
                intervals,
                break_minutes: BreakMinutes(break_minutes),
                enabled,
            });
        }

        Ok(Some(WeekSheet {
            week_id: WeekId(stored_week_id),
            week_start: WeekStartDate::parse(&week_start).map_err(|_| StorageError::SerializationFailed)?,
            entries,
            overtime_threshold: OvertimeThresholdMinutes(overtime_threshold_minutes),
        }))
    }
}

impl WeekRepository for DuckDbWeekRepository {
    fn migrate(&self) -> Result<(), StorageError> {
        let connection = open_connection(&self.database_path)?;

        // Create weeks table
        map_storage_error(connection.execute(
            "CREATE TABLE IF NOT EXISTS weeks (
                id TEXT PRIMARY KEY,
                week_start TEXT NOT NULL,
                overtime_threshold_minutes INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        ))?;

        // Create day_entries table
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

        // Create settings table
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

        // Create diagnostic_snapshots table
        map_storage_error(connection.execute(
            "CREATE TABLE IF NOT EXISTS diagnostic_snapshots (
                snapshot_id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                reason TEXT NOT NULL,
                correlation_id TEXT NOT NULL,
                payload_json TEXT NOT NULL
            )",
            [],
        ))?;

        // Create app_metadata table
        map_storage_error(connection.execute(
            "CREATE TABLE IF NOT EXISTS app_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        ))?;

        // Initialize metadata
        map_storage_error(connection.execute(
            "INSERT INTO app_metadata (key, value)
             SELECT 'latest_migration_status', 'success'
             WHERE NOT EXISTS (SELECT 1 FROM app_metadata WHERE key = 'latest_migration_status')",
            [],
        ))?;

        Ok(())
    }

    fn get_week_by_id(&self, week_id: &WeekId) -> Result<Option<WeekSheet>, StorageError> {
        let connection = open_connection(&self.database_path)?;
        self.load_week(&connection, week_id)
    }

    fn get_week_by_start(&self, week_start: &WeekStartDate) -> Result<Option<WeekSheet>, StorageError> {
        let connection = open_connection(&self.database_path)?;
        let mut statement = map_storage_error(connection.prepare(
            "SELECT id FROM weeks WHERE week_start = ?1",
        ))?;
        let mut rows = map_storage_error(statement.query(params![week_start.as_string()]))?;
        let Some(row) = map_storage_error(rows.next())? else {
            return Ok(None);
        };
        let week_id: String = map_storage_error(row.get(0))?;
        self.load_week(&connection, &WeekId(week_id))
    }

    fn save_week(&self, week: &WeekSheet) -> Result<(), StorageError> {
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
                .intervals
                .first()
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

    fn list_weeks(&self) -> Result<Vec<WeekSheet>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        // Single JOIN query instead of N+1
        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                w.id, w.week_start, w.overtime_threshold_minutes,
                de.day_id, de.label, de.enabled, de.start_minutes, de.end_minutes, de.break_minutes
             FROM weeks w
             LEFT JOIN day_entries de ON w.id = de.week_id
             ORDER BY w.week_start DESC, de.day_id ASC"
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;

        // Group entries by week_id
        use std::collections::HashMap;
        let mut weeks_map: HashMap<String, (WeekId, WeekStartDate, OvertimeThresholdMinutes, Vec<DayEntry>)> = HashMap::new();

        while let Some(row) = map_storage_error(rows.next())? {
            let week_id_str: String = map_storage_error(row.get(0))?;
            let week_start_str: String = map_storage_error(row.get(1))?;
            let overtime_threshold: u16 = map_storage_error(row.get(2))?;

            // Get or create week entry in map
            if !weeks_map.contains_key(&week_id_str) {
                let week_start = WeekStartDate::parse(&week_start_str)
                    .map_err(|_| StorageError::SerializationFailed)?;
                weeks_map.insert(
                    week_id_str.clone(),
                    (WeekId(week_id_str.clone()), week_start, OvertimeThresholdMinutes(overtime_threshold), Vec::new())
                );
            }

            // Add entry if present (LEFT JOIN can have NULL for day columns)
            let day_id: Option<u8> = row.get(3).ok();
            if let Some(day_id) = day_id {
                let label: String = map_storage_error(row.get(4))?;
                let enabled: bool = map_storage_error(row.get(5))?;
                let start_minutes: Option<u16> = map_storage_error(row.get(6))?;
                let end_minutes: Option<u16> = map_storage_error(row.get(7))?;
                let break_minutes: u16 = map_storage_error(row.get(8))?;

                let intervals = match (start_minutes, end_minutes) {
                    (Some(start), Some(end)) => vec![WorkInterval {
                        start: TimeOfDay(start),
                        end: TimeOfDay(end),
                    }],
                    _ => Vec::new(),
                };

                let entry = DayEntry {
                    day_id: DayId(day_id),
                    label: DayLabel(label),
                    intervals,
                    break_minutes: BreakMinutes(break_minutes),
                    enabled,
                };

                weeks_map.get_mut(&week_id_str).unwrap().3.push(entry);
            }
        }

        // Convert map to Vec<WeekSheet>, sorted by week_start DESC
        let mut weeks: Vec<WeekSheet> = weeks_map.into_values()
            .map(|(week_id, week_start, overtime_threshold, entries)| WeekSheet {
                week_id,
                week_start,
                entries,
                overtime_threshold,
            })
            .collect();

        // Sort by week_start descending (since HashMap doesn preserve order)
        weeks.sort_by(|a, b| b.week_start.as_string().cmp(&a.week_start.as_string()));

        Ok(weeks)
    }

    fn delete_week(&self, week_id: &WeekId) -> Result<(), StorageError> {
        let connection = open_connection(&self.database_path)?;
        map_storage_error(connection.execute(
            "DELETE FROM day_entries WHERE week_id = ?1",
            params![week_id.0],
        ))?;
        map_storage_error(connection.execute("DELETE FROM weeks WHERE id = ?1", params![week_id.0]))?;
        Ok(())
    }

    fn metadata(&self) -> Result<AppMetadata, StorageError> {
        let connection = open_connection(&self.database_path)?;
        let mut statement = map_storage_error(connection.prepare(
            "SELECT value FROM app_metadata WHERE key = ?1",
        ))?;
        let value: Result<String, duckdb::Error> =
            statement.query_row(params!["latest_migration_status"], |row| row.get(0));
        Ok(AppMetadata {
            latest_migration_status: value.unwrap_or_else(|_| "unknown".to_string()),
        })
    }

    fn ping(&self) -> Result<(), StorageError> {
        let connection = open_connection(&self.database_path)?;
        map_storage_error(connection.execute("SELECT 1", []))?;
        Ok(())
    }
}

impl AnalyticsRepository for DuckDbWeekRepository {
    fn get_day_of_week_stats(&self) -> Result<Vec<DayOfWeekStats>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        // Requête pour calculer les statistiques par jour de la semaine
        // day_id correspond à l'index du jour (0=Lundi, 1=Mardi, etc.)
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

        // Noms des jours de la semaine en français
        let day_names = ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche"];

        while let Some(row) = map_storage_error(rows.next())? {
            let day_index: u8 = map_storage_error(row.get(0))?;
            let day_name: String = map_storage_error(row.get(1))?;
            let entry_count: i64 = map_storage_error(row.get(2))?;
            let total_minutes: i64 = map_storage_error(row.get(3))?;

            let entry_count = entry_count.max(0) as u32;
            let total_minutes = total_minutes.max(0) as u32;
            let average_minutes = if entry_count > 0 {
                total_minutes / entry_count
            } else {
                0
            };

            stats.push(DayOfWeekStats {
                day_index,
                day_name: day_names.get(day_index as usize).copied().unwrap_or(day_name.as_str()).to_string(),
                entry_count,
                average_minutes,
                average_label: minutes_to_label(average_minutes as u16),
                total_minutes,
                total_label: minutes_to_label(total_minutes as u16),
            });
        }

        Ok(stats)
    }

    fn get_weekly_trends(&self) -> Result<Vec<WeeklyTrendPoint>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        // Requête pour obtenir les tendances des 12 dernières semaines
        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                w.week_start,
                w.overtime_threshold_minutes,
                COALESCE(SUM(
                    CASE
                        WHEN de.enabled = 1
                            AND de.start_minutes IS NOT NULL
                            AND de.end_minutes IS NOT NULL
                        THEN GREATEST(0, de.end_minutes - de.start_minutes - de.break_minutes)
                        ELSE 0
                    END
                ), 0) as total_minutes,
                COUNT(CASE
                    WHEN de.enabled = 1
                        AND de.start_minutes IS NOT NULL
                        AND de.end_minutes IS NOT NULL
                        AND (de.end_minutes - de.start_minutes - de.break_minutes) > 0
                    THEN 1
                END) as worked_days
            FROM weeks w
            LEFT JOIN day_entries de ON w.id = de.week_id
            GROUP BY w.id, w.week_start, w.overtime_threshold_minutes
            ORDER BY w.week_start DESC
            LIMIT 12",
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;
        let mut trends = Vec::new();

        while let Some(row) = map_storage_error(rows.next())? {
            let week_start: String = map_storage_error(row.get(0))?;
            let overtime_threshold: i64 = map_storage_error(row.get(1))?;
            let total_minutes: i64 = map_storage_error(row.get(2))?;
            let worked_days: i64 = map_storage_error(row.get(3))?;

            let total_minutes = total_minutes.max(0) as u32;
            let overtime_threshold = overtime_threshold.max(0) as u32;
            let overtime_minutes = total_minutes.saturating_sub(overtime_threshold);
            let worked_days = worked_days.max(0) as u8;

            trends.push(WeeklyTrendPoint {
                week_start,
                total_minutes,
                total_label: minutes_to_label(total_minutes as u16),
                worked_days,
                overtime_minutes,
                overtime_label: minutes_to_label(overtime_minutes as u16),
            });
        }

        // Inverser pour avoir l'ordre chronologique
        trends.reverse();
        Ok(trends)
    }

    fn get_monthly_stats(&self) -> Result<Vec<MonthlyStatsView>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        // Requête pour agréger les statistiques par mois
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

            let weeks_count = weeks_count.max(0) as u32;
            let total_minutes = total_minutes.max(0) as u32;
            let weekly_average_minutes = if weeks_count > 0 {
                total_minutes / weeks_count
            } else {
                0
            };

            stats.push(MonthlyStatsView {
                month,
                weeks_count,
                total_minutes,
                total_label: minutes_to_label(total_minutes as u16),
                weekly_average_minutes,
                weekly_average_label: minutes_to_label(weekly_average_minutes as u16),
            });
        }

        // Inverser pour avoir l'ordre chronologique
        stats.reverse();
        Ok(stats)
    }
}

#[derive(Clone)]
pub struct DuckDbSettingsRepository {
    database_path: PathBuf,
}

impl DuckDbSettingsRepository {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

impl SettingsRepository for DuckDbSettingsRepository {
    fn ensure_default_settings(&self) -> Result<(), StorageError> {
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
                theme_to_string(settings.theme),
                configured_days_json,
                settings.default_work_interval.start.to_hhmm(),
                settings.default_work_interval.end.to_hhmm(),
                TimeOfDay(settings.default_break_minutes.0).to_hhmm(),
                Utc::now().to_rfc3339()
            ],
        ))?;
        Ok(())
    }

    fn load_settings(&self) -> Result<AppSettings, StorageError> {
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
            theme: match theme.as_str() {
                "light" => ThemePreference::Light,
                _ => ThemePreference::Dark,
            },
            default_work_interval: DefaultWorkInterval {
                start: TimeOfDay::parse(&default_start).map_err(|_| StorageError::SerializationFailed)?,
                end: TimeOfDay::parse(&default_end).map_err(|_| StorageError::SerializationFailed)?,
            },
            default_break_minutes: DefaultBreakMinutes(BreakMinutes::parse(&default_break).map_err(|_| StorageError::SerializationFailed)?.0),
            configured_days,
            active_week_id: active_week_id.map(WeekId),
        })
    }

    fn save_settings(&self, settings: &AppSettings) -> Result<(), StorageError> {
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
                theme_to_string(settings.theme),
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
}

#[derive(Clone)]
pub struct DuckDbDiagnosticsStore {
    database_path: PathBuf,
}

impl DuckDbDiagnosticsStore {
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }
}

impl DiagnosticsStore for DuckDbDiagnosticsStore {
    fn save_snapshot(&self, snapshot: &DiagnosticSnapshot) -> Result<(), StorageError> {
        let connection = open_connection(&self.database_path)?;
        map_storage_error(connection.execute(
            "INSERT INTO diagnostic_snapshots
             (snapshot_id, created_at, reason, correlation_id, payload_json)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (snapshot_id) DO UPDATE SET
               created_at = ?2, reason = ?3, correlation_id = ?4, payload_json = ?5",
            params![
                snapshot.snapshot_id,
                snapshot.created_at,
                snapshot.reason,
                snapshot.correlation_id,
                snapshot.payload_json
            ],
        ))?;
        Ok(())
    }

    fn latest_snapshot_id(&self) -> Result<Option<String>, StorageError> {
        let connection = open_connection(&self.database_path)?;
        let mut statement = map_storage_error(connection.prepare(
            "SELECT snapshot_id FROM diagnostic_snapshots ORDER BY created_at DESC LIMIT 1",
        ))?;
        let mut rows = map_storage_error(statement.query([]))?;
        if let Some(row) = map_storage_error(rows.next())? {
            let snapshot_id: String = map_storage_error(row.get(0))?;
            Ok(Some(snapshot_id))
        } else {
            Ok(None)
        }
    }
}

fn theme_to_string(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        application::ports::{SettingsRepository, WeekRepository},
        domain::{
            logic::{default_entries, summarize_week},
            types::{default_configured_days, OvertimeThresholdMinutes, WeekId, WeekSheet, WeekStartDate},
        },
        infrastructure::duckdb::{DuckDbSettingsRepository, DuckDbWeekRepository},
    };

    #[test]
    fn persists_and_loads_week() {
        let temp_dir = tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("integration.duckdb");
        let week_repository = DuckDbWeekRepository::new(db_path.clone());
        let settings_repository = DuckDbSettingsRepository::new(db_path);

        week_repository.migrate().expect("migrations");
        settings_repository.ensure_default_settings().expect("default settings");

        let week = WeekSheet {
            week_id: WeekId::new(),
            week_start: WeekStartDate::today(),
            entries: default_entries(&default_configured_days()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
        };

        week_repository.save_week(&week).expect("save");
        let loaded = week_repository
            .get_week_by_id(&week.week_id)
            .expect("load")
            .expect("week should exist");

        assert_eq!(loaded.entries.len(), 7);
        assert_eq!(summarize_week(&loaded).expect("summary").worked_days, 5);
    }
}
