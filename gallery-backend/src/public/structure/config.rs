// src/public/structure/config.rs

use anyhow::Context;
use base64::{Engine as _, engine::general_purpose};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

const CONFIG_FILE: &str = "config.json";

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
    pub discord_hook_url: Option<String>,
    pub read_only_mode: bool,
    pub disable_img: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivateConfig {
    pub password: String,
    pub auth_key: Option<String>,
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
                discord_hook_url: None,
                read_only_mode: false,
                disable_img: false,
            },

            private: PrivateConfig {
                password: "password".to_string(),
                auth_key: None,
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
        let should_migrate = if Path::new(CONFIG_FILE).exists() {
            match fs::read_to_string(CONFIG_FILE) {
                Ok(content) => serde_json::from_str::<AppConfig>(&content).is_err(),
                Err(_) => true,
            }
        } else {
            true
        };

        let mut config = if should_migrate {
            println!("Legacy configuration or missing file detected. Starting migration...");
            let cfg = crate::migration::construct_migrated_config();

            if let Err(e) = Self::save_update(&cfg) {
                eprintln!("Warning: Failed to save migrated config: {}", e);
            } else {
                crate::migration::cleanup_legacy_config_files();
            }
            println!(
                "Migration completed. New configuration saved to {}",
                CONFIG_FILE
            );
            cfg
        } else {
            println!("Loading configuration from {}", CONFIG_FILE);
            Self::load_from_file()
        };

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
        let file_content = fs::read_to_string(CONFIG_FILE).unwrap_or_else(|e| {
            println!(
                "Failed to read config file {}: {}, using defaults",
                CONFIG_FILE, e
            );
            "{}".to_string()
        });

        match serde_json::from_str::<AppConfig>(&file_content) {
            Ok(config) => {
                println!("Successfully loaded configuration from {}", CONFIG_FILE);
                config
            }
            Err(e) => {
                println!(
                    "Failed to deserialize config from {}: {:?}, using defaults",
                    CONFIG_FILE, e
                );
                AppConfig::default()
            }
        }
    }

    pub fn update(mut new_config: AppConfig) -> anyhow::Result<()> {
        use crate::tasks::batcher::start_watcher::reload_watcher;

        println!("Updating configuration...");

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
        println!("Configuration updated successfully");
        Ok(())
    }

    fn save_update(config: &AppConfig) -> anyhow::Result<()> {
        let mut file = File::create(CONFIG_FILE)
            .context(format!("Failed to create config file {}", CONFIG_FILE))?;

        let pretty_json = serde_json::to_string_pretty(config)
            .context("Failed to serialize configuration to JSON")?;

        file.write_all(pretty_json.as_bytes())
            .context(format!("Failed to write configuration to {}", CONFIG_FILE))?;

        Ok(())
    }
}
