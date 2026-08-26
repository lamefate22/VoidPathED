use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum JournalEvent {
    Location {
        system: String,
        station: Option<String>,
        docked: bool,
    },
    Jump {
        system: String,
    },
    Docked {
        system: String,
        station: String,
    },
    Undocked {
        station: String,
    },
    MarketBuy {
        commodity: String,
        count: u32,
    },
    MarketSell {
        commodity: String,
        count: u32,
        profit: Option<i64>,
    },
}

#[allow(dead_code)]
pub struct JournalWatcher {
    is_running: Arc<AtomicBool>,
    journal_dir: PathBuf,
}

#[allow(dead_code)]
impl JournalWatcher {
    pub fn new(custom_path: Option<&str>) -> Option<Self> {
        let dir = if let Some(p) = custom_path {
            if !p.trim().is_empty() {
                PathBuf::from(p)
            } else {
                Self::find_default_journal_dir()?
            }
        } else {
            Self::find_default_journal_dir()?
        };

        if !dir.exists() {
            tracing::warn!("Journal directory not found at {:?}", dir);
            return None;
        }

        Some(Self {
            is_running: Arc::new(AtomicBool::new(false)),
            journal_dir: dir,
        })
    }

    pub fn find_default_journal_dir() -> Option<PathBuf> {
        // Windows default path
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let path = PathBuf::from(profile)
                .join("Saved Games")
                .join("Frontier Developments")
                .join("Elite Dangerous");
            if path.exists() {
                return Some(path);
            }
        }

        // Linux / Steam Deck Proton path (AppID: 359320)
        if let Ok(home) = std::env::var("HOME") {
            let path = PathBuf::from(home)
                .join(".local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    pub fn get_latest_journal_file(dir: &Path) -> Option<PathBuf> {
        let read_dir = std::fs::read_dir(dir).ok()?;
        let mut journals: Vec<PathBuf> = read_dir
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| {
                p.is_file()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|name| name.starts_with("Journal.") && name.ends_with(".log"))
                        .unwrap_or(false)
            })
            .collect();

        journals.sort_by(|a, b| {
            let meta_a = a.metadata().and_then(|m| m.modified()).ok();
            let meta_b = b.metadata().and_then(|m| m.modified()).ok();
            meta_a.cmp(&meta_b)
        });

        journals.pop()
    }

    pub fn read_initial_state(dir: &Path) -> Option<JournalEvent> {
        let latest_file = Self::get_latest_journal_file(dir)?;
        let file = File::open(&latest_file).ok()?;
        let reader = BufReader::new(file);

        let mut last_location_event = None;

        for line in reader.lines().filter_map(|l| l.ok()) {
            if let Some(event) = Self::parse_journal_line(&line) {
                match &event {
                    JournalEvent::Location { .. } | JournalEvent::Docked { .. } | JournalEvent::Jump { .. } => {
                        last_location_event = Some(event);
                    }
                    _ => {}
                }
            }
        }

        last_location_event
    }

    pub fn parse_journal_line(line: &str) -> Option<JournalEvent> {
        let v: Value = serde_json::from_str(line).ok()?;
        let event_type = v.get("event")?.as_str()?;

        match event_type {
            "Location" => {
                let system = v.get("StarSystem")?.as_str()?.to_string();
                let docked = v.get("Docked").and_then(|d| d.as_bool()).unwrap_or(false);
                let station = v.get("StationName").and_then(|s| s.as_str()).map(|s| s.to_string());

                Some(JournalEvent::Location {
                    system,
                    station,
                    docked,
                })
            }
            "FSDJump" | "CarrierJump" => {
                let system = v.get("StarSystem")?.as_str()?.to_string();
                Some(JournalEvent::Jump { system })
            }
            "Docked" => {
                let system = v.get("StarSystem")?.as_str()?.to_string();
                let station = v.get("StationName")?.as_str()?.to_string();
                Some(JournalEvent::Docked { system, station })
            }
            "Undocked" => {
                let station = v.get("StationName")?.as_str()?.to_string();
                Some(JournalEvent::Undocked { station })
            }
            "MarketBuy" => {
                let commodity = v
                    .get("Type_Localised")
                    .or_else(|| v.get("Type"))?
                    .as_str()?
                    .to_string();
                let count = v.get("Count")?.as_u64()? as u32;
                Some(JournalEvent::MarketBuy { commodity, count })
            }
            "MarketSell" => {
                let commodity = v
                    .get("Type_Localised")
                    .or_else(|| v.get("Type"))?
                    .as_str()?
                    .to_string();
                let count = v.get("Count")?.as_u64()? as u32;
                let profit = v.get("Profit").and_then(|p| p.as_i64());
                Some(JournalEvent::MarketSell {
                    commodity,
                    count,
                    profit,
                })
            }
            _ => None,
        }
    }

    pub fn start<F>(&self, callback: F)
    where
        F: Fn(JournalEvent) + Send + Sync + 'static,
    {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let is_running = Arc::clone(&self.is_running);
        let dir = self.journal_dir.clone();
        let callback = Arc::new(callback);

        std::thread::spawn(move || {
            tracing::info!("Started ED Journal watcher for directory {:?}", dir);

            // Trigger initial location if available
            if let Some(initial_event) = Self::read_initial_state(&dir) {
                tracing::info!("Initial journal state detected: {:?}", initial_event);
                callback(initial_event);
            }

            let mut current_file = Self::get_latest_journal_file(&dir);
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
                // Check if a newer journal file was created
                let latest_file = Self::get_latest_journal_file(&dir);
                if latest_file != current_file {
                    tracing::info!("Newer journal file detected: {:?}", latest_file);
                    current_file = latest_file;
                    file_offset = 0;
                }

                if let Some(ref path) = current_file {
                    if let Ok(mut file) = File::open(path) {
                        let current_len = file.metadata().map(|m| m.len()).unwrap_or(0);
                        if current_len > file_offset {
                            if file.seek(SeekFrom::Start(file_offset)).is_ok() {
                                let reader = BufReader::new(file);
                                for line in reader.lines().filter_map(|l| l.ok()) {
                                    if let Some(event) = Self::parse_journal_line(&line) {
                                        tracing::info!("Journal event: {:?}", event);
                                        callback(event);
                                    }
                                }
                            }
                            file_offset = current_len;
                        }
                    }
                }

                // Drain notify messages or sleep
                let _ = rx.recv_timeout(Duration::from_millis(500));
            }

            tracing::info!("ED Journal watcher stopped");
        });
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}
