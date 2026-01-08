use crate::public::constant::storage::get_data_path;
use log::{error, info};
use std::env;
use std::process::Command;

pub fn check_ffmpeg_and_ffprobe() {
    let root = get_data_path();
    let bin_dir = root.join("bin");

    // Also check the executable directory's bin folder (for installed version)
    let mut bin_dirs = vec![bin_dir];
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
             bin_dirs.push(exe_dir.join("bin"));
        }
    }

    // Prepend local bin directories to PATH for this process
    if let Some(path) = env::var_os("PATH") {
        let mut paths = env::split_paths(&path).collect::<Vec<_>>();
        for dir in &bin_dirs {
            if dir.exists() {
                paths.insert(0, dir.clone());
                info!("Added {:?} to PATH", dir);
            }
        }
        let new_path = env::join_paths(paths).unwrap();
        
        // SAFETY: Modifying PATH env var is generally unsafe in multi-threaded context,
        // but this is initialization code running before any threads are spawned.
        unsafe {
            env::set_var("PATH", new_path);
        }
    }


    for command in &["ffmpeg", "ffprobe"] {
        match Command::new(command).arg("-version").output() {
            Ok(output) if output.status.success() => {
                let version_info = String::from_utf8_lossy(&output.stdout);
                let version_number = version_info
                    .lines()
                    .next()
                    .unwrap_or("Unknown version")
                    .split_whitespace()
                    .nth(2) // Get the third word
                    .unwrap_or("Unknown");
                info!("{} version: {}", command, version_number);
            }
            Ok(_) => {
                error!(
                    "`{}` command was found, but it returned an error. Please ensure it's correctly installed.",
                    command
                );
            }
            Err(_) => {
                error!(
                    "`{}` is not installed or not available in PATH. Please install it before running the application.",
                    command
                );
            }
        }
    }
}
