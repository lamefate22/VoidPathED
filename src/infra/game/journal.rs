use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::contract::JournalWatcher;
use crate::domain::event::GameEvent;
use crate::error::CoreError;
use crate::infra::game::finder::get_latest_journal_file;

pub struct GameJournalWatcher {
    is_running: Arc<AtomicBool>,
    journal_dir: PathBuf,
}

impl GameJournalWatcher {
    pub fn new(journal_dir: PathBuf) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            journal_dir,
        }
    }

    pub fn read_initial_state(dir: &Path) -> Option<GameEvent> {
        let latest_file = get_latest_journal_file(dir)?;
        let file = File::open(&latest_file).ok()?;
        let reader = BufReader::new(file);

        let mut last_location_event = None;

        for line in reader.lines().map_while(Result::ok) {
            if let Some(event) = Self::parse_journal_line(&line) {
                match &event {
                    GameEvent::Location { .. }
                    | GameEvent::Docked { .. }
                    | GameEvent::Jump { .. } => {
                        last_location_event = Some(event);
                    }
                    _ => {}
                }
            }
        }

        last_location_event
    }

    pub fn parse_journal_line(line: &str) -> Option<GameEvent> {
        let v: Value = serde_json::from_str(line).ok()?;
        let event_type = v.get("event")?.as_str()?;

        match event_type {
            "Location" => {
                let system = v.get("StarSystem")?.as_str()?.to_string();
                let docked = v.get("Docked").and_then(|d| d.as_bool()).unwrap_or(false);
                let station = v
                    .get("StationName")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                Some(GameEvent::Location {
                    system,
                    station,
                    docked,
                })
            }
            "FSDJump" | "CarrierJump" => {
                let system = v.get("StarSystem")?.as_str()?.to_string();
                Some(GameEvent::Jump { system })
            }
            "Docked" => {
                let system = v.get("StarSystem")?.as_str()?.to_string();
                let station = v.get("StationName")?.as_str()?.to_string();
                Some(GameEvent::Docked { system, station })
            }
            "Undocked" => {
                let station = v.get("StationName")?.as_str()?.to_string();
                Some(GameEvent::Undocked { station })
            }
            "MarketBuy" => {
                let commodity = v
                    .get("Type_Localised")
                    .or_else(|| v.get("Type"))?
                    .as_str()?
                    .to_string();
                let count = v.get("Count")?.as_u64()? as u32;
                Some(GameEvent::MarketBuy { commodity, count })
            }
            "MarketSell" => {
                let commodity = v
                    .get("Type_Localised")
                    .or_else(|| v.get("Type"))?
                    .as_str()?
                    .to_string();
                let count = v.get("Count")?.as_u64()? as u32;
                let profit = v.get("Profit").and_then(|p| p.as_i64());
                Some(GameEvent::MarketSell {
                    commodity,
                    count,
                    profit,
                })
            }
            _ => None,
        }
    }
}

impl JournalWatcher for GameJournalWatcher {
    fn start(
        &self,
        callback: Box<dyn Fn(GameEvent) + Send + Sync + 'static>,
    ) -> Result<(), CoreError> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let is_running = Arc::clone(&self.is_running);
        let dir = self.journal_dir.clone();
        let callback = Arc::new(callback);

        std::thread::spawn(move || {
            tracing::info!("Started ED Journal watcher for directory {:?}", dir);

            if let Some(initial_event) = Self::read_initial_state(&dir) {
                tracing::info!("Initial journal state detected: {:?}", initial_event);
                callback(initial_event);
            }

            let mut current_file = get_latest_journal_file(&dir);
            let mut file_offset: u64 = if let Some(ref path) = current_file {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher: Option<RecommendedWatcher> = RecommendedWatcher::new(
                move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        let _ = tx.send(event);
                    }
                },
                Config::default().with_poll_interval(Duration::from_millis(500)),
            )
            .ok();

            if let Some(ref mut w) = watcher {
                let _ = w.watch(&dir, RecursiveMode::NonRecursive);
            }

            while is_running.load(Ordering::SeqCst) {
                let latest_file = get_latest_journal_file(&dir);
                if latest_file != current_file {
                    tracing::info!("Newer journal file detected: {:?}", latest_file);
                    current_file = latest_file;
                    file_offset = 0;
                }

                if let Some(ref path) = current_file
                    && let Ok(mut file) = File::open(path)
                {
                    let current_len = file.metadata().map(|m| m.len()).unwrap_or(0);
                    if current_len > file_offset {
                        if file.seek(SeekFrom::Start(file_offset)).is_ok() {
                            let reader = BufReader::new(file);
                            for line in reader.lines().map_while(Result::ok) {
                                if let Some(event) = Self::parse_journal_line(&line) {
                                    tracing::info!("Journal event: {:?}", event);
                                    callback(event);
                                }
                            }
                        }
                        file_offset = current_len;
                    }
                }

                let _ = rx.recv_timeout(Duration::from_millis(500));
            }

            tracing::info!("ED Journal watcher stopped");
        });

        Ok(())
    }

    fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}
