use std::path::PathBuf;

use crate::core::models::settings::{Settings, SearchSettings};
use crate::error;

pub struct TOMLoader {
    pub settings: Settings,
    pub settings_path: PathBuf
}

impl TOMLoader {
    pub fn new(path: PathBuf) -> Self {
        Self {
            settings: Settings {
                search_settings: SearchSettings {
                    allow_restricted_access: false,
                    allow_player_owned: false,
                    requires_large_pad: false,
                    allow_prohibited: false,
                    allow_planetary: false,
                    unique: false,
                    permit: false,
                    max_system_distance: 100_000_000,
                    max_hop_distance: 20,
                    max_price_age: 28800,
                    max_cargo: 500,
                    starting_capital: 1_000_000,
                    station: "Galileo".to_string(),
                    system: "Sol".to_string(),
                    max_hops: 10
                }
            },
            settings_path: path
        }
    }

    pub fn load(&mut self) -> Result<(), error::Core> {
        let content = std::fs::read_to_string(&self.settings_path)?;
        self.settings = toml::from_str(&content)?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), error::Core> {
        let content = toml::to_string(&self.settings)?;
        std::fs::write(&self.settings_path, content.as_bytes())?;

        Ok(())
    }
}
