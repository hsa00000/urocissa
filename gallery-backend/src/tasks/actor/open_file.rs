use crate::public::error_data::handle_error;
use anyhow::Result;
use mini_executor::Task;
use std::path::Path;
use std::{fs::File, path::PathBuf};
use tokio::task::spawn_blocking;

use anyhow::Error;
use log::warn;
use std::thread::sleep;
use std::time::Duration;

pub struct OpenFileTask {
    pub path: PathBuf,
}

impl OpenFileTask {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Task for OpenFileTask {
    type Output = Result<File>;

    async fn run(self) -> Self::Output {
        spawn_blocking(move || open_file_task(&self.path))
            .await
            .expect("blocking task panicked")
            .map_err(|err| handle_error(err.context("Failed to run hash task")))
    }
}

fn open_file_task(path: &Path) -> Result<File> {
    const OPEN_FAIL_RETRY: usize = 3;
    const OPEN_RETRY_DELAY_MS: u64 = 100;
    let mut delay = Duration::from_millis(OPEN_RETRY_DELAY_MS);

    for attempt in 0..=OPEN_FAIL_RETRY {
        match File::open(path) {
            Ok(file) => return Ok(file),
            Err(e) if attempt < OPEN_FAIL_RETRY => {
                warn!(
                    "Attempt {}/{} failed to open {}: {e}. Retrying in {delay:?}…",
                    attempt + 1,
                    OPEN_FAIL_RETRY + 1,
                    path.display()
                );
                sleep(delay);
                delay = delay.checked_mul(2).unwrap_or(delay);
            }
            Err(e) => {
                return Err(Error::new(e).context(format!(
                    "Failed to open file {} after {} attempts",
                    path.display(),
                    OPEN_FAIL_RETRY + 1
                )));
            }
        }
    }

    unreachable!("open_file_with_retry logic error")
}
