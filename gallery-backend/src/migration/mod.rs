#![allow(clippy::too_many_lines)]

//! Database Migration Module
//!
//! Handles the migration from redb 2.6.x (Old Schema) to redb 3.1.x (New `AbstractData` Schema).
//!
//! Refactored into:
//! - v2_schema: Legacy V2 structs
//! - v3_schema: Legacy V3 structs
//! - v2_v3: Transformation logic V2 -> V3
//! - v3_v4: Transformation logic V3 -> V4
//! - mod.rs: Orchestration

use anyhow::{Context, Result};
use dotenv::dotenv;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use redb::{
    ReadableDatabase, ReadableTable as NewReadableTable,
    ReadableTableMetadata as NewReadableTableMetadata,
};

pub mod v2_schema;
pub mod v2_v3;
pub mod v3_schema;
pub mod v3_v4;

use crate::public::constant::redb::DATA_TABLE;
use crate::public::structure::config::AppConfig;

use v2_schema::{Album as V2Album, Database as V2Database};
use v3_schema::AbstractData as V3AbstractData;

// ==================================================================================
// Migration Logic
// ==================================================================================

const OLD_DB_PATH: &str = "./db/index.redb";
const NEW_DB_PATH: &str = "./db/index_v4.redb";

enum MigrationType {
    None,
    V2ToV3, // Actually V2 -> V4 (via V3)
    V3ToV4,
}

/// Checks if a migration is required by attempting to open the DB with the new and old drivers.
fn needs_migration() -> MigrationType {
    // 1. Check if V4 DB already exists
    if Path::new(NEW_DB_PATH).exists() {
        println!("[INFO] Found V4 database at {NEW_DB_PATH}. No migration needed.");
        return MigrationType::None;
    }

    // 2. If V4 doesn't exist, check for legacy V3/V2 at OLD_DB_PATH
    if !Path::new(OLD_DB_PATH).exists() {
        return MigrationType::None;
    }

    // Check V3/V4 (Redb 3.x)
    let is_v4 = std::panic::catch_unwind(|| {
        if let Ok(db) = redb::Database::open(OLD_DB_PATH) {
            let read_txn = db.begin_read().unwrap();
            if let Ok(table) = read_txn.open_table(DATA_TABLE) {
                if table.len().unwrap() > 0 {
                    if let Some((_k, v)) = table.first().unwrap() {
                        let _ = v.value();
                    }
                }
                return true;
            }
        }
        false
    });

    if let Ok(true) = is_v4 {
        println!(
            "[WARN] Found V4-compatible database at {OLD_DB_PATH}. Moving to {NEW_DB_PATH}."
        );
        if let Err(e) = std::fs::rename(OLD_DB_PATH, NEW_DB_PATH) {
            eprintln!("[ERROR] Failed to move V4 database: {e}");
        }
        return MigrationType::None;
    }

    // 2. Check if it is V3 (Old Schema - Missing update_at)
    let is_v3 = std::panic::catch_unwind(|| {
        if let Ok(db) = redb::Database::open(OLD_DB_PATH) {
            let read_txn = db.begin_read().unwrap();
            let table_def = redb::TableDefinition::<&str, V3AbstractData>::new("database");
            if let Ok(table) = read_txn.open_table(table_def) {
                if table.len().unwrap() > 0 {
                    if let Some((_k, v)) = table.first().unwrap() {
                        let _ = v.value();
                    }
                }
                return true;
            }
        }
        false
    });

    if let Ok(true) = is_v3 {
        return MigrationType::V3ToV4;
    }

    // Check V2
    let result = std::panic::catch_unwind(|| redb_old::Database::open(OLD_DB_PATH));

    match result {
        Ok(Ok(db)) => {
            if let Ok(txn) = db.begin_read() {
                let has_db_table = txn
                    .open_table(redb_old::TableDefinition::<&str, V2Database>::new(
                        "database",
                    ))
                    .is_ok();
                let has_album_table = txn
                    .open_table(redb_old::TableDefinition::<&str, V2Album>::new("album"))
                    .is_ok();

                if has_db_table || has_album_table {
                    return MigrationType::V2ToV3;
                }
                MigrationType::None
            } else {
                MigrationType::None
            }
        }
        _ => MigrationType::None,
    }
}

