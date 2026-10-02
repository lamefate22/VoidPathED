use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::contract::RouteCache;
use crate::domain::route::RouteStep;
use crate::error::CoreError;

#[derive(Serialize, Deserialize)]
struct CachedRouteData {
    current_step: usize,
    steps: Vec<RouteStep>,
}

#[derive(Clone)]
pub struct JsonRouteCache {
    path: PathBuf,
}

impl JsonRouteCache {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl RouteCache for JsonRouteCache {
    fn load_route(&self) -> Result<Option<(Vec<RouteStep>, usize)>, CoreError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.path)?;
        if content.trim().is_empty() {
            return Ok(None);
        }

        let data: CachedRouteData = serde_json::from_str(&content)?;
        Ok(Some((data.steps, data.current_step)))
    }

    fn save_route(&self, steps: &[RouteStep], current_step: usize) -> Result<(), CoreError> {
        let data = CachedRouteData {
            current_step,
            steps: steps.to_vec(),
        };
        let content = serde_json::to_string_pretty(&data)?;
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&self.path, content.as_bytes())?;
        Ok(())
    }

    fn clear_route(&self) -> Result<(), CoreError> {
        if self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
        }
        Ok(())
    }
}
