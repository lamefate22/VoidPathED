use std::path::PathBuf;

use crate::core::models::settings::Settings;
use crate::error;

#[derive(Clone)]
pub struct TOMLoader {
    pub settings: Settings,
    pub settings_path: PathBuf,
}

impl TOMLoader {
    pub fn new(path: PathBuf) -> Self {
        Self {
            settings: Settings::default(),
            settings_path: path,
        }
    }

    pub fn load(&mut self) -> Result<(), error::Core> {
        let content = std::fs::read_to_string(&self.settings_path)?;
        self.settings = toml::from_str(&content)?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), error::Core> {
        let content = toml::to_string_pretty(&self.settings)?;
        if let Some(parent) = self.settings_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&self.settings_path, content.as_bytes())?;
        Ok(())
    }
}
