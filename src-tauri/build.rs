fn main() {
    // Prevent console window from appearing on Windows in release builds
    #[cfg(windows)]
    #[cfg(not(debug_assertions))]
    {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }

    if std::env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        tauri_build::build();
    }
}
