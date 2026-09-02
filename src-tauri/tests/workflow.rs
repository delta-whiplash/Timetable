#![cfg(feature = "storage-duckdb")]

use std::sync::Arc;

use timetable_desktop_lib::{
    application::{
        dto::{SaveSettingsInput, SaveWeekDayEntryInput, SaveWeekInput},
        ports::{SettingsRepository, WeekRepository},
        service::ApplicationService,
    },
    domain::{logic::default_settings, types::WeekStartDate},
    infrastructure::duckdb::{DuckDbSettingsRepository, DuckDbWeekRepository},
};
use tempfile::tempdir;

#[test]
fn saves_week_and_updates_summary() {
    let temp_dir = tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("workflow.duckdb");

    let week_repository = Arc::new(DuckDbWeekRepository::new(db_path.clone()));
    let settings_repository = Arc::new(DuckDbSettingsRepository::new(db_path.clone()));

    week_repository.migrate().expect("migrate");
    settings_repository.ensure_default_settings().expect("default settings");

    let service = ApplicationService::new(
        week_repository.clone(),
        settings_repository,
        week_repository,
    );

    let bootstrap = service.load_bootstrap().expect("bootstrap");
    assert_eq!(bootstrap.active_week.entries.len(), 7);

    let week = bootstrap.active_week;
    let result = service
        .save_week(SaveWeekInput {
            week_id: week.week_id,
            week_start: week.week_start,
            overtime_threshold_minutes: 35 * 60,
            entries: vec![
                SaveWeekDayEntryInput {
                    day_id: 0,
                    label: "Lundi".to_string(),
                    enabled: true,
                    start: Some("08:00".to_string()),
                    end: Some("18:00".to_string()),
                    break_time: "01:00".to_string(),
                },
                SaveWeekDayEntryInput {
                    day_id: 1,
                    label: "Mardi".to_string(),
                    enabled: true,
                    start: Some("08:00".to_string()),
                    end: Some("17:30".to_string()),
                    break_time: "00:30".to_string(),
                },
                SaveWeekDayEntryInput {
                    day_id: 2,
                    label: "Mercredi".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                },
                SaveWeekDayEntryInput {
                    day_id: 3,
                    label: "Jeudi".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                },
                SaveWeekDayEntryInput {
                    day_id: 4,
                    label: "Vendredi".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                },
                SaveWeekDayEntryInput {
                    day_id: 5,
                    label: "Samedi".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
                },
                SaveWeekDayEntryInput {
                    day_id: 6,
                    label: "Dimanche".to_string(),
                    enabled: false,
                    start: None,
                    end: None,
                    break_time: "00:00".to_string(),
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
        })
        .expect("save settings");

    assert_eq!(updated_settings.overtime_threshold_minutes, 1800);
    assert_eq!(WeekStartDate::parse("2026-03-12").expect("date").as_string(), "2026-03-09");
}
