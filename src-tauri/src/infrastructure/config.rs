use std::path::PathBuf;

use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize)]
pub struct AppRuntimeConfig {
    pub database_path: PathBuf,
    pub app_name: String,
    pub version: String,
    pub bundle_id: String,
    pub schema_version: u32,
    pub config_checksum: String,
}

impl AppRuntimeConfig {
    pub fn new(
        database_path: PathBuf,
        app_name: &str,
        version: &str,
        bundle_id: &str,
        schema_version: u32,
    ) -> Self {
        let seed = serde_json::json!({
            "database_path": database_path,
            "app_name": app_name,
            "version": version,
            "bundle_id": bundle_id,
            "schema_version": schema_version
        });

        let mut hasher = Sha256::new();
        hasher.update(seed.to_string().as_bytes());
        let config_checksum = format!("{:x}", hasher.finalize());

        Self {
            database_path,
            app_name: app_name.to_string(),
            version: version.to_string(),
            bundle_id: bundle_id.to_string(),
            schema_version,
            config_checksum,
        }
    }
}
