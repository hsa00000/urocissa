use crate::public::structure::config::AppConfig;
use dotenv::dotenv;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

/// Reads legacy configuration sources (config.json, .env, env vars) and constructs
/// the new nested  ( + ).
pub fn construct_migrated_config() -> AppConfig {
    let mut config = AppConfig::default();

    // 1. Migrate from old 
    // Legacy files were flat. We map recognized fields to .
    if let Ok(file) = File::open("config.json") {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyConfigJson {
            #[serde(default)]
            read_only_mode: bool,
            #[serde(default)]
            disable_img: bool,
            // Add other legacy fields here if they existed in the old json
        }

        if let Ok(old) = serde_json::from_reader::<_, LegacyConfigJson>(file) {
            config.public.read_only_mode = old.read_only_mode;
            config.public.disable_img = old.disable_img;
            println!("Migrated settings from legacy config.json into PublicConfig");
        }
    }

    // 1b. Migrate from Rocket.toml (legacy Rocket configuration)
    if let Ok(toml_content) = fs::read_to_string("Rocket.toml")
        && let Ok(toml_value) = toml_content.parse::<toml::Table>()
    {
        // Try to read from [default] section first, then root level
        let default_section = toml_value.get("default").and_then(|v| v.as_table());

        // Read port
        let port = default_section
            .and_then(|d| d.get("port"))
            .or_else(|| toml_value.get("port"))
            .and_then(toml::Value::as_integer)
            .map(|p| u16::try_from(p).unwrap_or(8000));

        if let Some(p) = port {
            config.public.port = p;
            println!("Migrated port {p} from Rocket.toml into PublicConfig");
        }

        // Read address
        let address = default_section
            .and_then(|d| d.get("address"))
            .or_else(|| toml_value.get("address"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        if let Some(addr) = address {
            config.public.address.clone_from(&addr);
            println!(
                "Migrated address {addr} from Rocket.toml into PublicConfig"
            );
        }
    }

    // 2. Migrate from Environment Variables (.env)
    dotenv().ok();

    if let Ok(pwd) = std::env::var("PASSWORD")
        && !pwd.trim().is_empty()
    {
        // Sensitive -> PrivateConfig
        config.private.password = pwd;
        println!("Migrated PASSWORD from environment into PrivateConfig");
    }

    if let Ok(key) = std::env::var("AUTH_KEY")
        && !key.trim().is_empty()
    {
        // Sensitive -> PrivateConfig
        config.private.auth_key = Some(key);
        println!("Migrated AUTH_KEY from environment into PrivateConfig");
    }

    if let Ok(hook) = std::env::var("DISCORD_HOOK_URL")
        && !hook.trim().is_empty()
    {
        // Non-sensitive -> PublicConfig
        config.public.discord_hook_url = Some(hook);
        println!("Migrated DISCORD_HOOK_URL from environment into PublicConfig");
    }

    // 3. Migrate Sync Paths
    if let Ok(sync_paths_str) = std::env::var("SYNC_PATH") {
        let mut count = 0;
        for path_str in sync_paths_str.split(',') {
            let path_str = path_str.trim();
            if !path_str.is_empty() {
                config.public.sync_paths.insert(PathBuf::from(path_str));
                count += 1;
            }
        }
        if count > 0 {
            println!(
                "Migrated {count} sync paths from SYNC_PATH into PublicConfig"
            );
        }
    }

    // Constraint: Prevent recursive syncing.
    if let Ok(upload_path) = fs::canonicalize(PathBuf::from("./upload")) {
        config
            .public
            .sync_paths
            .retain(|p| match fs::canonicalize(p) {
                Ok(c) => c != upload_path,
                Err(_) => p != &upload_path,
            });
    }

    config
}

/// Removes legacy configuration files (.env, Rocket.toml) to finalize migration.
pub fn cleanup_legacy_config_files() {
    if Path::new(".env").exists() {
        if let Err(e) = fs::remove_file(".env") {
            eprintln!("Failed to remove legacy .env file: {e}");
        } else {
            println!("Removed legacy .env file");
        }
    }

    if Path::new("Rocket.toml").exists() {
        if let Err(e) = fs::remove_file("Rocket.toml") {
            eprintln!("Failed to remove legacy Rocket.toml file: {e}");
        } else {
            println!("Removed legacy Rocket.toml file");
        }
    }
}
