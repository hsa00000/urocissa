use anyhow::Context;
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock, RwLock};

const CONFIG_FILE: &str = "config.json";

// Refactor: Renamed AppSettings to AppConfig
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    // --- Rocket Settings ---
    pub address: String,
    pub port: u16,
    // 使用 HashMap<String, String> 來儲存 limits (例如 "json": "10MiB")
    pub limits: HashMap<String, String>,

    // --- App Settings ---
    /// 管理員密碼 (明文)
    pub password: String,
    /// 需要監聽/同步的資料夾路徑
    pub sync_paths: HashSet<PathBuf>,
    /// 驗證金鑰 (JWT Secret)
    /// 注意：如果從 JSON 讀取為 None，則會在記憶體中生成一次性的隨機金鑰
    pub auth_key: Option<String>,
    /// Discord Webhook URL
    pub discord_hook_url: Option<String>,
    /// 唯讀模式 (不允許上傳/刪除)
    pub read_only_mode: bool,
    /// 禁用圖片處理 (僅顯示檔案)
    pub disable_img: bool,
    /// 上傳檔案大小限制 (MB) - 這主要用於前端檢查，後端實際限制看 limits
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

static FALLBACK_SECRET_KEY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut secret = vec![0u8; 32];
    OsRng
        .try_fill_bytes(&mut secret)
        .expect("Failed to generate random secret key");
    secret
});

impl AppConfig {
    /// 獲取 JWT Secret Key (實例方法)
    /// 如果配置中沒有 auth_key，則使用記憶體中隨機生成的一次性金鑰
    pub fn get_jwt_secret_key(&self) -> Vec<u8> {
        match self.auth_key.as_ref() {
            Some(auth_key) => auth_key.as_bytes().to_vec(),
            None => FALLBACK_SECRET_KEY.clone(),
        }
    }

    /// 初始化設定系統
    pub fn init() {
        // 檢查是否需要遷移
        // 條件：檔案不存在，或是檔案存在但是是舊格式（沒有 "default" 鍵）
        let should_migrate = if Path::new(CONFIG_FILE).exists() {
            match fs::read_to_string(CONFIG_FILE) {
                Ok(content) => {
                    let json: Value = serde_json::from_str(&content).unwrap_or(json!({}));
                    // 如果沒有 "default" 鍵，代表是舊的 config.json，需要執行遷移流程
                    json.get("default").is_none()
                }
                Err(_) => true, // 讀取失敗當作不存在
            }
        } else {
            true // 檔案不存在
        };

        let config = if should_migrate {
            println!("Legacy configuration or missing file detected. Starting migration...");

            // 1. 執行遷移邏輯 (從 migration.rs)
            // 注意：這裡假設 migration 模組在 crate 根目錄下可用
            let cfg = crate::migration::construct_migrated_config();

            // 2. 將完整的設定 (包含 Rocket 預設值) 寫入新格式
            if let Err(e) = Self::save_update(&cfg) {
                eprintln!("Warning: Failed to save migrated config: {}", e);
            } else {
                // 3. 只有在成功儲存新設定後，才刪除舊檔案
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

        let json: Value = serde_json::from_str(&file_content).unwrap_or_else(|e| {
            println!(
                "Failed to parse config file {} as JSON: {}, using defaults",
                CONFIG_FILE, e
            );
            json!({})
        });

        // 只讀取 "default" 區塊
        if let Some(default_section) = json.get("default") {
            match serde_json::from_value(default_section.clone()) {
                Ok(config) => {
                    println!("Successfully loaded configuration from {}", CONFIG_FILE);
                    config
                }
                Err(e) => {
                    // 如果解析失敗（例如欄位型別錯誤），我們顯示錯誤並回退預設值
                    println!(
                        "Failed to deserialize 'default' section from {}: {:?}, using defaults",
                        CONFIG_FILE, e
                    );
                    AppConfig::default()
                }
            }
        } else {
            // 理論上 init 已經擋掉了這種情況，但為了安全起見
            println!("Config file format is invalid (missing 'default' section), using defaults");
            AppConfig::default()
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
            *w = new_config.clone();
        }

        // 3. 觸發副作用
        reload_watcher();

        println!("Configuration updated successfully");
        Ok(())
    }

    /// 將設定寫入檔案 (內部使用)
    fn save_update(config: &AppConfig) -> anyhow::Result<()> {
        // 構造 Rocket 需要的 profile 結構
        let final_json = json!({
            "default": config
        });

        let mut file = File::create(CONFIG_FILE)
            .context(format!("Failed to create config file {}", CONFIG_FILE))?;

        let pretty_json = serde_json::to_string_pretty(&final_json)
            .context("Failed to serialize configuration to JSON")?;

        file.write_all(pretty_json.as_bytes())
            .context(format!("Failed to write configuration to {}", CONFIG_FILE))?;

        Ok(())
    }
}
