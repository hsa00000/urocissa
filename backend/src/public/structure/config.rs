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

// 新增：定義全域的 Fallback Key
pub static FALLBACK_SECRET_KEY: OnceLock<String> = OnceLock::new();

// Helper function to generate a random secret key string
fn generate_secret_key() -> String {
    let mut secret = vec![0u8; 32];
    OsRng
        .try_fill_bytes(&mut secret)
        .expect("Failed to generate random secret key");
    general_purpose::STANDARD.encode(secret)
}

// Refactor: Renamed AppSettings to AppConfig
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    // --- Rocket Settings ---
    pub address: String,
    pub port: u16,
    pub limits: HashMap<String, String>,

    // --- App Settings ---
    /// 管理員密碼 (明文)
    pub password: String,
    /// 需要監聽/同步的資料夾路徑
    pub sync_paths: HashSet<PathBuf>,
    /// 驗證金鑰 (JWT Secret)，若為 None 則使用隨機生成的 FALLBACK_SECRET_KEY
    pub auth_key: Option<String>,
    /// Discord Webhook URL
    pub discord_hook_url: Option<String>,
    /// 唯讀模式 (不允許上傳/刪除)
    pub read_only_mode: bool,
    /// 禁用圖片處理 (僅顯示檔案)
    pub disable_img: bool,
    /// 上傳檔案大小限制 (MB)
    pub upload_limit_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut limits = HashMap::new();
        limits.insert("json".to_string(), "10MiB".to_string());
        limits.insert("file".to_string(), "10GiB".to_string());
        limits.insert("data-form".to_string(), "10GiB".to_string());

        Self {
            address: "0.0.0.0".to_string(),
            port: 5673,
            limits,
            password: "password".to_string(),
            sync_paths: HashSet::new(),
            auth_key: None,
            discord_hook_url: None,
            read_only_mode: false,
            disable_img: false,
            upload_limit_mb: 2048,
        }
    }
}

pub static APP_CONFIG: OnceLock<RwLock<AppConfig>> = OnceLock::new();

impl AppConfig {
    /// 獲取 JWT Secret Key (實例方法)
    pub fn get_jwt_secret_key(&self) -> Vec<u8> {
        match self.auth_key.as_ref() {
            // 如果設定檔中有 Key，直接使用
            Some(auth_key) => auth_key.as_bytes().to_vec(),
            // 如果設定檔是 None (null)，使用全域的 Fallback Key
            None => {
                // 這裡使用 get_or_init 是為了雙重保險，
                // 正常流程下 init() 應該已經初始化過了。
                FALLBACK_SECRET_KEY
                    .get_or_init(generate_secret_key)
                    .as_bytes()
                    .to_vec()
            }
        }
    }

    /// 初始化設定系統
    pub fn init() {
        // 檢查是否需要遷移
        let should_migrate = if Path::new(CONFIG_FILE).exists() {
            match fs::read_to_string(CONFIG_FILE) {
                Ok(content) => serde_json::from_str::<AppConfig>(&content).is_err(),
                Err(_) => true,
            }
        } else {
            true
        };

        let config = if should_migrate {
            println!("Legacy configuration or missing file detected. Starting migration...");

            // 1. 執行遷移邏輯
            let cfg = crate::migration::construct_migrated_config();

            // 2. 將完整的設定寫入新格式
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

        // --- 關鍵邏輯：處理 Fallback Key ---
        // 檢查載入的設定中 auth_key 是否為 None
        if config.auth_key.is_none() {
            println!(
                "No authKey found in config (set to null). Initializing ephemeral fallback key."
            );
            // 初始化全域 Fallback Key，但不修改 config.auth_key
            FALLBACK_SECRET_KEY.get_or_init(generate_secret_key);
        }
        // --------------------------------

        println!("Configuration: {:?}", config);

        APP_CONFIG
            .set(RwLock::new(config))
            .expect("Config already initialized");
    }

    /// 從檔案載入設定 (內部使用)
    fn load_from_file() -> AppConfig {
        let file_content = fs::read_to_string(CONFIG_FILE).unwrap_or_else(|e| {
            println!(
                "Failed to read config file {}: {}, using defaults",
                CONFIG_FILE, e
            );
            "{}".to_string()
        });

        match serde_json::from_str(&file_content) {
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

    /// 更新設定並儲存
    pub fn update(new_config: AppConfig) -> anyhow::Result<()> {
        use crate::tasks::batcher::start_watcher::reload_watcher;

        println!("Updating configuration...");

        // 1. 寫入檔案
        Self::save_update(&new_config).context("Failed to save configuration to file")?;

        // 2. 更新記憶體
        {
            let mut w = APP_CONFIG.get().unwrap().write().unwrap();

            // 如果使用者更新配置時，特地將 auth_key 改為 None
            if new_config.auth_key.is_none() {
                // 確保 Fallback 有值 (雖然通常 init 時就有了，但以防萬一)
                FALLBACK_SECRET_KEY.get_or_init(generate_secret_key);
            }

            *w = new_config.clone();
        }

        // 3. 觸發副作用
        reload_watcher();

        println!("Configuration updated successfully");
        Ok(())
    }

    /// 將設定寫入檔案 (內部使用)
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
