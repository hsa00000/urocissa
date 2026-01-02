use crate::public::constant::runtime::INDEX_RUNTIME;
use crate::public::constant::{VALID_IMAGE_EXTENSIONS, VALID_VIDEO_EXTENSIONS};
use crate::public::structure::config::APP_CONFIG;
use crate::{public::error_data::handle_error, workflow::index_for_watch};
use anyhow::Result;
use log::{error, info};
use mini_executor::BatchTask;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use tokio::time::{Duration, sleep};
use walkdir::WalkDir;

static IS_WATCHING: AtomicBool = AtomicBool::new(false);

static WATCHER_HANDLE: LazyLock<Mutex<Option<RecommendedWatcher>>> =
    LazyLock::new(|| Mutex::new(None));

/// The last trigger time for each path
static DEBOUNCE_POOL: LazyLock<Mutex<HashMap<PathBuf, Instant>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct StartWatcherTask;

impl BatchTask for StartWatcherTask {
    fn batch_run(_: Vec<Self>) -> impl Future<Output = ()> + Send {
        async move {
            if let Err(e) = start_watcher_task_internal() {
                handle_error(e);
            }
        }
    }
}

/// Reload watcher with new paths from config
pub fn reload_watcher() {
    info!("Reloading watcher...");

    {
        let mut guard = WATCHER_HANDLE.lock().unwrap();
        *guard = None; // Drop old watcher
    }

    // Reset the flag so we can start again
    IS_WATCHING.store(false, Ordering::SeqCst);

    if let Err(e) = start_watcher_task_internal() {
        error!("Failed to reload watcher: {}", e);
    }
}

/// Resolve relative paths in the config file to absolute paths
/// If UROCISSA_PATH is set, use it as the base, otherwise use the current working directory as the base
fn resolve_sync_paths(paths: HashSet<PathBuf>) -> HashSet<PathBuf> {
    let (base_path, append_subdir) = match std::env::var("UROCISSA_PATH") {
        Ok(p) => (PathBuf::from(p), true),
        Err(_) => (std::env::current_dir().unwrap_or_default(), false),
    };

    paths
        .into_iter()
        .map(|p| {
            if p.is_relative() {
                let mut resolved = base_path.clone();
                if append_subdir {
                    resolved.push("gallery-backend");
                }
                resolved.push(&p);
                // Try to canonicalize (normalize . and ..), fall back to resolved path
                resolved.canonicalize().unwrap_or(resolved)
            } else {
                p
            }
        })
        .collect()
}

fn start_watcher_task_internal() -> Result<()> {
    // Fast-path: already running.
    if IS_WATCHING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    // Get raw paths from config system
    let raw_sync_paths = APP_CONFIG
        .get()
        .unwrap()
        .read()
        .unwrap()
        .public
        .sync_paths
        .clone();

    if raw_sync_paths.is_empty() {
        info!("No paths to watch");
        return Ok(());
    }

    // Resolve paths to absolute paths before watching
    let sync_paths = resolve_sync_paths(raw_sync_paths);

    // Build the watcher.
    let mut watcher = new_watcher()?;
    for path in &sync_paths {
        if path.exists() {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| anyhow::anyhow!("Failed to watch path {:?}: {}", path, e))?;
            info!("Watching path {:?}", path);
        } else {
            error!("Path not found, skipped: {:?}", path);
        }
    }

    // Store it globally to keep it alive.
    *WATCHER_HANDLE.lock().unwrap() = Some(watcher);
    Ok(())
}

fn is_valid_media_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .map(|ext| {
            VALID_IMAGE_EXTENSIONS.contains(&ext.as_str())
                || VALID_VIDEO_EXTENSIONS.contains(&ext.as_str())
        })
        .unwrap_or(false)
}

fn submit_to_debounce_pool(path: PathBuf) {
    let now = Instant::now();

    {
        let mut pool = DEBOUNCE_POOL.lock().unwrap();
        pool.insert(path.clone(), now);
    }

    // Start a task to check after 1 second (running on INDEX_RUNTIME)
    INDEX_RUNTIME.spawn(async move {
        sleep(Duration::from_secs(1)).await;

        // Check if there are any events for the same path within this 1 second (i.e., whether the last time is still now)
        let should_run = {
            let mut pool = DEBOUNCE_POOL.lock().unwrap();
            match pool.get(&path).copied() {
                Some(last) if last == now => {
                    // Not updated, remove and execute
                    pool.remove(&path);
                    true
                }
                _ => false, // There are later events or it has been removed, abandon this time
            }
        };

        if should_run && is_valid_media_file(&path) {
            // Really need to do indexing
            if let Err(e) = index_for_watch(path, None).await {
                handle_error(e);
            }
        }
    });
}

fn new_watcher() -> Result<RecommendedWatcher> {
    notify::recommended_watcher(move |result: Result<Event, notify::Error>| match result {
        Ok(event) => {
            match event.kind {
                EventKind::Create(_) => {
                    let mut path_list: HashSet<PathBuf> = HashSet::new();

                    for path in event.paths {
                        if path.is_file() {
                            path_list.insert(path);
                        } else if path.is_dir() {
                            WalkDir::new(&path)
                                .into_iter()
                                .filter_map(|dir_entry| dir_entry.ok())
                                .filter(|dir_entry| dir_entry.file_type().is_file())
                                .for_each(|dir_entry| {
                                    path_list.insert(dir_entry.into_path());
                                });
                        }
                    }

                    for path in path_list {
                        if is_valid_media_file(&path) {
                            submit_to_debounce_pool(path);
                        }
                    }
                }

                EventKind::Modify(_) => {
                    let mut path_list: HashSet<PathBuf> = HashSet::new();

                    for path in event.paths {
                        if path.is_file() {
                            path_list.insert(path);
                        }
                    }

                    for path in path_list {
                        if is_valid_media_file(&path) {
                            submit_to_debounce_pool(path);
                        }
                    }
                }

                _ => { /* ignore other kinds */ }
            }
        }
        Err(err) => {
            handle_error(anyhow::anyhow!("Watch error: {:#?}", err));
        }
    })
    .map_err(|e| anyhow::anyhow!("Failed to create watcher: {}", e))
}
