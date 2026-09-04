pub mod application;
pub mod domain;
pub mod infrastructure;

#[cfg(feature = "desktop")]
mod desktop {
    use std::{fs, path::PathBuf, sync::Arc, sync::mpsc};

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

    #[tauri::command]
    async fn export_week(
        app: tauri::AppHandle,
        state: tauri::State<'_, SharedState>,
        week_start: String,
    ) -> Result<Option<String>, PublicError> {
        use tauri::async_runtime::spawn_blocking;
        use tauri_plugin_dialog::DialogExt;

        let exported = state
            .service
            .export_week(week_start)
            .map_err(|error| to_public_error("export_week", error))?;

        // Le recv bloquant vit sur un thread dédié : le dialogue natif pompe
        // la message loop du thread principal, une commande sync qui attend
        // rx.recv() gèle l'app entière (freeze constaté en E2E).
        let result = spawn_blocking(move || -> Result<Option<String>, PublicError> {
            let (tx, rx) = mpsc::channel();
            app.dialog()
                .file()
                .set_file_name(&exported.file_name)
                .add_filter("Classeur Excel", &["xlsx"])
                .save_file(move |path| {
                    let _ = tx.send(path);
                });

            let picked = rx.recv().map_err(|_| PublicError {
                message: "dialogue d'enregistrement interrompu".to_string(),
            })?;

            let Some(file_path) = picked else {
                return Ok(None);
            };

            let path = file_path.into_path().map_err(|error| PublicError {
                message: format!("chemin invalide: {error:?}"),
            })?;

            fs::write(&path, &exported.bytes).map_err(|error| PublicError {
                message: format!("écriture impossible: {error}"),
            })?;

            Ok(Some(path.to_string_lossy().into_owned()))
        })
        .await
        .map_err(|error| PublicError {
            message: format!("tâche d'export interrompue: {error}"),
        })??;

        Ok(result)
    }

    #[tauri::command]
    async fn export_all_weeks(
        app: tauri::AppHandle,
        state: tauri::State<'_, SharedState>,
    ) -> Result<Option<String>, PublicError> {
        use tauri::async_runtime::spawn_blocking;
        use tauri_plugin_dialog::DialogExt;

        let exported = state
            .service
            .export_all_weeks()
            .map_err(|error| to_public_error("export_all_weeks", error))?;

        let result = spawn_blocking(move || -> Result<Option<String>, PublicError> {
            let (tx, rx) = std::sync::mpsc::channel();
            app.dialog()
                .file()
                .set_file_name(&exported.file_name)
                .add_filter("Classeur Excel", &["xlsx"])
                .save_file(move |path| {
                    let _ = tx.send(path);
                });

            let picked = rx.recv().map_err(|_| PublicError {
                message: "dialogue d'enregistrement interrompu".to_string(),
            })?;

            let Some(file_path) = picked else {
                return Ok(None);
            };

            let path = file_path.into_path().map_err(|error| PublicError {
                message: format!("chemin invalide: {error:?}"),
            })?;

            fs::write(&path, &exported.bytes).map_err(|error| PublicError {
                message: format!("écriture impossible: {error}"),
            })?;

            Ok(Some(path.to_string_lossy().into_owned()))
        })
        .await
        .map_err(|error| PublicError {
            message: format!("tâche d'export interrompue: {error}"),
        })??;

        Ok(result)
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
            .plugin(tauri_plugin_dialog::init())
            .invoke_handler(tauri::generate_handler![
                load_bootstrap,
                save_week,
                create_or_switch_week,
                list_weeks,
                delete_week,
                load_settings,
                save_settings,
                set_theme,
                get_analytics,
                export_week,
                export_all_weeks
            ])
            .run(tauri::generate_context!())
            .expect("tauri application failed to run");
    }
}

#[cfg(feature = "desktop")]
pub use desktop::run;

#[cfg(not(feature = "desktop"))]
pub fn run() {}
