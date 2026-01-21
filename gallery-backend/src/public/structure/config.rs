// src/public/structure/config.rs

use anyhow::Context;
use base64::{Engine as _, engine::general_purpose};
use log::{info, warn};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use crate::public::constant::storage::get_config_path;

pub static FALLBACK_SECRET_KEY: OnceLock<String> = OnceLock::new();

fn generate_secret_key() -> String {
    let mut secret = vec![0u8; 32];
    OsRng
        .try_fill_bytes(&mut secret)
        .expect("Failed to generate random secret key");
    general_purpose::STANDARD.encode(secret)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicConfig {
    pub address: String,
    pub port: u16,
    pub limits: HashMap<String, String>,
    pub sync_paths: HashSet<PathBuf>,
    pub read_only_mode: bool,
    pub disable_img: bool,
    #[serde(default)]
    pub local_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivateConfig {
    pub password: Option<String>,
    pub auth_key: Option<String>,
    pub discord_hook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub public: PublicConfig,
    pub private: PrivateConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut limits = HashMap::new();
        limits.insert("json".to_string(), "10MiB".to_string());
        limits.insert("file".to_string(), "10GiB".to_string());
        limits.insert("data-form".to_string(), "10GiB".to_string());

        Self {
            public: PublicConfig {
                address: "0.0.0.0".to_string(),
                port: 5673,
                limits,
                sync_paths: HashSet::new(),
                read_only_mode: false,
                disable_img: false,
                local_mode: false,
            },

            private: PrivateConfig {
                password: None,
                auth_key: None,
                discord_hook_url: None,
            },
        }
    }
}

pub static APP_CONFIG: OnceLock<RwLock<AppConfig>> = OnceLock::new();

impl AppConfig {
    pub fn get_jwt_secret_key(&self) -> Vec<u8> {
        match self.private.auth_key.as_ref() {
            Some(auth_key) => auth_key.as_bytes().to_vec(),
            None => FALLBACK_SECRET_KEY
                .get_or_init(generate_secret_key)
                .as_bytes()
                .to_vec(),
        }
    }

    pub fn init() {
        let config_path = get_config_path();
        let config_path_display = config_path.display();

        // Create default config file if it doesn't exist
        if !config_path.exists() {
            info!(
                "Configuration file not found at {config_path_display}. Creating default config.json..."
            );
            let default_config = AppConfig::default();

            if let Err(e) = Self::save_update(&default_config) {
                warn!("Failed to create default config file: {e}");
            } else {
                info!("Default configuration created successfully.");
            }
        }

        info!("Loading configuration from {config_path_display}");
        let mut config = Self::load_from_file();

        if config
            .private
            .auth_key
            .as_ref()
            .filter(|k| !k.is_empty())
            .is_none()
        {
            config.private.auth_key = None;
            FALLBACK_SECRET_KEY.get_or_init(generate_secret_key);
        }

        APP_CONFIG
            .set(RwLock::new(config))
            .expect("Config already initialized");
    }

    fn load_from_file() -> AppConfig {
        let config_path = get_config_path();
        let config_path_display = config_path.display();

        let file_content = fs::read_to_string(&config_path).unwrap_or_else(|e| {
            warn!("Failed to read config file {config_path_display}: {e}, using defaults");
            "{}".to_string()
        });

        match serde_json::from_str::<AppConfig>(&file_content) {
            Ok(config) => {
                info!("Successfully loaded configuration from {config_path_display}");
                config
            }
            Err(e) => {
                warn!(
                    "Failed to deserialize config from {config_path_display}: {e:?}, using defaults"
                );
                AppConfig::default()
            }
        }
    }

    pub fn update(mut new_config: AppConfig) -> anyhow::Result<()> {
        use crate::tasks::batcher::start_watcher::reload_watcher;

        info!("Updating configuration...");

        // Sanitize paths: only remove quotes and spaces, do not resolve paths
        let sanitized_paths: HashSet<PathBuf> = new_config
            .public
            .sync_paths
            .iter()
            .map(|p| PathBuf::from(p.to_string_lossy().trim().trim_matches('"')))
            .collect();

        new_config.public.sync_paths = sanitized_paths;

        if new_config
            .private
            .auth_key
            .as_ref()
            .filter(|k| !k.is_empty())
            .is_none()
        {
            new_config.private.auth_key = None;
        }

        Self::save_update(&new_config).context("Failed to save configuration to file")?;

        {
            let mut w = APP_CONFIG.get().unwrap().write().unwrap();
            if new_config.private.auth_key.is_none() {
                FALLBACK_SECRET_KEY.get_or_init(generate_secret_key);
            }
            *w = new_config.clone();
        }

        reload_watcher();
        info!("Configuration updated successfully");
        Ok(())
    }

    fn save_update(config: &AppConfig) -> anyhow::Result<()> {
        let config_path = get_config_path();
        let config_path_display = config_path.display();

        let mut file = File::create(&config_path).context(format!(
            "Failed to create config file {config_path_display}"
        ))?;

        let pretty_json = serde_json::to_string_pretty(config)
            .context("Failed to serialize configuration to JSON")?;

        file.write_all(pretty_json.as_bytes()).context(format!(
            "Failed to write configuration to {config_path_display}"
        ))?;

        Ok(())
    }
}
