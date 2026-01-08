#[macro_use]
extern crate rocket;
use anyhow::Result;
use figment::{
    Figment,
    providers::{Format, Json},
};
use redb::{ReadableTable, ReadableTableMetadata};
use rocket::data::{ByteUnit, Limits}; // Add ByteUnit import
use rocket::fs::FileServer;
use std::thread;
use std::time::Instant;

mod migration;
mod operations;
mod process;
mod public;
mod router;
mod tasks;
mod workflow;

use crate::process::initialization::initialize;
use crate::operations::initialization::logger::initialize_logger;
use crate::public::constant::runtime::{INDEX_RUNTIME, ROCKET_RUNTIME};

use crate::public::error_data::handle_error;
use crate::public::structure::config::AppConfig;
use crate::public::tui::{DASHBOARD, tui_task};
use crate::tasks::BATCH_COORDINATOR;
use crate::tasks::batcher::start_watcher::StartWatcherTask;
use crate::tasks::batcher::update_tree::UpdateTreeTask;
use crate::tasks::looper::start_expire_check_loop;
use public::constant::redb::DATA_TABLE;
use public::db::tree::TREE;
use public::structure::abstract_data::AbstractData;
use router::fairing::cache_control_fairing::cache_control_fairing;
use router::fairing::generate_fairing_routes;
use router::{
    delete::generate_delete_routes, get::generate_get_routes, post::generate_post_routes,
    put::generate_put_routes,
}; // Add import for existing AppConfig

#[cfg(feature = "embed-frontend")]
#[get("/assets/<file..>")]
async fn assets(
    file: std::path::PathBuf,
) -> Option<(rocket::http::ContentType, std::borrow::Cow<'static, [u8]>)> {
    use crate::public::embedded::FrontendAssets;

    let filename = format!("assets/{}", file.display());
    let asset = FrontendAssets::get(&filename)?;
    let mime = mime_guess::from_path(&filename).first_or_octet_stream();

    // Fix: Convert mime_guess::Mime to Rocket ContentType
    let content_type = rocket::http::ContentType::parse_flexible(mime.as_ref())
        .unwrap_or(rocket::http::ContentType::Binary);

    Some((content_type, asset.data))
}

/// Configures the Rocket instance with routes, fairings, and file servers.
async fn build_rocket() -> rocket::Rocket<rocket::Build> {
    // Modified: Load config.json into Figment for extraction
    let figment = Figment::new().merge(Json::file("config.json"));

    // New: Extract and validate config with type checking
    let app_config: AppConfig = figment
        .extract()
        .expect("config.json format error or type mismatch");

    // New: Convert human-readable limits to Rocket's Limits (using HashMap from config.rs)
    let get_limit = |key: &str, default: &str| -> ByteUnit {
        let val = app_config
            .public
            .limits
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(default);
        parse_limit(val)
    };

    let limits = Limits::default()
        .limit("form", get_limit("data-form", "10GiB"))
        .limit("file", get_limit("file", "10GiB"))
        .limit("json", get_limit("json", "10MiB"));

    // New: Build Rocket config manually with extracted values
    let rocket_config = rocket::Config::figment()
        .merge(("address", &app_config.public.address))
        .merge(("port", app_config.public.port))
        .merge(("limits", limits));

    #[cfg(not(feature = "embed-frontend"))]
    let app = {
        // Modified: Use custom config and manage AppConfig as state
        // Determine asset path: local "./assets" (prod) or "../gallery-frontend/dist/assets" (dev)
        let prod_assets = std::path::Path::new("assets");
        let asset_path = if prod_assets.exists() {
            prod_assets.to_path_buf()
        } else {
            std::path::PathBuf::from("../gallery-frontend/dist/assets")
        };

        info!("Serving assets from: {:?}", asset_path);

        rocket::custom(rocket_config)
            .manage(app_config)
            .attach(cache_control_fairing())
            .mount("/assets", FileServer::from(asset_path))
    };

    #[cfg(feature = "embed-frontend")]
    let app = {
        info!("Serving assets from embedded binary");
        rocket::custom(rocket_config)
            .manage(app_config)
            .attach(cache_control_fairing())
            .mount("/", routes![assets])
    };

    app.mount("/", generate_get_routes())
        .mount("/", generate_post_routes())
        .mount("/", generate_put_routes())
        .mount("/", generate_delete_routes())
        .mount("/", generate_fairing_routes())
}

