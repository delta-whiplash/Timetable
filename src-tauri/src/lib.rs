pub mod application;
pub mod domain;
pub mod infrastructure;

#[cfg(feature = "desktop")]
mod desktop {
    use std::{
        fs,
        path::PathBuf,
        sync::{Arc, OnceLock},
    };

    use tauri::{Manager, State};

    use crate::{
        application::{
            dto::{
                AnalyticsDataView, AppStatusView, BootstrapState, DeleteWeekInput, SaveSettingsInput,
                SaveWeekInput, SettingsView, ThemeInput, ThemeView, WeekListItem, WeekSelectorInput,
                WeekSheetView,
            },
            ports::{SettingsRepository, WeekRepository},
            service::ApplicationService,
        },
        domain::errors::{ApplicationError, PublicError},
        infrastructure::{
            config::AppRuntimeConfig,
            duckdb::{DuckDbDiagnosticsStore, DuckDbSettingsRepository, DuckDbWeekRepository},
            tracing::init_tracing,
        },
    };

    static RUNTIME_CONFIG: OnceLock<AppRuntimeConfig> = OnceLock::new();

    pub struct SharedState {
        service: Arc<ApplicationService>,
    }

    fn to_public_error(
        service: &ApplicationService,
        context: &'static str,
        error: ApplicationError,
    ) -> PublicError {
        if !matches!(error, ApplicationError::Validation(_)) {
            service.capture_error(context, &error);
        }

        error.into()
    }

    #[tauri::command]
    fn load_bootstrap(state: State<'_, SharedState>) -> Result<BootstrapState, PublicError> {
        state
            .service
            .load_bootstrap()
            .map_err(|error| to_public_error(&state.service, "load_bootstrap", error))
    }

    #[tauri::command]
    fn get_active_week(state: State<'_, SharedState>) -> Result<WeekSheetView, PublicError> {
        state
            .service
            .get_active_week()
            .map_err(|error| to_public_error(&state.service, "get_active_week", error))
    }

    #[tauri::command]
    fn save_week(
        state: State<'_, SharedState>,
        input: SaveWeekInput,
    ) -> Result<WeekSheetView, PublicError> {
        state
            .service
            .save_week(input)
            .map_err(|error| to_public_error(&state.service, "save_week", error))
    }

    #[tauri::command]
    fn create_or_switch_week(
        state: State<'_, SharedState>,
        input: WeekSelectorInput,
    ) -> Result<WeekSheetView, PublicError> {
        state
            .service
            .create_or_switch_week(input)
            .map_err(|error| to_public_error(&state.service, "create_or_switch_week", error))
    }

    #[tauri::command]
    fn list_weeks(state: State<'_, SharedState>) -> Result<Vec<WeekListItem>, PublicError> {
        state
            .service
            .list_weeks()
            .map_err(|error| to_public_error(&state.service, "list_weeks", error))
    }

    #[tauri::command]
    fn delete_week(
        state: State<'_, SharedState>,
        input: DeleteWeekInput,
    ) -> Result<(), PublicError> {
        state
            .service
            .delete_week(input)
            .map_err(|error| to_public_error(&state.service, "delete_week", error))
    }

    #[tauri::command]
    fn load_settings(state: State<'_, SharedState>) -> Result<SettingsView, PublicError> {
        state
            .service
            .load_settings()
            .map_err(|error| to_public_error(&state.service, "load_settings", error))
    }

    #[tauri::command]
    fn save_settings(
        state: State<'_, SharedState>,
        input: SaveSettingsInput,
    ) -> Result<SettingsView, PublicError> {
        state
            .service
            .save_settings(input)
            .map_err(|error| to_public_error(&state.service, "save_settings", error))
    }

    #[tauri::command]
    fn set_theme(
        state: State<'_, SharedState>,
        input: ThemeInput,
    ) -> Result<ThemeView, PublicError> {
        state
            .service
            .set_theme(input)
            .map_err(|error| to_public_error(&state.service, "set_theme", error))
    }

    #[tauri::command]
    fn get_app_status(state: State<'_, SharedState>) -> Result<AppStatusView, PublicError> {
        state
            .service
            .get_app_status()
            .map_err(|error| to_public_error(&state.service, "get_app_status", error))
    }

    #[tauri::command]
    fn export_data(state: State<'_, SharedState>) -> Result<String, PublicError> {
        state
            .service
            .export_data()
            .map_err(|error| to_public_error(&state.service, "export_data", error))
    }

    #[tauri::command]
    fn import_data(
        state: State<'_, SharedState>,
        json_data: String,
    ) -> Result<BootstrapState, PublicError> {
        state
            .service
            .import_data(json_data)
            .map_err(|error| to_public_error(&state.service, "import_data", error))
    }

    #[tauri::command]
    fn get_analytics(state: State<'_, SharedState>) -> Result<AnalyticsDataView, PublicError> {
        state
            .service
            .get_analytics()
            .map_err(|error| to_public_error(&state.service, "get_analytics", error))
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

                let runtime_config = RUNTIME_CONFIG.get_or_init(|| runtime_config).clone();

                let week_repository =
                    Arc::new(DuckDbWeekRepository::new(runtime_config.database_path.clone()));
                let settings_repository =
                    Arc::new(DuckDbSettingsRepository::new(runtime_config.database_path.clone()));
                let diagnostics_store =
                    Arc::new(DuckDbDiagnosticsStore::new(runtime_config.database_path.clone()));

                week_repository
                    .migrate()
                    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
                settings_repository
                    .ensure_default_settings()
                    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;

                let service = Arc::new(ApplicationService::new(
                    week_repository.clone(),
                    settings_repository,
                    diagnostics_store,
                    week_repository, // DuckDbWeekRepository implémente aussi AnalyticsRepository
                    runtime_config,
                ));

                app.manage(SharedState { service });
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                load_bootstrap,
                get_active_week,
                save_week,
                create_or_switch_week,
                list_weeks,
                delete_week,
                load_settings,
                save_settings,
                set_theme,
                get_app_status,
                export_data,
                import_data,
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
