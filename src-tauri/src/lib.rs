pub mod application;
pub mod domain;
pub mod infrastructure;

#[cfg(feature = "desktop")]
mod desktop {
    use std::{fs, path::PathBuf, sync::Arc};

    use tauri::{Manager, State};

    use crate::{
        application::{
            dto::{
                AnalyticsDataView, BootstrapState, DeleteWeekInput, SaveSettingsInput,
                SaveWeekInput, SettingsView, WeekListItem, WeekSelectorInput, WeekSheetView,
            },
            ports::{SettingsRepository, WeekRepository},
            service::ApplicationService,
        },
        domain::errors::{ApplicationError, PublicError},
        infrastructure::{
            config::AppRuntimeConfig,
            duckdb::{DuckDbSettingsRepository, DuckDbWeekRepository},
            tracing::init_tracing,
        },
    };

    pub struct SharedState {
        service: Arc<ApplicationService>,
    }

    fn to_public_error(context: &'static str, error: ApplicationError) -> PublicError {
        if !matches!(error, ApplicationError::Validation(_)) {
            tracing::error!(context, code = error.code(), "tauri command failed");
        }
        error.into()
    }

    #[tauri::command]
    fn load_bootstrap(state: State<'_, SharedState>) -> Result<BootstrapState, PublicError> {
        state
            .service
            .load_bootstrap()
            .map_err(|error| to_public_error("load_bootstrap", error))
    }

    #[tauri::command]
    fn save_week(
        state: State<'_, SharedState>,
        input: SaveWeekInput,
    ) -> Result<WeekSheetView, PublicError> {
        state
            .service
            .save_week(input)
            .map_err(|error| to_public_error("save_week", error))
    }

    #[tauri::command]
    fn create_or_switch_week(
        state: State<'_, SharedState>,
        input: WeekSelectorInput,
    ) -> Result<WeekSheetView, PublicError> {
        state
            .service
            .create_or_switch_week(input)
            .map_err(|error| to_public_error("create_or_switch_week", error))
    }

    #[tauri::command]
    fn list_weeks(state: State<'_, SharedState>) -> Result<Vec<WeekListItem>, PublicError> {
        state
            .service
            .list_weeks()
            .map_err(|error| to_public_error("list_weeks", error))
    }

    #[tauri::command]
    fn delete_week(
        state: State<'_, SharedState>,
        input: DeleteWeekInput,
    ) -> Result<(), PublicError> {
        state
            .service
            .delete_week(input)
            .map_err(|error| to_public_error("delete_week", error))
    }

    #[tauri::command]
    fn load_settings(state: State<'_, SharedState>) -> Result<SettingsView, PublicError> {
        state
            .service
            .load_settings()
            .map_err(|error| to_public_error("load_settings", error))
    }

    #[tauri::command]
    fn save_settings(
        state: State<'_, SharedState>,
        input: SaveSettingsInput,
    ) -> Result<SettingsView, PublicError> {
        state
            .service
            .save_settings(input)
            .map_err(|error| to_public_error("save_settings", error))
    }

    #[tauri::command]
    fn set_theme(state: State<'_, SharedState>, theme: String) -> Result<(), PublicError> {
        state
            .service
            .set_theme(theme)
            .map_err(|error| to_public_error("set_theme", error))
    }

    #[tauri::command]
    fn get_analytics(state: State<'_, SharedState>) -> Result<AnalyticsDataView, PublicError> {
        state
            .service
            .get_analytics()
            .map_err(|error| to_public_error("get_analytics", error))
    }

    fn resolve_app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("unable to resolve app data dir: {error}"))?;

        fs::create_dir_all(&app_data_dir)
            .map_err(|error| format!("unable to create app data dir: {error}"))?;

        Ok(app_data_dir)
    }

    pub fn run() {
        init_tracing();

        tauri::Builder::default()
            .setup(|app| {
                let app_data_dir =
                    resolve_app_data_dir(app.handle()).map_err(|error| -> Box<dyn std::error::Error> {
                        Box::new(std::io::Error::other(error))
                    })?;

                let runtime_config = AppRuntimeConfig::new(
                    app_data_dir.join("timetable.duckdb"),
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                    "com.delta.timetable",
                    1,
                );

                let week_repository =
                    Arc::new(DuckDbWeekRepository::new(runtime_config.database_path.clone()));
                let settings_repository =
                    Arc::new(DuckDbSettingsRepository::new(runtime_config.database_path.clone()));

                week_repository
                    .migrate()
                    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
                settings_repository
                    .ensure_default_settings()
                    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;

                let service = Arc::new(ApplicationService::new(
                    week_repository.clone(),
                    settings_repository,
                    week_repository, // DuckDbWeekRepository implémente aussi AnalyticsRepository
                ));

                app.manage(SharedState { service });
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                load_bootstrap,
                save_week,
                create_or_switch_week,
                list_weeks,
                delete_week,
                load_settings,
                save_settings,
                set_theme,
                get_analytics
            ])
            .run(tauri::generate_context!())
            .expect("tauri application failed to run");
    }
}

#[cfg(feature = "desktop")]
pub use desktop::run;

#[cfg(not(feature = "desktop"))]
pub fn run() {}