/// Executes the migration from v2 to v4
pub fn migrate() -> Result<()> {
    let migration_type = needs_migration();
    if let MigrationType::None = migration_type {
        return Ok(());
    }

    println!("========================================================");
    match migration_type {
        MigrationType::V2ToV3 => {
            println!(" DETECTED OLD DATABASE (V2 / redb 2.6.x) at {OLD_DB_PATH}");
            println!(" A MIGRATION IS REQUIRED TO UPGRADE TO V4 (redb 3.1, schema V4)");
        }
        MigrationType::V3ToV4 => {
            println!(" DETECTED OLD SCHEMA (V3) at {OLD_DB_PATH}");
            println!(" A MIGRATION IS REQUIRED TO UPGRADE TO V4 (schema with update_at)");
        }
        MigrationType::None => {}
    }
    println!("========================================================");
    println!(" Please ensure you have BACKED UP your './db' folder.");
    println!("Type 'yes' to start migration:");

    let mut input = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;

    if input.trim() != "yes" {
        println!("Migration cancelled.");
        std::process::exit(0);
    }

    println!("Starting migration...");

    let new_db =
        redb::Database::create(NEW_DB_PATH).context("Failed to create new database file.")?;
    let write_txn = new_db.begin_write()?;

    match migration_type {
        MigrationType::V2ToV3 => {
            v2_v3::migrate_v2_to_v4(OLD_DB_PATH, &write_txn)?;
        }
        MigrationType::V3ToV4 => {
            v3_v4::migrate_v3_to_v4(OLD_DB_PATH, &write_txn)?;
        }
        MigrationType::None => unreachable!(),
    }

    write_txn.commit()?;
    println!("Migration completed successfully.");

    let backup_path = format!("{OLD_DB_PATH}.bak");
    std::fs::rename(OLD_DB_PATH, &backup_path)
        .context(format!("Failed to rename old DB to {backup_path}"))?;
    println!("Old database renamed to {backup_path}");

    println!("New database created at {NEW_DB_PATH}");

    Ok(())
}

// ==================================================================================
// Config Migration Logic
// ==================================================================================

pub fn construct_migrated_config() -> AppConfig {
    let mut config = AppConfig::default();

    // 1. Migrate from old `config.json`
    if let Ok(file) = File::open("config.json") {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyConfigJson {
            #[serde(default)]
            read_only_mode: bool,
            #[serde(default)]
            disable_img: bool,
        }

        if let Ok(old) = serde_json::from_reader::<_, LegacyConfigJson>(file) {
            config.public.read_only_mode = old.read_only_mode;
            config.public.disable_img = old.disable_img;
            println!("Migrated settings from legacy config.json into PublicConfig");
        }
    }

    // 1b. Migrate from Rocket.toml
    if let Ok(toml_content) = fs::read_to_string("Rocket.toml")
        && let Ok(toml_value) = toml_content.parse::<toml::Table>()
    {
        let default_section = toml_value.get("default").and_then(|v| v.as_table());

        let port = default_section
            .and_then(|d| d.get("port"))
            .or_else(|| toml_value.get("port"))
            .and_then(toml::Value::as_integer)
            .map(|p| u16::try_from(p).unwrap_or(8000));

        if let Some(p) = port {
            config.public.port = p;
            println!("Migrated port {p} from Rocket.toml into PublicConfig");
        }

        let address = default_section
            .and_then(|d| d.get("address"))
            .or_else(|| toml_value.get("address"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        if let Some(addr) = address {
            config.public.address.clone_from(&addr);
            println!("Migrated address {addr} from Rocket.toml into PublicConfig");
        }
    }

    // 2. Migrate from Environment Variables
    dotenv().ok();

    if let Ok(pwd) = std::env::var("PASSWORD")
        && !pwd.trim().is_empty()
    {
        config.private.password = Some(pwd);
        println!("Migrated PASSWORD from environment into PrivateConfig");
    }

    if let Ok(key) = std::env::var("AUTH_KEY")
        && !key.trim().is_empty()
    {
        config.private.auth_key = Some(key);
        println!("Migrated AUTH_KEY from environment into PrivateConfig");
    }

    if let Ok(hook) = std::env::var("DISCORD_HOOK_URL")
        && !hook.trim().is_empty()
    {
        config.public.discord_hook_url = Some(hook);
        println!("Migrated DISCORD_HOOK_URL from environment into PublicConfig");
    }

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
            println!("Migrated {count} sync paths from SYNC_PATH into PublicConfig");
        }
    }

    if let Ok(upload_path) = fs::canonicalize(PathBuf::from("./upload")) {
        config.public.sync_paths.retain(|p| match fs::canonicalize(p) {
            Ok(c) => c != upload_path,
            Err(_) => p != &upload_path,
        });
    }

    config
}

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
