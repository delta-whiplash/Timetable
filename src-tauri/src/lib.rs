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
            service::ApplicationService,
        },
        domain::errors::{ApplicationError, PublicError},
        infrastructure::{
            duckdb::DuckDb,
            tracing::init_tracing,
        },
    };

    pub struct SharedState {
        service: Arc<ApplicationService>,
    }

    fn to_public_error(context: &'static str, error: ApplicationError) -> PublicError {
        if !matches!(error, ApplicationError::Validation(_)) {
            tracing::error!(context, error = %error, "tauri command failed");
        }
        error.into()
    }

    macro_rules! tauri_command {
        ($name:ident, $output:ty, $method:ident $(, $arg_name:ident: $arg_ty:ty)?) => {
            #[tauri::command]
            fn $name(
                state: State<'_, SharedState>,
                $($arg_name: $arg_ty,)?
            ) -> Result<$output, PublicError> {
                state
                    .service
                    .$method($($arg_name,)?)
                    .map_err(|error| to_public_error(stringify!($name), error))
            }
        };
    }

    tauri_command!(load_bootstrap, BootstrapState, load_bootstrap);
    tauri_command!(save_week, WeekSheetView, save_week, input: SaveWeekInput);
    tauri_command!(
        create_or_switch_week,
        WeekSheetView,
        create_or_switch_week,
        input: WeekSelectorInput
    );
    tauri_command!(list_weeks, Vec<WeekListItem>, list_weeks);
    tauri_command!(delete_week, (), delete_week, input: DeleteWeekInput);
    tauri_command!(load_settings, SettingsView, load_settings);
    tauri_command!(save_settings, SettingsView, save_settings, input: SaveSettingsInput);
    tauri_command!(set_theme, (), set_theme, theme: String);
    tauri_command!(get_analytics, AnalyticsDataView, get_analytics);

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

                let store = Arc::new(DuckDb::new(app_data_dir.join("timetable.duckdb")));

                store
                    .migrate()
                    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
                store
                    .ensure_default_settings()
                    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;

                app.manage(SharedState {
                    service: Arc::new(ApplicationService::new(store)),
                });
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
