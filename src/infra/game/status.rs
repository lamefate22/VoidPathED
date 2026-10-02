use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::contract::StatusWatcher;
use crate::domain::event::ShipStatus;
use crate::error::CoreError;

pub struct GameStatusWatcher {
    is_running: Arc<AtomicBool>,
    status_file: PathBuf,
    current_status: Arc<Mutex<Option<ShipStatus>>>,
}

impl GameStatusWatcher {
    pub fn new(ed_dir: PathBuf) -> Self {
        let status_file = ed_dir.join("Status.json");
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            status_file,
            current_status: Arc::new(Mutex::new(None)),
        }
    }

    pub fn parse_status_file(path: &std::path::Path) -> Option<ShipStatus> {
        let content = std::fs::read_to_string(path).ok()?;
        let json: Value = serde_json::from_str(&content).ok()?;
        let flags = json.get("Flags")?.as_u64()?;
        Some(ShipStatus::from_flags(flags))
    }
}

impl StatusWatcher for GameStatusWatcher {
    fn start(
        &self,
        callback: Box<dyn Fn(ShipStatus) + Send + Sync + 'static>,
    ) -> Result<(), CoreError> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let is_running = Arc::clone(&self.is_running);
        let path = self.status_file.clone();
        let current_state = Arc::clone(&self.current_status);
        let callback = Arc::new(callback);

        std::thread::spawn(move || {
            tracing::info!("Started ED Status.json watcher for {:?}", path);
            let mut last_modified = None;

            while is_running.load(Ordering::SeqCst) {
                if let Ok(metadata) = std::fs::metadata(&path)
                    && let Ok(modified) = metadata.modified()
                    && last_modified != Some(modified)
                {
                    last_modified = Some(modified);
                    if let Some(status) = Self::parse_status_file(&path) {
                        let mut guard = current_state.lock().unwrap_or_else(|p| p.into_inner());
                        if *guard != Some(status) {
                            *guard = Some(status);
                            callback(status);
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(500));
            }

            tracing::info!("ED Status.json watcher stopped");
        });

        Ok(())
    }

    fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    fn current_status(&self) -> Option<ShipStatus> {
        let guard = self
            .current_status
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        *guard
    }
}