// New: Helper function to parse limit strings (e.g., "10GiB") to bytes
fn parse_limit(s: &str) -> ByteUnit {
    let bytes = if let Some(gib) = s.strip_suffix("GiB") {
        gib.parse::<u64>().unwrap_or(10) * 1024 * 1024 * 1024
    } else if let Some(mib) = s.strip_suffix("MiB") {
        mib.parse::<u64>().unwrap_or(10) * 1024 * 1024
    } else {
        s.parse::<u64>().unwrap_or(10 * 1024 * 1024 * 1024) // Default to 10GiB
    };
    ByteUnit::from(bytes)
}

fn main() -> Result<()> {
    // Initialize logger first thing
    let tui_events_rx = initialize_logger();

    if let Err(e) = migration::migrate() {
        error!("Migration failed: {:?}\nCheck logs above.", e);
        std::process::exit(1);
    }

    // Initialize core subsystems (Config, DB, FFmpeg checks)
    initialize();

    #[cfg(feature = "embed-frontend")]
    info!("Frontend Configuration: EMBEDDED (Assets compiled into binary)");
    #[cfg(not(feature = "embed-frontend"))]
    info!("Frontend Configuration: EXTERNAL (Loading from file system)");

    // Architecture: Isolate the Indexing/TUI runtime from the Rocket server runtime.


    // This prevents heavy blocking operations in the indexer from stalling web requests.
    let worker_handle = thread::spawn(move || {
        INDEX_RUNTIME.block_on(async {
            let start_time = Instant::now();
            let txn = TREE.in_disk.begin_write().unwrap();

            {
                let table = txn.open_table(DATA_TABLE).unwrap();
                let total_count = table.len().unwrap();

                // Constraint: DATA_TABLE stores mixed types (Albums and Media).
                // We must perform an O(N) scan to differentiate counts.
                let album_count = table
                    .iter()
                    .unwrap()
                    .filter_map(|entry| entry.ok())
                    .filter(|(_, guard)| matches!(guard.value(), AbstractData::Album(_)))
                    .count();

                let media_count = total_count as usize - album_count;

                info!(
                    duration = &*format!("{:?}", start_time.elapsed());
                    "Read {} photos/videos and {} albums from database.",
                    media_count, album_count
                );
            }

            txn.commit().unwrap();

            BATCH_COORDINATOR.execute_batch_detached(StartWatcherTask);
            BATCH_COORDINATOR.execute_batch_detached(UpdateTreeTask);
            start_expire_check_loop();

            if let Some(console) = superconsole::SuperConsole::new() {
                INDEX_RUNTIME.spawn(async move {
                    if let Err(e) = tui_task(console, DASHBOARD.clone(), tui_events_rx)
                        .await
                        .map_err(|error| handle_error(error.context("TUI error.")))
                    {
                        panic!("TUI error: {e:?}");
                    }
                });
            } else {
                error!("Superconsole disabled (no TTY)");
            }

            if let Err(e) = tokio::signal::ctrl_c().await {
                error!("Failed to listen for ctrl-c in worker: {}", e);
            }
            info!("Worker thread shutting down.");
        });
    });

    let rocket_handle = thread::spawn(|| {
        info!("Rocket thread starting.");
        if let Err(e) = ROCKET_RUNTIME.block_on(async {
            let rocket = build_rocket().await.ignite().await?;
            let port = rocket.config().port;
            let shutdown_handle = rocket.shutdown();

            // Manually handle Ctrl-C to trigger graceful shutdown
            // since we are running outside the default global runtime.
            ROCKET_RUNTIME.spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                info!("Ctrl-C received, shutting down Rocket server gracefully.");
                shutdown_handle.notify();
            });

            // Open browser after server starts listening
            let launch_future = rocket.launch();
            open_browser(port);
            launch_future.await.map_err(anyhow::Error::from)
        }) {
            error!("Rocket thread exited with an error: {}", e);
        }
    });

    worker_handle.join().expect("Worker thread panicked");
    rocket_handle.join().expect("Rocket thread panicked");

    Ok(())
}

fn open_browser(port: u16) {
    let url = format!("http://localhost:{}", port);
    info!("Opening browser at {}", url);
    if let Err(e) = webbrowser::open(&url) {
        error!("Failed to open browser: {}", e);
    }
}
