use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::Utc;
use duckdb::{params, Connection};

use crate::{
    application::dto::{DayOfWeekStats, MonthlyStatsView, WeekAnalyticsPoint, WeeklyTrendPoint},
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

/// Guard de la connexion partagée : se déréférence en `&Connection` /
/// `&mut Connection` pour que les méthodes du store ne changent pas de forme.
struct SharedConnection<'a>(MutexGuard<'a, Option<Connection>>);

impl Deref for SharedConnection<'_> {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        self.0.as_ref().expect("connexion initialisée")
    }
}

impl DerefMut for SharedConnection<'_> {
    fn deref_mut(&mut self) -> &mut Connection {
        self.0.as_mut().expect("connexion initialisée")
    }
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
///
/// Une connexion unique est ouverte paresseusement puis réutilisée par toutes
/// les opérations : ouvrir un fichier DuckDB coûte cher (buffer pool,
/// métadonnées, checksum) et dominait la latence de l'autosave (~8-10
/// ouvertures par save débounced). Le `Mutex` sérialise les accès.
#[derive(Clone)]
pub struct DuckDb {
    pub database_path: PathBuf,
    connection: Arc<Mutex<Option<Connection>>>,
    open_count: Arc<AtomicUsize>,
}

impl DuckDb {
    pub fn new(database_path: PathBuf) -> Self {
        Self {
            database_path,
            connection: Arc::new(Mutex::new(None)),
            open_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Connexion partagée, ouverte au premier besoin.
    fn shared(&self) -> Result<SharedConnection<'_>, StorageError> {
        let mut guard = self
            .connection
            .lock()
            .map_err(|_| StorageError::StorageUnavailable)?;
        if guard.is_none() {
            *guard = Some(open_connection(&self.database_path)?);
            self.open_count.fetch_add(1, Ordering::SeqCst);
        }
        Ok(SharedConnection(guard))
    }

    /// Nombre d'ouvertures de fichier effectuées par ce store (tests).
    #[cfg(test)]
    pub(crate) fn connection_open_count(&self) -> usize {
        self.open_count.load(Ordering::SeqCst)
    }

    /// Accès brut à la connexion partagée (corruption volontaire en tests).
    #[cfg(test)]
    pub(crate) fn raw_connection(&self) -> MutexGuard<'_, Option<Connection>> {
        self.connection.lock().expect("verrou de la connexion")
    }

    fn load_week(&self, connection: &Connection, week_id: &WeekId) -> Result<Option<WeekSheet>, StorageError> {
        let mut statement = map_storage_error(connection.prepare(
            "SELECT id, week_start, overtime_threshold_minutes, travel_deduction_minutes, updated_at FROM weeks WHERE id = ?1",
        ))?;

        let mut rows = map_storage_error(statement.query(params![week_id.0]))?;
        let Some(row) = map_storage_error(rows.next())? else {
            return Ok(None);
        };

        let stored_week_id: String = map_storage_error(row.get(0))?;
        let week_start: String = map_storage_error(row.get(1))?;
        let overtime_threshold_minutes: u16 = map_storage_error(row.get(2))?;
        let travel_deduction_minutes: u16 = map_storage_error(row.get(3))?;
        let updated_at: String = map_storage_error(row.get(4))?;

        let entries = self.load_entries(connection, &stored_week_id)?;

        Ok(Some(WeekSheet {
            week_id: Some(WeekId(stored_week_id)),
            week_start: WeekStartDate::parse(&week_start).map_err(|_| StorageError::SerializationFailed)?,
            entries,
            overtime_threshold: OvertimeThresholdMinutes(overtime_threshold_minutes),
            travel_deduction_minutes: crate::domain::types::TravelDeductionMinutes(travel_deduction_minutes),
            updated_at,
        }))
    }

