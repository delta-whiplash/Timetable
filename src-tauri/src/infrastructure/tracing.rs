use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(debug_assertions)]
    {
        // Development mode: Log to console only
        let subscriber = fmt()
            .with_env_filter(env_filter)
            .with_target(false)
            .compact()
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    #[cfg(not(debug_assertions))]
    {
        // Release mode: Log to file only, no console output
        if let Ok(app_dir) = std::env::var("TAURI_APP_DIR") {
            let log_dir = std::path::PathBuf::from(app_dir).join("logs");
            let _ = std::fs::create_dir_all(&log_dir);

            let file_appender = tracing_appender::rolling::daily(&log_dir, "timetable.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            let subscriber = fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_writer(non_blocking)
                .finish();

            let _ = tracing::subscriber::set_global_default(subscriber);

            // Prevent console window from appearing by redirect stdout/stderr to nowhere
            let _ = std::io::sink();
        } else {
            // Fallback if TAURI_APP_DIR is not set (shouldn't happen in production)
            let subscriber = fmt()
                .with_env_filter(env_filter)
                .with_target(false)
                .compact()
                .finish();
            let _ = tracing::subscriber::set_global_default(subscriber);
        }
    }
}
