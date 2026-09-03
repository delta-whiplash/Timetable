#![cfg(feature = "storage-duckdb")]

#![cfg(feature = "storage-duckdb")]

use std::sync::Arc;

use timetable_desktop_lib::{
    application::{
        dto::{
            ConfiguredDayView, DeleteWeekInput, SaveSettingsInput, SaveWeekDayEntryInput,
            SaveWeekInput, WeekSelectorInput,
        },
        service::ApplicationService,
    },
    domain::{
        errors::{ApplicationError, StorageError},
        logic::{default_entries, default_settings},
        types::{OvertimeThresholdMinutes, WeekId, WeekSheet, WeekStartDate},
    },
    infrastructure::duckdb::DuckDb,
};
use tempfile::tempdir;

#[test]
fn save_settings_does_not_retroactively_change_past_week_threshold() {
    // Changer le seuil ne doit PAS rétro-écrire le seuil des semaines passées,
    // seulement celui de la semaine active si c'est la semaine courante.
    let temp_dir = tempdir().expect("temp dir");
    let store = Arc::new(DuckDb::new(temp_dir.path().join("threshold.duckdb")));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");
    let service = ApplicationService::new(store.clone());

    // Sauvegarder une semaine passée (2020-01-06)
    let view = service
        .create_or_switch_week(WeekSelectorInput {
            week_start: "2020-01-06".to_string(),
        })
        .expect("create past week");

    // Sauvegarder explicitement pour persister (C1 empêche la persistance auto)
    service
        .save_week(SaveWeekInput {
            week_id: view.week_id,
            week_start: "2020-01-06".to_string(),
            overtime_threshold_minutes: 2100,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("08:00".to_string()),
                end: Some("18:00".to_string()),
                break_time: "01:00".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("save past week");

    // Vérifier le seuil initial (35h = 2100 min)
    let week_before = store
        .get_week_by_start(&WeekStartDate::parse("2020-01-06").expect("date"))
        .expect("query")
        .expect("week exists");
    assert_eq!(week_before.overtime_threshold.0, 2100);

    // Changer le seuil global à 40h (2400 min)
    let configured_days: Vec<ConfiguredDayView> = default_settings()
        .configured_days
        .iter()
        .map(|day| ConfiguredDayView {
            day_id: day.day_id.0,
            label: day.label.0.clone(),
            enabled: day.enabled,
        })
        .collect();

    service
        .save_settings(SaveSettingsInput {
            overtime_threshold_minutes: 2400,
            travel_deduction_minutes: 30,
            default_start: "08:00".to_string(),
            default_end: "18:00".to_string(),
            default_break: "01:00".to_string(),
            configured_days,
            enable_travel_deduction: true,
        })
        .expect("change threshold");

    // La semaine passée doit garder son seuil de 2100 (pas rétro-écrit)
    let week_after = store
        .get_week_by_start(&WeekStartDate::parse("2020-01-06").expect("date"))
        .expect("query")
        .expect("week exists");
    assert_eq!(
        week_after.overtime_threshold.0, 2100,
        "le seuil d'une semaine passée ne doit pas être rétro-écrit"
    );
}

#[test]
fn delete_week_is_transactional() {
    // Supprimer une semaine doit être atomique : soit tout est supprimé,
    // soit rien. Pas de coquille vide (day_entries sans weeks).
    let temp_dir = tempdir().expect("temp dir");
    let store = Arc::new(DuckDb::new(temp_dir.path().join("delete.duckdb")));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");
    let service = ApplicationService::new(store.clone());

    // Créer et sauvegarder une semaine
    let view = service
        .create_or_switch_week(WeekSelectorInput {
            week_start: "2030-04-01".to_string(),
        })
        .expect("create week");

    // Sauvegarder explicitement pour persister
    service
        .save_week(SaveWeekInput {
            week_id: view.week_id,
            week_start: "2030-04-01".to_string(),
            overtime_threshold_minutes: 2100,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("08:00".to_string()),
                end: Some("18:00".to_string()),
                break_time: "01:00".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("save week");

    let week = store
        .get_week_by_start(&WeekStartDate::parse("2030-04-01").expect("date"))
        .expect("query")
        .expect("week exists");

    // Supprimer
    service
        .delete_week(DeleteWeekInput { week_id: week.week_id.expect("persisted week must have id").0.clone() })
        .expect("delete");

    // Vérifier qu'il n'y a ni semaine ni day_entries résiduels
    assert!(
        store
            .get_week_by_start(&WeekStartDate::parse("2030-04-01").expect("date"))
            .expect("query")
            .is_none()
    );

    // Vérifier via SQL direct qu'il n'y a pas de coquille vide
    let conn = duckdb::Connection::open(&store.database_path).expect("open");
    let orphan_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM day_entries WHERE week_id NOT IN (SELECT id FROM weeks)",
            [],
            |row| row.get(0),
        )
        .expect("query orphans");
    assert_eq!(orphan_count, 0, "pas de day_entries orphelins après suppression");
}

#[test]
fn list_weeks_returns_real_updated_at() {
    // list_weeks doit retourner le vrai updated_at de la DB, pas un timestamp fabriqué.
    let temp_dir = tempdir().expect("temp dir");
    let store = Arc::new(DuckDb::new(temp_dir.path().join("updated.duckdb")));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");
    let service = ApplicationService::new(store.clone());

    // Créer et sauvegarder une semaine
    let view = service
        .save_week(SaveWeekInput {
            week_id: Some("test-week".to_string()),
            week_start: "2030-05-06".to_string(),
            overtime_threshold_minutes: 2100,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("08:00".to_string()),
                end: Some("18:00".to_string()),
                break_time: "01:00".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("save week");

    // Attendre un peu pour que updated_at soit différent de week_start
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Lister les semaines
    let weeks = service.list_weeks().expect("list");
    assert_eq!(weeks.len(), 1);

    // updated_at ne doit PAS être "{week_start} 00:00"
    assert_ne!(
        weeks[0].updated_at, "2030-05-06 00:00",
        "updated_at doit être le vrai timestamp de la DB, pas un timestamp fabriqué"
    );

    // updated_at doit être un timestamp ISO 8601 valide
    assert!(
        weeks[0].updated_at.contains("T"),
        "updated_at doit être un timestamp RFC3339/ISO8601"
    );
}

#[test]
fn save_week_returns_fresh_balance() {
    // save_week doit retourner un solde qui INCLUT la semaine qui vient d'être sauvée,
    // pas le solde précédent.
    let temp_dir = tempdir().expect("temp dir");
    let store = Arc::new(DuckDb::new(temp_dir.path().join("balance_fresh.duckdb")));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");
    let service = ApplicationService::new(store.clone());

    // Sauvegarder une semaine avec 10h de travail (seuil 35h = -25h)
    let view = service
        .save_week(SaveWeekInput {
            week_id: Some("balance-test".to_string()),
            week_start: "2030-06-03".to_string(),
            overtime_threshold_minutes: 2100,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("08:00".to_string()),
                end: Some("18:00".to_string()),
                break_time: "00:00".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("save week");

    // Le solde cumulé dans la vue retournée doit inclure cette semaine
    // 10h - 35h = -25h = -1500 min
    let balance_label = view.summary.cumulative_balance_label.clone();
    assert_eq!(
        balance_label, "-25h00",
        "le solde retourné doit inclure la semaine qui vient d'être sauvée"
    );
}

#[test]
fn resolve_active_week_falls_back_on_invalid_week() {
    // Si la semaine active pointée par settings est invalide (corrompue),
    // resolve_active_week doit fallbacker sur today() au lieu de paniquer.
    let temp_dir = tempdir().expect("temp dir");
    let store = Arc::new(DuckDb::new(temp_dir.path().join("lockout.duckdb")));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");
    let service = ApplicationService::new(store.clone());

    // Sauvegarder une semaine
    let view = service
        .save_week(SaveWeekInput {
            week_id: Some("will-be-corrupt".to_string()),
            week_start: "2030-07-01".to_string(),
            overtime_threshold_minutes: 2100,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("08:00".to_string()),
                end: Some("18:00".to_string()),
                break_time: "01:00".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("save week");

    // Corrompre la semaine en SQL (fin < début)
    {
        let conn = duckdb::Connection::open(&store.database_path).expect("open");
        conn.execute(
            "UPDATE day_entries SET start_minutes = 1080, end_minutes = 480 WHERE week_id = ?1",
            duckdb::params![view.week_id.clone()],
        )
        .expect("corrupt");
    }

    // load_bootstrap doit réussir (fallback sur today), pas paniquer
    let bootstrap = service.load_bootstrap().expect("bootstrap should fallback");
    assert!(
        bootstrap.active_week.week_start != "2030-07-01",
        "bootstrap doit fallbacker sur une semaine valide, pas sur la semaine corrompue"
    );
}

#[test]
fn saves_week_and_updates_summary() {
    let temp_dir = tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("workflow.duckdb");

    let store = Arc::new(DuckDb::new(db_path));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");

    let service = ApplicationService::new(store);

    let bootstrap = service.load_bootstrap().expect("bootstrap");
    assert_eq!(bootstrap.active_week.entries.len(), 7);

    let week = bootstrap.active_week;
    let result = service
        .save_week(SaveWeekInput {
            week_id: week.week_id,
            week_start: week.week_start,
            overtime_threshold_minutes: 35 * 60,
            travel_deduction_minutes: 30,
            entries: vec![
                SaveWeekDayEntryInput {
                    day_id: 0,
                    label: "Lundi".to_string(),
                    enabled: true,
                    start: Some("08:00".to_string()),
                    end: Some("18:00".to_string()),
                    break_time: "01:00".to_string(),
                    has_departure_deduction: false,
                    has_return_deduction: false,
                },
                SaveWeekDayEntryInput {
                    day_id: 1,
                    label: "Mardi".to_string(),
                    enabled: true,
                    start: Some("08:00".to_string()),
                    end: Some("17:30".to_string()),
                    break_time: "00:30".to_string(),
                    has_departure_deduction: false,
                    has_return_deduction: false,
                },
                SaveWeekDayEntryInput {
                    day_id: 2,
                    label: "Mercredi".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                    has_departure_deduction: false,
                    has_return_deduction: false,
                },
                SaveWeekDayEntryInput {
                    day_id: 3,
                    label: "Jeudi".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                    has_departure_deduction: false,
                    has_return_deduction: false,
                },
                SaveWeekDayEntryInput {
                    day_id: 4,
                    label: "Vendredi".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                    has_departure_deduction: false,
                    has_return_deduction: false,
                },
                SaveWeekDayEntryInput {
                    day_id: 5,
                    label: "Samedi".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                    has_departure_deduction: false,
                    has_return_deduction: false,
                },
                SaveWeekDayEntryInput {
                    day_id: 6,
                    label: "Dimanche".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                    has_departure_deduction: false,
                    has_return_deduction: false,
                }
            ],
        })
        .expect("save week");

    assert_eq!(result.summary.total_label, "18h00");
    assert_eq!(result.entries[0].total_minutes, 540);
    assert_eq!(result.entries[1].total_minutes, 540);

    let defaults = default_settings();
    let updated_settings = service
        .save_settings(SaveSettingsInput {
            overtime_threshold_minutes: 30 * 60,
            travel_deduction_minutes: 30,
            default_start: defaults.default_work_interval.start.to_hhmm(),
            default_end: defaults.default_work_interval.end.to_hhmm(),
            default_break: "01:00".to_string(),
            configured_days: defaults
                .configured_days
                .into_iter()
                .map(|day| timetable_desktop_lib::application::dto::ConfiguredDayView {
                    day_id: day.day_id.0,
                    label: day.label.0,
                    enabled: day.enabled,
                })
                .collect(),
            enable_travel_deduction: true,
        })
        .expect("save settings");

    assert_eq!(updated_settings.overtime_threshold_minutes, 1800);
    assert_eq!(WeekStartDate::parse("2026-03-12").expect("date").as_string(), "2026-03-09");
}

#[test]
fn exports_week_as_xlsx() {
    let temp_dir = tempdir().expect("temp dir");
    let store = Arc::new(DuckDb::new(temp_dir.path().join("export.duckdb")));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");

    let service = ApplicationService::new(store);

    // Navigue vers la semaine puis la sauvegarde explicitement
    // (l'export ne concerne que les semaines réellement saisies)
    let view = service
        .create_or_switch_week(WeekSelectorInput {
            week_start: "2026-09-07".to_string(),
        })
        .expect("create week");

    service
        .save_week(SaveWeekInput {
            week_id: view.week_id,
            week_start: "2026-09-07".to_string(),
            overtime_threshold_minutes: 35 * 60,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("08:00".to_string()),
                end: Some("18:00".to_string()),
                break_time: "01:00".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("save week before export");

    let exported = service
        .export_week("2026-09-07".to_string())
        .expect("export week");

    assert_eq!(exported.file_name, "timetable-2026-09-07.xlsx");
    assert_eq!(&exported.bytes[..4], b"PK\x03\x04", "un xlsx est un zip");
    assert!(exported.bytes.len() > 500);

    // Semaine inexistante : erreur explicite, pas de fichier fantôme
    let missing = service.export_week("2020-01-06".to_string());
    assert!(matches!(
        missing,
        Err(ApplicationError::Storage(StorageError::EntityNotFound))
    ));
}

#[test]
fn week_navigation_does_not_persist_template() {
    let temp_dir = tempdir().expect("temp dir");
    let store = Arc::new(DuckDb::new(temp_dir.path().join("nav.duckdb")));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");

    let service = ApplicationService::new(store.clone());

    // Naviguer vers une semaine jamais saisie ne doit RIEN écrire en base :
    // sinon le solde cumulé compte des heures template jamais travaillées.
    // Issue #1 : la semaine non sauvegardée n'a pas encore d'ID (None).
    let view = service
        .create_or_switch_week(WeekSelectorInput {
            week_start: "2030-01-07".to_string(),
        })
        .expect("switch to future week");

    // La vue retournée contient bien les 7 jours pré-remplis (template en mémoire)
    assert_eq!(view.entries.len(), 7);

    // Issue #1 : week_id doit être None tant que la semaine n'est pas sauvegardée
    assert!(
        view.week_id.is_none(),
        "une semaine non sauvegardée ne doit pas avoir d'ID"
    );

    // Mais aucune semaine n'est persistée tant que l'utilisateur n'a pas sauvegardé
    assert!(
        store
            .get_week_by_start(&WeekStartDate::parse("2030-01-07").expect("date"))
            .expect("query")
            .is_none()
    );

    // Et le solde cumulé ne compte pas d'heures fantômes
    assert_eq!(
        store
            .get_cumulative_balance(&WeekStartDate::parse("2030-06-01").expect("date"))
            .expect("balance"),
        0
    );

    // Après un save explicite avec week_id: None, la semaine est créée avec un ID généré
    let saved = service
        .save_week(SaveWeekInput {
            week_id: None,
            week_start: "2030-01-07".to_string(),
            overtime_threshold_minutes: 35 * 60,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("08:00".to_string()),
                end: Some("18:00".to_string()),
                break_time: "01:00".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("explicit save");

    // Issue #1 : après sauvegarde, la semaine a maintenant un ID
    assert!(
        saved.week_id.is_some(),
        "la semaine sauvegardée doit avoir un ID"
    );

    assert!(
        store
            .get_week_by_start(&WeekStartDate::parse("2030-01-07").expect("date"))
            .expect("query")
            .is_some()
    );
}

#[test]
fn duplicate_week_start_fails_at_db_level() {
    let temp_dir = tempdir().expect("temp dir");
    let store = DuckDb::new(temp_dir.path().join("dup.duckdb"));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");

    let week = WeekSheet {
        week_id: Some(WeekId("premiere".to_string())),
        week_start: WeekStartDate::parse("2030-02-04").expect("date"),
        entries: default_entries(&default_settings()),
        overtime_threshold: OvertimeThresholdMinutes(35 * 60),
        travel_deduction_minutes: timetable_desktop_lib::domain::types::TravelDeductionMinutes::default(),
        updated_at: String::new(),
    };
    store.save_week(&week).expect("première semaine");

    // Deuxième semaine, id différent, même week_start : la base doit refuser
    // (sinon le solde cumulé compte deux fois les mêmes heures)
    let doublon = WeekSheet {
        week_id: Some(WeekId("deuxieme".to_string())),
        week_start: WeekStartDate::parse("2030-02-04").expect("date"),
        entries: default_entries(&default_settings()),
        overtime_threshold: OvertimeThresholdMinutes(35 * 60),
        travel_deduction_minutes: timetable_desktop_lib::domain::types::TravelDeductionMinutes::default(),
        updated_at: String::new(),
    };
    let result = store.save_week(&doublon);
    assert!(
        matches!(result, Err(StorageError::QueryFailed)),
        "le doublon doit être rejeté par la contrainte UNIQUE, reçu: {result:?}"
    );

    // Une seule semaine en base
    assert_eq!(store.list_weeks().expect("list").len(), 1);
}

#[test]
fn save_with_stale_week_id_adopts_existing_week() {
    let temp_dir = tempdir().expect("temp dir");
    let store = Arc::new(DuckDb::new(temp_dir.path().join("adopt.duckdb")));

    store.migrate().expect("migrate");
    store.ensure_default_settings().expect("default settings");
    let service = ApplicationService::new(store.clone());

    // Sauvegarde normale : id A
    service
        .save_week(SaveWeekInput {
            week_id: Some("id-A".to_string()),
            week_start: "2030-03-04".to_string(),
            overtime_threshold_minutes: 35 * 60,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("08:00".to_string()),
                end: Some("18:00".to_string()),
                break_time: "01:00".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("save A");

    // Sauvegarde depuis un état obsolète (id B inconnu, même semaine) :
    // doit ADOPTER la ligne existante au lieu d'erreur ou de créer un doublon
    let view = service
        .save_week(SaveWeekInput {
            week_id: Some("id-B".to_string()),
            week_start: "2030-03-04".to_string(),
            overtime_threshold_minutes: 35 * 60,
            travel_deduction_minutes: 30,
            entries: vec![SaveWeekDayEntryInput {
                day_id: 0,
                label: "Lundi".to_string(),
                enabled: true,
                start: Some("09:00".to_string()),
                end: Some("17:00".to_string()),
                break_time: "00:30".to_string(),
                has_departure_deduction: false,
                has_return_deduction: false,
            }],
        })
        .expect("save B adopte la semaine existante");

    assert_eq!(view.week_id, Some("id-A".to_string()));
    let weeks = store.list_weeks().expect("list");
    assert_eq!(weeks.len(), 1, "une seule semaine, pas de doublon");
}