    fn load_entries(&self, connection: &Connection, week_id: &str) -> Result<Vec<DayEntry>, StorageError> {
        let mut statement = map_storage_error(connection.prepare(
            "SELECT day_id, label, enabled, start_minutes, end_minutes, break_minutes, has_departure_deduction, has_return_deduction
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
            let has_departure_deduction: bool = map_storage_error(day_row.get::<_, Option<u8>>(6))?.unwrap_or(0) == 1;
            let has_return_deduction: bool = map_storage_error(day_row.get::<_, Option<u8>>(7))?.unwrap_or(0) == 1;

            // Les horaires sont préservés même si le jour est désactivé
            // pour pouvoir les restaurer si l'utilisateur réactive le jour
            entries.push(DayEntry {
                day_id: DayId(day_id),
                label: DayLabel(label),
                interval: parse_interval(start_minutes, end_minutes),
                break_minutes: BreakMinutes(break_minutes),
                enabled,
                has_departure_deduction,
                has_return_deduction,
            });
        }

        Ok(entries)
    }

    pub fn migrate(&self) -> Result<(), StorageError> {
        let connection = self.shared()?;

        map_storage_error(connection.execute(
            "CREATE TABLE IF NOT EXISTS weeks (
                id TEXT PRIMARY KEY,
                week_start TEXT NOT NULL,
                overtime_threshold_minutes INTEGER NOT NULL,
                travel_deduction_minutes INTEGER NOT NULL DEFAULT 30,
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
                has_departure_deduction INTEGER NOT NULL DEFAULT 0,
                has_return_deduction INTEGER NOT NULL DEFAULT 0,
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
                enable_travel_deduction INTEGER NOT NULL DEFAULT 1,
                travel_deduction_minutes INTEGER NOT NULL DEFAULT 30,
                updated_at TEXT NOT NULL
            )",
            [],
        ))?;

        // Migration: ajouter les colonnes déplacement aux tables existantes
        // Utilise des transactions séparées et ignore les erreurs silencieusement
        let _ = connection.execute(
            "ALTER TABLE day_entries ADD COLUMN IF NOT EXISTS has_departure_deduction INTEGER DEFAULT 0",
            [],
        );
        let _ = connection.execute(
            "ALTER TABLE day_entries ADD COLUMN IF NOT EXISTS has_return_deduction INTEGER DEFAULT 0",
            [],
        );

        // Migration: ajouter la colonne travel_deduction_minutes à weeks
        let _ = connection.execute(
            "ALTER TABLE weeks ADD COLUMN IF NOT EXISTS travel_deduction_minutes INTEGER DEFAULT 30",
            [],
        );

        // Migration: ajouter les colonnes de configuration trajet aux settings
        let _ = connection.execute(
            "ALTER TABLE settings ADD COLUMN IF NOT EXISTS enable_travel_deduction INTEGER DEFAULT 1",
            [],
        );
        let _ = connection.execute(
            "ALTER TABLE settings ADD COLUMN IF NOT EXISTS travel_deduction_minutes INTEGER DEFAULT 30",
            [],
        );

        // Dédoublonnage historique : garde la version la plus récente par semaine
        map_storage_error(connection.execute(
            "DELETE FROM day_entries WHERE week_id IN (
                SELECT id FROM (
                    SELECT id, ROW_NUMBER() OVER (
                        PARTITION BY week_start ORDER BY updated_at DESC
                    ) AS rn
                    FROM weeks
                ) WHERE rn > 1
            )",
            [],
        ))?;
        map_storage_error(connection.execute(
            "DELETE FROM weeks WHERE id IN (
                SELECT id FROM (
                    SELECT id, ROW_NUMBER() OVER (
                        PARTITION BY week_start ORDER BY updated_at DESC
                    ) AS rn
                    FROM weeks
                ) WHERE rn > 1
            )",
            [],
        ))?;

        // Une semaine = une date de début : sans cette contrainte, le solde
        // cumulé peut compter deux fois les mêmes heures.
        map_storage_error(connection.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS uq_weeks_week_start ON weeks(week_start)",
            [],
        ))?;

        Ok(())
    }

    pub fn get_week_by_id(&self, week_id: &WeekId) -> Result<Option<WeekSheet>, StorageError> {
        let connection = self.shared()?;
        self.load_week(&connection, week_id)
    }

    pub fn get_week_by_start(&self, week_start: &WeekStartDate) -> Result<Option<WeekSheet>, StorageError> {
        let connection = self.shared()?;
        let mut statement = map_storage_error(connection.prepare(
            "SELECT id, week_start, overtime_threshold_minutes, travel_deduction_minutes, updated_at FROM weeks WHERE week_start = ?1",
        ))?;
        let mut rows = map_storage_error(statement.query(params![week_start.as_string()]))?;
        let Some(row) = map_storage_error(rows.next())? else {
            return Ok(None);
        };

        let week_id: String = map_storage_error(row.get(0))?;
        let stored_week_start: String = map_storage_error(row.get(1))?;
        let overtime_threshold_minutes: u16 = map_storage_error(row.get(2))?;
        let travel_deduction_minutes: u16 = map_storage_error(row.get(3))?;
        let updated_at: String = map_storage_error(row.get(4))?;
        let entries = self.load_entries(&connection, &week_id)?;

        Ok(Some(WeekSheet {
            week_id: Some(WeekId(week_id)),
            week_start: WeekStartDate::parse(&stored_week_start)
                .map_err(|_| StorageError::SerializationFailed)?,
            entries,
            overtime_threshold: OvertimeThresholdMinutes(overtime_threshold_minutes),
            travel_deduction_minutes: crate::domain::types::TravelDeductionMinutes(travel_deduction_minutes),
            updated_at,
        }))
    }

    pub fn save_week(&self, week: &WeekSheet) -> Result<(), StorageError> {
        let mut connection = self.shared()?;
        let transaction = map_storage_error(connection.transaction())?;

        map_storage_error(transaction.execute(
            "INSERT INTO weeks (id, week_start, overtime_threshold_minutes, travel_deduction_minutes, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (id) DO UPDATE SET week_start = ?2, overtime_threshold_minutes = ?3, travel_deduction_minutes = ?4, updated_at = ?5",
            params![
                week.week_id.as_ref().expect("week_id must be set before save").0,
                week.week_start.as_string(),
                week.overtime_threshold.0,
                week.travel_deduction_minutes.0,
                Utc::now().to_rfc3339()
            ],
        ))?;

        map_storage_error(transaction.execute(
            "DELETE FROM day_entries WHERE week_id = ?1",
            params![week.week_id.as_ref().expect("week_id must be set before save").0],
        ))?;

        for entry in &week.entries {
            let (start_minutes, end_minutes) = entry
                .interval
                .map(|interval| (Some(interval.start.0), Some(interval.end.0)))
                .unwrap_or((None, None));

            map_storage_error(transaction.execute(
                "INSERT INTO day_entries (week_id, day_id, label, enabled, start_minutes, end_minutes, break_minutes, has_departure_deduction, has_return_deduction)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    week.week_id.as_ref().expect("week_id must be set before save").0,
                    entry.day_id.0,
                    entry.label.0,
                    entry.enabled,
                    start_minutes,
                    end_minutes,
                    entry.break_minutes.0,
                    entry.has_departure_deduction,
                    entry.has_return_deduction
                ],
            ))?;
        }

        map_storage_error(transaction.commit())?;
        Ok(())
    }

    pub fn list_weeks(&self) -> Result<Vec<WeekSheet>, StorageError> {
        let connection = self.shared()?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                w.id, w.week_start, w.overtime_threshold_minutes, w.travel_deduction_minutes, w.updated_at,
                de.day_id, de.label, de.enabled, de.start_minutes, de.end_minutes, de.break_minutes, de.has_departure_deduction, de.has_return_deduction
             FROM weeks w
             LEFT JOIN day_entries de ON w.id = de.week_id
             ORDER BY w.week_start DESC, de.day_id ASC"
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;

        let mut weeks: Vec<WeekSheet> = Vec::new();
        while let Some(row) = map_storage_error(rows.next())? {
            let week_id_str: String = map_storage_error(row.get(0))?;

            if weeks.last().map(|week| week.week_id.as_ref().expect("persisted week must have id").0.as_str()) != Some(week_id_str.as_str()) {
                let week_start_str: String = map_storage_error(row.get(1))?;
                let overtime_threshold: u16 = map_storage_error(row.get(2))?;
                let travel_deduction_minutes: u16 = map_storage_error(row.get(3))?;
                let updated_at: String = map_storage_error(row.get(4))?;
                weeks.push(WeekSheet {
                    week_id: Some(WeekId(week_id_str.clone())),
                    week_start: WeekStartDate::parse(&week_start_str)
                        .map_err(|_| StorageError::SerializationFailed)?,
                    entries: Vec::new(),
                    overtime_threshold: OvertimeThresholdMinutes(overtime_threshold),
                    travel_deduction_minutes: crate::domain::types::TravelDeductionMinutes(travel_deduction_minutes),
                    updated_at,
                });
            }

            // LEFT JOIN can have NULL day columns for a week without entries
            if let Ok(Some(day_id)) = row.get::<_, Option<u8>>(5) {
                let label: String = map_storage_error(row.get(6))?;
                let enabled: bool = map_storage_error(row.get(7))?;
                let start_minutes: Option<u16> = map_storage_error(row.get(8))?;
                let end_minutes: Option<u16> = map_storage_error(row.get(9))?;
                let break_minutes: u16 = map_storage_error(row.get(10))?;
                let has_departure_deduction: bool = map_storage_error(row.get::<_, Option<u8>>(11))?.unwrap_or(0) == 1;
                let has_return_deduction: bool = map_storage_error(row.get::<_, Option<u8>>(12))?.unwrap_or(0) == 1;

                weeks.last_mut().expect("week pushed above").entries.push(DayEntry {
                    day_id: DayId(day_id),
                    label: DayLabel(label),
                    interval: parse_interval(start_minutes, end_minutes),
                    break_minutes: BreakMinutes(break_minutes),
                    enabled,
                    has_departure_deduction,
                    has_return_deduction,
                });
            }
        }

        Ok(weeks)
    }

    pub fn delete_week(&self, week_id: &WeekId) -> Result<(), StorageError> {
        let mut connection = self.shared()?;
        let transaction = connection.transaction().map_err(|_| StorageError::QueryFailed)?;

        transaction
            .execute("DELETE FROM day_entries WHERE week_id = ?1", params![week_id.0])
            .map_err(|_| StorageError::QueryFailed)?;
        transaction
            .execute("DELETE FROM weeks WHERE id = ?1", params![week_id.0])
            .map_err(|_| StorageError::QueryFailed)?;

        transaction.commit().map_err(|_| StorageError::QueryFailed)?;
        Ok(())
    }

    pub fn get_cumulative_balance(&self, up_to_week_start: &WeekStartDate) -> Result<i32, StorageError> {
        // Agrégat SQL : ne désérialise plus tout l'historique en WeekSheet
        // (l'ancienne version passait par list_weeks() + summarize_week en
        // Rust, coût O(historique) à chaque save). La sémantique fail-closed
        // de validate_day est reproduite en deux temps :
        // 1. détection des lignes invalides sur les semaines <= up_to
        //    (label vide — même jour désactivé ; activé sans horaires ;
        //    fin <= début ; pause >= durée) -> erreur propre, pas de mensonge ;
        // 2. SUM des deltas hebdo uniquement si aucune ligne n'est invalide.
        let connection = self.shared()?;

        let mut validation = map_storage_error(connection.prepare(
            "SELECT 1
             FROM day_entries de
             JOIN weeks w ON w.id = de.week_id
             WHERE w.week_start <= ?1
               AND (
                     trim(de.label) = ''
                     OR (de.enabled = 1
                         AND (de.start_minutes IS NULL
                              OR de.end_minutes IS NULL
                              OR de.end_minutes <= de.start_minutes
                              OR de.break_minutes >= de.end_minutes - de.start_minutes))
                   )
             LIMIT 1",
        ))?;
        let mut validation_rows = map_storage_error(validation.query(params![up_to_week_start.as_string()]))?;
        if map_storage_error(validation_rows.next())?.is_some() {
            return Err(StorageError::SerializationFailed);
        }
        drop(validation_rows);
        drop(validation);

        let mut statement = map_storage_error(connection.prepare(
            "SELECT COALESCE(SUM(week_total - overtime_threshold_minutes), 0)
             FROM (
                 SELECT w.overtime_threshold_minutes,
                     SUM(CASE
                         WHEN de.enabled = 1
                              AND de.start_minutes IS NOT NULL
                              AND de.end_minutes IS NOT NULL
                         THEN GREATEST(0,
                              de.end_minutes - de.start_minutes - de.break_minutes
                              - CASE WHEN COALESCE(de.has_departure_deduction, 0) = 1 THEN 30 ELSE 0 END
                              - CASE WHEN COALESCE(de.has_return_deduction, 0) = 1 THEN 30 ELSE 0 END)
                         ELSE 0
                     END) AS week_total
                 FROM weeks w
                 LEFT JOIN day_entries de ON w.id = de.week_id
                 WHERE w.week_start <= ?1
                 GROUP BY w.id, w.overtime_threshold_minutes
             ) per_week",
        ))?;
        let mut rows = map_storage_error(statement.query(params![up_to_week_start.as_string()]))?;
        let Some(row) = map_storage_error(rows.next())? else {
            return Ok(0);
        };
        let balance: i64 = map_storage_error(row.get(0))?;
        Ok(balance.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
    }

    pub fn ensure_default_settings(&self) -> Result<(), StorageError> {
        let settings = default_settings();
        let configured_days_json =
            serde_json::to_string(&settings.configured_days).map_err(|_| StorageError::SerializationFailed)?;

        let connection = self.shared()?;
        map_storage_error(connection.execute(
            "INSERT INTO settings
             (id, overtime_threshold_minutes, theme, configured_days_json, active_week_id, default_start, default_end, default_break, enable_travel_deduction, travel_deduction_minutes, updated_at)
             SELECT 1, ?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9
             WHERE NOT EXISTS (SELECT 1 FROM settings WHERE id = 1)",
            params![
                settings.overtime_threshold.0,
                settings.theme.to_string(),
                configured_days_json,
                settings.default_work_interval.start.to_hhmm(),
                settings.default_work_interval.end.to_hhmm(),
                TimeOfDay(settings.default_break_minutes.0).to_hhmm(),
                settings.enable_travel_deduction,
                settings.travel_deduction_minutes.0,
                Utc::now().to_rfc3339()
            ],
        ))?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<AppSettings, StorageError> {
        let connection = self.shared()?;
        let mut statement = map_storage_error(connection.prepare(
            "SELECT overtime_threshold_minutes, theme, configured_days_json, active_week_id, default_start, default_end, default_break, enable_travel_deduction, travel_deduction_minutes
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
        let enable_travel_deduction: bool = map_storage_error(row.get::<_, Option<u8>>(7))?.unwrap_or(1) == 1;
        let travel_deduction_minutes: u16 = map_storage_error(row.get::<_, Option<u16>>(8))?.unwrap_or(30);
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
            enable_travel_deduction,
            travel_deduction_minutes: crate::domain::types::TravelDeductionMinutes(travel_deduction_minutes),
        })
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), StorageError> {
        let connection = self.shared()?;
        let configured_days_json =
            serde_json::to_string(&settings.configured_days).map_err(|_| StorageError::SerializationFailed)?;
        map_storage_error(connection.execute(
            "INSERT INTO settings
             (id, overtime_threshold_minutes, theme, configured_days_json, active_week_id, default_start, default_end, default_break, enable_travel_deduction, travel_deduction_minutes, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT (id) DO UPDATE SET
               overtime_threshold_minutes = ?1, theme = ?2, configured_days_json = ?3, active_week_id = ?4,
               default_start = ?5, default_end = ?6, default_break = ?7, enable_travel_deduction = ?8, travel_deduction_minutes = ?9, updated_at = ?10",
            params![
                settings.overtime_threshold.0,
                settings.theme.to_string(),
                configured_days_json,
                settings.active_week_id.as_ref().map(|week_id| week_id.0.clone()),
                settings.default_work_interval.start.to_hhmm(),
                settings.default_work_interval.end.to_hhmm(),
                TimeOfDay(settings.default_break_minutes.0).to_hhmm(),
                settings.enable_travel_deduction,
                settings.travel_deduction_minutes.0,
                Utc::now().to_rfc3339()
            ],
        ))?;
        Ok(())
    }

    pub fn get_day_of_week_stats(&self) -> Result<Vec<DayOfWeekStats>, StorageError> {
        let connection = self.shared()?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                de.day_id,
                COUNT(CASE WHEN de.enabled = 1
                    AND de.start_minutes IS NOT NULL
                    AND de.end_minutes IS NOT NULL
                    AND de.end_minutes > de.start_minutes
                THEN 1 END) as worked_days,
                COALESCE(SUM(
                    CASE
                        WHEN de.enabled = 1
                            AND de.start_minutes IS NOT NULL
                            AND de.end_minutes IS NOT NULL
                            AND de.end_minutes > de.start_minutes
                        THEN GREATEST(0,
                            de.end_minutes - de.start_minutes - COALESCE(de.break_minutes, 0)
                            - COALESCE(w.travel_deduction_minutes, 30) * COALESCE(de.has_departure_deduction, 0)
                            - COALESCE(w.travel_deduction_minutes, 30) * COALESCE(de.has_return_deduction, 0)
                        )
                        ELSE 0
                    END
                ), 0) as total_minutes
            FROM day_entries de
            LEFT JOIN weeks w ON de.week_id = w.id
            GROUP BY de.day_id
            ORDER BY de.day_id ASC",
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;
        let mut stats = Vec::new();

        let day_names = ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche"];

        while let Some(row) = map_storage_error(rows.next())? {
            let day_index: u8 = map_storage_error(row.get(0))?;
            let worked_days: i64 = map_storage_error(row.get(1))?;
            let total_minutes: i64 = map_storage_error(row.get(2))?;

            let worked_days = worked_days.max(0) as u32;
            let total_minutes = total_minutes.max(0) as u32;
            let average_minutes = if worked_days > 0 {
                total_minutes / worked_days
            } else {
                0
            };

            stats.push(DayOfWeekStats {
                day_name: day_names.get(day_index as usize).copied().unwrap_or("Inconnu").to_string(),
                average_minutes,
            });
        }

        Ok(stats)
    }

    pub fn get_weekly_trends(&self) -> Result<Vec<WeeklyTrendPoint>, StorageError> {
        let connection = self.shared()?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                w.week_start,
                COALESCE(SUM(
                    CASE
                        WHEN de.enabled = 1
                            AND de.start_minutes IS NOT NULL
                            AND de.end_minutes IS NOT NULL
                        THEN GREATEST(0,
                            de.end_minutes - de.start_minutes - de.break_minutes
                            - COALESCE(w.travel_deduction_minutes, 30) * COALESCE(de.has_departure_deduction, 0)
                            - COALESCE(w.travel_deduction_minutes, 30) * COALESCE(de.has_return_deduction, 0)
                        )
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
        let connection = self.shared()?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                SUBSTR(w.week_start, 1, 7) as month,
                COUNT(DISTINCT w.id) as weeks_count,
                COALESCE(SUM(
                    CASE
                        WHEN de.enabled = 1
                            AND de.start_minutes IS NOT NULL
                            AND de.end_minutes IS NOT NULL
                        THEN GREATEST(0,
                            de.end_minutes - de.start_minutes - de.break_minutes
                            - COALESCE(w.travel_deduction_minutes, 30) * COALESCE(de.has_departure_deduction, 0)
                            - COALESCE(w.travel_deduction_minutes, 30) * COALESCE(de.has_return_deduction, 0)
                        )
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

    /// Récupère les données pour les courbes analytics (présence vs heures sup)
    /// Retourne les 12 dernières semaines avec numéro de semaine ISO
    pub fn get_weekly_curves(&self) -> Result<Vec<WeekAnalyticsPoint>, StorageError> {
        let connection = open_connection(&self.database_path)?;

        let mut statement = map_storage_error(connection.prepare(
            "SELECT
                w.week_start,
                w.overtime_threshold_minutes,
                w.travel_deduction_minutes,
                COALESCE(SUM(
                    CASE
                        WHEN de.enabled = 1
                            AND de.start_minutes IS NOT NULL
                            AND de.end_minutes IS NOT NULL
                        THEN GREATEST(0,
                            de.end_minutes - de.start_minutes - de.break_minutes
                            - COALESCE(w.travel_deduction_minutes, 30) * COALESCE(de.has_departure_deduction, 0)
                            - COALESCE(w.travel_deduction_minutes, 30) * COALESCE(de.has_return_deduction, 0)
                        )
                        ELSE 0
                    END
                ), 0) as effective_minutes
            FROM weeks w
            LEFT JOIN day_entries de ON w.id = de.week_id
            GROUP BY w.id, w.week_start, w.overtime_threshold_minutes, w.travel_deduction_minutes
            ORDER BY w.week_start DESC
            LIMIT 12",
        ))?;

        let mut rows = map_storage_error(statement.query([]))?;
        let mut curves = Vec::new();

        while let Some(row) = map_storage_error(rows.next())? {
            let week_start: String = map_storage_error(row.get(0))?;
            let overtime_threshold: i64 = map_storage_error(row.get(1))?;
            let effective_minutes: i64 = map_storage_error(row.get(2))?;

            let effective = effective_minutes.max(0) as u32;
            let threshold = overtime_threshold.max(0) as u32;
            
            // Heures sup consommées = excès au-dessus du seuil
            let consumed_overtime = effective.saturating_sub(threshold);
            
            // Calcul du numéro de semaine ISO à partir de la date
            let week_number = Self::iso_week_number(&week_start);

            curves.push(WeekAnalyticsPoint {
                week_start,
                week_number,
                effective_minutes: effective,
                consumed_overtime_minutes: consumed_overtime,
            });
        }

        // Inverser pour avoir l'ordre chronologique (du plus ancien au plus récent)
        curves.reverse();
        Ok(curves)
    }

    /// Calcule le numéro de semaine ISO à partir d'une date (format YYYY-MM-DD)
    fn iso_week_number(date_str: &str) -> u32 {
        use chrono::Datelike;
        
        // Parse la date
        let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        
        // Retourne le numéro de semaine ISO
        date.iso_week().week()
    }
}

#[cfg(test)]
mod connection_reuse_tests {
    use tempfile::tempdir;

    use crate::{
        domain::{
            logic::{default_entries, default_settings},
            types::{OvertimeThresholdMinutes, TravelDeductionMinutes, WeekId, WeekSheet, WeekStartDate},
        },
        infrastructure::duckdb::DuckDb,
    };

    #[test]
    fn reuses_single_connection_across_operations() {
        let temp_dir = tempdir().expect("temp dir");
        let store = DuckDb::new(temp_dir.path().join("reuse.duckdb"));

        store.migrate().expect("migrations");
        let count_after_migrate = store.connection_open_count();
        assert_eq!(count_after_migrate, 1, "migrate() ouvre la connexion partagée");

        store.ensure_default_settings().expect("default settings");

        let week = WeekSheet {
            week_id: Some(WeekId::new()),
            week_start: WeekStartDate::today(),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
        };

        store.save_week(&week).expect("save");
        store
            .get_week_by_id(week.week_id.as_ref().expect("persisted week must have id"))
            .expect("load")
            .expect("week should exist");
        store.list_weeks().expect("list");
        store.load_settings().expect("settings");

        // Une seule ouverture pour servir toutes les opérations successives :
        // rouvrir le fichier DuckDB à chaque appel coûte cher (buffer pool,
        // métadonnées, checksum) et domine la latence de l'autosave.
        assert_eq!(
            store.connection_open_count(),
            1,
            "toutes les opérations doivent partager la connexion ouverte par migrate()"
        );
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        domain::{
            logic::{default_entries, default_settings, summarize_week},
            types::{OvertimeThresholdMinutes, TravelDeductionMinutes, WeekSheet, WeekStartDate},
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
            week_id: Some(crate::domain::types::WeekId::new()),
            week_start: WeekStartDate::today(),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
        };

        store.save_week(&week).expect("save");
        let loaded = store
            .get_week_by_id(week.week_id.as_ref().expect("persisted week must have id"))
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
            week_id: Some(crate::domain::types::WeekId::new()),
            week_start: WeekStartDate::today(),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
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
            week_id: Some(crate::domain::types::WeekId::new()),
            week_start: WeekStartDate::today(),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
        };
        store.save_week(&week1).expect("save week1");

        let week2 = WeekSheet {
            week_id: Some(crate::domain::types::WeekId::new()),
            week_start: WeekStartDate::parse("2024-01-15").expect("parse date"),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
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

#[cfg(test)]
mod balance_tests {
    use duckdb::params;
    use tempfile::tempdir;

    use crate::{
        domain::{
            logic::{default_entries, default_settings, summarize_week},
            types::{OvertimeThresholdMinutes, TravelDeductionMinutes, WeekId, WeekSheet, WeekStartDate},
        },
        infrastructure::duckdb::DuckDb,
    };

    #[test]
    fn balance_reflects_each_weeks_own_threshold() {
        // Deux semaines de 2700 min avec des seuils différents :
        // le solde doit être la somme des deltas hebdo (source unique Rust).
        let temp_dir = tempdir().expect("temp dir");
        let store = DuckDb::new(temp_dir.path().join("balance_mixed.duckdb"));
        store.migrate().expect("migrations");

        let week1 = WeekSheet {
            week_id: Some(WeekId::new()),
            week_start: WeekStartDate::parse("2030-01-07").expect("date"),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
        };
        let week2 = WeekSheet {
            week_id: Some(WeekId::new()),
            week_start: WeekStartDate::parse("2030-01-14").expect("date"),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(3000),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
        };
        store.save_week(&week1).expect("save w1");
        store.save_week(&week2).expect("save w2");

        let total1 = i32::from(summarize_week(&week1).expect("s1").total_minutes);
        let total2 = i32::from(summarize_week(&week2).expect("s2").total_minutes);
        let expected = (total1 - 2100) + (total2 - 3000);

        let balance = store
            .get_cumulative_balance(&WeekStartDate::parse("2030-06-01").expect("date"))
            .expect("balance");
        assert_eq!(balance, expected);
    }

    #[test]
    fn balance_fails_closed_on_corrupt_stored_row() {
        // Une ligne invalide en base (fin < début) doit produire une ERREUR
        // visible, jamais un solde silencieusement faux. Avant : le SQL
        // additionnait le négatif sans rien dire.
        let temp_dir = tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("balance_corrupt.duckdb");
        let store = DuckDb::new(db_path.clone());
        store.migrate().expect("migrations");

        let week = WeekSheet {
            week_id: Some(WeekId::new()),
            week_start: WeekStartDate::parse("2030-01-07").expect("date"),
            entries: default_entries(&default_settings()),
            overtime_threshold: OvertimeThresholdMinutes(2100),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
        };
        store.save_week(&week).expect("save");

        // Corrompre via la connexion partagée du store (DuckDB interdit une
        // seconde connexion sur le même fichier dans un même process).
        {
            let guard = store.raw_connection();
            let connection = guard.as_ref().expect("connexion partagée");
            connection
                .execute(
                    "UPDATE day_entries SET start_minutes = 1080, end_minutes = 480
                     WHERE week_id = ?1 AND day_id = 0",
                    params![week.week_id.as_ref().expect("persisted week").0],
                )
                .expect("corrupt");
        } // guard drop ici, verrou libéré

        let balance = store.get_cumulative_balance(&week.week_start);
        assert!(balance.is_err(), "le solde doit échouer proprement, pas mentir, reçu {balance:?}");
    }
}


#[cfg(test)]
mod analytics_consistency_tests {
    use tempfile::tempdir;

    use crate::{
        domain::{
            logic::{default_entries, default_settings},
            types::{
                BreakMinutes, DayEntry, DayId, DayLabel, OvertimeThresholdMinutes,
                TimeOfDay, TravelDeductionMinutes, WeekId, WeekSheet, WeekStartDate, WorkInterval,
            },
        },
        infrastructure::duckdb::DuckDb,
    };

    /// Remplace le lundi des entrées par défaut par un jour custom.
    /// Les autres 6 jours restent 08:00-18:00/60min (valides).
    fn week_with(monday: DayEntry, week_start: &str) -> WeekSheet {
        let mut entries = default_entries(&default_settings());
        entries[0] = monday;
        WeekSheet {
            week_id: Some(WeekId::new()),
            week_start: WeekStartDate::parse(week_start).expect("date"),
            entries,
            overtime_threshold: OvertimeThresholdMinutes(2100),
            travel_deduction_minutes: TravelDeductionMinutes::default(),
            updated_at: String::new(),
        }
    }

    /// Lundi travaillé : start-end-break valides, 9h00 par défaut.
    fn active_monday() -> DayEntry {
        DayEntry {
            day_id: DayId(0),
            label: DayLabel("Lundi".to_string()),
            interval: Some(WorkInterval {
                start: TimeOfDay(8 * 60),
                end: TimeOfDay(17 * 60),
            }),
            break_minutes: BreakMinutes(0),
            enabled: true,
            has_departure_deduction: false,
            has_return_deduction: false,
        }
    }

    /// Lundi désactivé : intervalle présent mais compte pour 0 min en base.
    fn disabled_monday() -> DayEntry {
        DayEntry {
            day_id: DayId(0),
            label: DayLabel("Lundi".to_string()),
            interval: None,
            break_minutes: BreakMinutes(0),
            enabled: false,
            has_departure_deduction: false,
            has_return_deduction: false,
        }
    }

    #[test]
    fn average_per_day_ignores_disabled_days() {
        // 10 semaines : 8 lundis actifs (540 min chacun), 2 lundis désactivés.
        // Moyenne attendue : 8 * 540 / 8 jours travailles = 540 min/jour.
        // Avant : COUNT(*) divise par 10 -> moyenne artificiellement basse (432).
        let dir = tempdir().expect("dir");
        let store = DuckDb::new(dir.path().join("avg.duckdb"));
        store.migrate().expect("migrations");

        // 10 dates de lundi consécutives (2030-01-07 à 2030-03-11)
        let dates = [
            "2030-01-07", "2030-01-14", "2030-01-21", "2030-01-28", "2030-02-04",
            "2030-02-11", "2030-02-18", "2030-02-25", "2030-03-04", "2030-03-11",
        ];

        for (i, date) in dates.iter().enumerate() {
            let monday = if i < 8 { active_monday() } else { disabled_monday() };
            store.save_week(&week_with(monday, date)).expect("save");
        }

        let stats = store.get_day_of_week_stats().expect("stats");
        assert_eq!(stats.len(), 7, "un groupe par day_id (0-6)");
        
        let lundi = stats.iter().find(|s| s.day_name == "Lundi").expect("lundi");
        assert_eq!(
            lundi.average_minutes, 540,
            "moyenne calculee sur les jours travailles uniquement, pas tous les jours enregistres"
        );
    }

    #[test]
    fn renamed_day_merges_into_single_bar() {
        // Renommer "Lundi" en "Lundi pro" entre deux semaines ne doit pas
        // creer deux barres distinctes dans les stats.
        let dir = tempdir().expect("dir");
        let store = DuckDb::new(dir.path().join("rename.duckdb"));
        store.migrate().expect("migrations");

        let mut day_old = active_monday();
        day_old.label = DayLabel("Lundi".to_string());

        let mut day_new = active_monday();
        day_new.label = DayLabel("Lundi pro".to_string());

        store.save_week(&week_with(day_old, "2030-01-07")).expect("old");
        store.save_week(&week_with(day_new, "2030-01-14")).expect("new");

        let stats = store.get_day_of_week_stats().expect("stats");
        assert_eq!(stats.len(), 7, "7 groupes (un par day_id), pas 8");
        
        // Vérifier qu'il n'y a qu'un seul groupe pour le lundi (day_id=0)
        let lundi_groups: Vec<_> = stats.iter().filter(|s| s.day_name == "Lundi").collect();
        assert_eq!(lundi_groups.len(), 1, "groupe par day_id, pas par (day_id, label)");
    }

    #[test]
    fn negative_daily_minutes_are_clamped_to_zero() {
        // Une ligne corrompue (end < start) en base ne doit PAS faire
        // chuter la moyenne en dessous de 0.
        let dir = tempdir().expect("dir");
        let store = DuckDb::new(dir.path().join("neg.duckdb"));
        store.migrate().expect("migrations");

        let week = week_with(active_monday(), "2030-01-07");
        store.save_week(&week).expect("save");
        let week_id = week.week_id.clone();

        // Corromper directement le lundi via la connexion partagée : swap start/end (fin < debut).
        {
            let guard = store.raw_connection();
            let conn = guard.as_ref().expect("connexion partagée");
            conn.execute(
                "UPDATE day_entries SET start_minutes = 17*60, end_minutes = 8*60
                 WHERE week_id = ?1 AND day_id = 0",
                duckdb::params![week_id.expect("persisted week").0],
            )
            .expect("corrupt");
        } // guard drop -> verrou libéré

        let stats = store.get_day_of_week_stats().expect("stats");
        assert_eq!(stats.len(), 7, "7 groupes");
        
        let lundi = stats.iter().find(|s| s.day_name == "Lundi").expect("lundi");
        assert_eq!(
            lundi.average_minutes, 0,
            "minutes negatives clampées à 0, jamais propagees dans la moyenne"
        );
    }
}

#[cfg(test)]
mod balance_sql_equivalence_tests {
    use duckdb::params;
    use tempfile::tempdir;

    use crate::{
        domain::{
            logic::{default_entries, default_settings},
            types::{BreakMinutes, DayEntry, DayId, DayLabel, OvertimeThresholdMinutes, TimeOfDay, WeekId, WeekSheet, WeekStartDate, WorkInterval},
        },
        infrastructure::duckdb::DuckDb,
    };

    fn store() -> (tempfile::TempDir, DuckDb) {
        let dir = tempdir().expect("dir");
        let store = DuckDb::new(dir.path().join("balance_sql.duckdb"));
        store.migrate().expect("migrations");
        (dir, store)
    }

    /// Lundi désactivé avec un intervalle inversé conservé en base.
    fn disabled_inverted_monday() -> DayEntry {
        DayEntry {
            day_id: DayId(0),
            label: DayLabel("Lundi".to_string()),
            interval: Some(WorkInterval {
                start: TimeOfDay(17 * 60),
                end: TimeOfDay(8 * 60),
            }),
            break_minutes: BreakMinutes(0),
            enabled: false,
            has_departure_deduction: false,
            has_return_deduction: false,
        }
    }

    fn week_with(monday: DayEntry, week_start: &str) -> WeekSheet {
        let mut entries = default_entries(&default_settings());
        entries[0] = monday;
        WeekSheet {
            week_id: Some(WeekId::new()),
            week_start: WeekStartDate::parse(week_start).expect("date"),
            entries,
            overtime_threshold: OvertimeThresholdMinutes(2100),
            updated_at: String::new(),
        }
    }

    #[test]
    fn travel_deductions_reduce_balance_like_rust() {
        // Chaque tick de deduction (depart/retour) retire 30 min du jour,
        // sature a 0 : l'agregat SQL doit suivre summarize_week a la lettre.
        let (_dir, store) = store();
        let mut monday = DayEntry {
            day_id: DayId(0),
            label: DayLabel("Lundi".to_string()),
            interval: Some(WorkInterval {
                start: TimeOfDay(8 * 60),
                end: TimeOfDay(17 * 60),
            }),
            break_minutes: BreakMinutes(0),
            enabled: true,
            has_departure_deduction: true,
            has_return_deduction: true,
        };
        let week = week_with(monday.clone(), "2030-01-07");
        store.save_week(&week).expect("save");

        // Attendu : la boucle Rust (summarize_week) sur la meme feuille.
        let mut sheet = week_with(disabled_inverted_monday(), "2030-01-07");
        sheet.entries[0] = monday;
        let expected = i32::from(
            crate::domain::logic::summarize_week(&sheet).expect("s").total_minutes,
        ) - 2100;

        let balance =
            store.get_cumulative_balance(&WeekStartDate::parse("2030-06-01").expect("date"));
        assert_eq!(balance.expect("balance"), expected);
    }

    #[test]
    fn disabled_day_with_inverted_interval_counts_zero() {
        // Un jour désactivé garde son intervalle en base même invalide :
        // validate_day retourne Ok, le jour compte pour 0 minute.
        let (_dir, store) = store();
        let week = week_with(disabled_inverted_monday(), "2030-01-07");
        store.save_week(&week).expect("save");

        let mut expected_week = week_with(disabled_inverted_monday(), "2030-01-07");
        expected_week.entries[0].interval = None;
        let expected = i32::from(
            crate::domain::logic::summarize_week(&expected_week).expect("s").total_minutes,
        ) - 2100;

        let balance = store
            .get_cumulative_balance(&WeekStartDate::parse("2030-06-01").expect("date"))
            .expect("balance");
        assert_eq!(balance, expected);
    }

    #[test]
    fn invalid_row_in_future_week_is_not_validated() {
        // summarize_week ne tourne que sur les semaines <= up_to :
        // une corruption après la date demandée ne doit pas faire échouer.
        let (_dir, store) = store();
        let week = week_with(disabled_inverted_monday(), "2030-01-07");
        // activer le lundi invalide puis le corrompre n'est pas nécessaire :
        // on corrompt une semaine future directement
        let future = week_with(disabled_inverted_monday(), "2030-03-04");
        store.save_week(&week).expect("save past");
        store.save_week(&future).expect("save future");
        {
            let guard = store.raw_connection();
            let conn = guard.as_ref().expect("connexion partagée");
            conn.execute(
                "UPDATE day_entries SET enabled = 1 WHERE week_id = ?1 AND day_id = 0",
                params![future.week_id.as_ref().expect("id").0],
            )
            .expect("corrupt future");
        }

        let balance =
            store.get_cumulative_balance(&WeekStartDate::parse("2030-02-01").expect("date"));
        assert!(balance.is_ok(), "semaine future non validée, reçu {balance:?}");
    }

    #[test]
    fn empty_label_fails_even_when_day_disabled() {
        // validate_day vérifie le label AVANT le flag enabled.
        let (_dir, store) = store();
        let week = week_with(disabled_inverted_monday(), "2030-01-07");
        store.save_week(&week).expect("save");
        {
            let guard = store.raw_connection();
            let conn = guard.as_ref().expect("connexion partagée");
            conn.execute(
                "UPDATE day_entries SET label = '' WHERE week_id = ?1 AND day_id = 2",
                params![week.week_id.as_ref().expect("id").0],
            )
            .expect("empty label");
        }

        let balance =
            store.get_cumulative_balance(&WeekStartDate::parse("2030-06-01").expect("date"));
        assert!(balance.is_err(), "label vide => erreur, reçu {balance:?}");
    }

    #[test]
    fn break_exceeds_day_fails() {
        // enabled avec break >= durée : BreakExceedsDay côté Rust.
        let (_dir, store) = store();
        let monday = DayEntry {
            day_id: DayId(0),
            label: DayLabel("Lundi".to_string()),
            interval: Some(WorkInterval {
                start: TimeOfDay(8 * 60),
                end: TimeOfDay(17 * 60),
            }),
            break_minutes: BreakMinutes(9 * 60),
            enabled: true,
            has_departure_deduction: false,
            has_return_deduction: false,
        };
        let week = week_with(monday, "2030-01-07");
        store.save_week(&week).expect("save");

        let balance =
            store.get_cumulative_balance(&WeekStartDate::parse("2030-06-01").expect("date"));
        assert!(balance.is_err(), "pause >= durée => erreur, reçu {balance:?}");
    }

    #[test]
    fn large_history_balance_matches_rust_computation() {
        // Équivalence agrégat SQL vs boucle Rust sur 60 semaines à seuils mixtes.
        let (_dir, store) = store();
        let mut expected: i32 = 0;
        for i in 0..60 {
            let date = chrono::NaiveDate::from_ymd_opt(2030, 1, 7).expect("date")
                + chrono::Duration::weeks(i);
            let week_start = date.format("%Y-%m-%d").to_string();
            let sheet = WeekSheet {
                week_id: Some(WeekId::new()),
                week_start: WeekStartDate::parse(&week_start).expect("date"),
                entries: default_entries(&default_settings()),
                overtime_threshold: OvertimeThresholdMinutes(if i % 2 == 0 { 2100 } else { 1950 }),
                updated_at: String::new(),
            };
            let total = i32::from(
                crate::domain::logic::summarize_week(&sheet).expect("s").total_minutes,
            );
            // Le solde ne compte que les semaines <= up_to (2031-01-01) :
            // les 8 dernières semaines de l'historique sont hors périmètre.
            if i <= 51 {
                expected += total - i32::from(if i % 2 == 0 { 2100 } else { 1950 });
            }
            store.save_week(&sheet).expect("save");
        }

        let balance =
            store.get_cumulative_balance(&WeekStartDate::parse("2031-01-01").expect("date"));
        assert_eq!(balance.expect("balance"), expected);
    }
}
