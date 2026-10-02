use std::path::PathBuf;

use crate::contract::ConfigStore;
use crate::domain::config::AppConfig;
use crate::error::CoreError;

#[derive(Clone)]
pub struct TomlConfigStore {
    path: PathBuf,
}

impl TomlConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl ConfigStore for TomlConfigStore {
    fn load(&self) -> Result<AppConfig, CoreError> {
        if !self.path.exists() {
            let default_config = AppConfig::default();
            self.save(&default_config)?;
            return Ok(default_config);
        }

        let content = std::fs::read_to_string(&self.path)?;
        match toml::from_str::<AppConfig>(&content) {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                tracing::warn!(
                    "Failed to deserialize config, falling back to default and backing up: {}",
                    e
                );
                let default_cfg = AppConfig::default();
                let _ = self.save(&default_cfg);
                Ok(default_cfg)
            }
        }
    }

    fn save(&self, config: &AppConfig) -> Result<(), CoreError> {
        let content = toml::to_string_pretty(config)?;
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&self.path, content.as_bytes())?;
        Ok(())
    }
}
