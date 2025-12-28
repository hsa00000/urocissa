use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::RwLock;

use crate::public::structure::config::{APP_CONFIG, AppConfig};
use anyhow::Context;

const CONFIG_FILE: &str = "config.json";

/// 初始化設定系統
pub fn init_config() {
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
        let cfg = crate::migration::construct_migrated_config();

        // 2. 將完整的設定 (包含 Rocket 預設值) 寫入新格式
        if let Err(e) = save_config_update(&cfg) {
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
        load_config_from_file()
    };

    APP_CONFIG
        .set(RwLock::new(config))
        .expect("Config already initialized");
}

fn load_config_from_file() -> AppConfig {
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
                // 這裡使用了 {:?} 來顯示更詳細的錯誤，幫助 debug
                println!(
                    "Failed to deserialize 'default' section from {}: {:?}, using defaults",
                    CONFIG_FILE, e
                );
                AppConfig::default()
            }
        }
    } else {
        // 理論上 init_config 已經擋掉了這種情況，但為了安全起見
        println!("Config file format is invalid (missing 'default' section), using defaults");
        AppConfig::default()
    }
}

pub fn update_config(new_config: AppConfig) -> anyhow::Result<()> {
    use crate::tasks::batcher::start_watcher::reload_watcher;

    println!("Updating configuration...");

    // 1. 寫入檔案
    save_config_update(&new_config).context("Failed to save configuration to file")?;

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

fn save_config_update(config: &AppConfig) -> anyhow::Result<()> {
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
