use crate::domain::config::AppConfig;
use crate::domain::route::RouteStep;
use crate::error::CoreError;

pub trait ConfigStore: Send + Sync {
    fn load(&self) -> Result<AppConfig, CoreError>;
    fn save(&self, config: &AppConfig) -> Result<(), CoreError>;
}

pub trait RouteCache: Send + Sync {
    fn load_route(&self) -> Result<Option<(Vec<RouteStep>, usize)>, CoreError>;
    fn save_route(&self, steps: &[RouteStep], current_step: usize) -> Result<(), CoreError>;
    fn clear_route(&self) -> Result<(), CoreError>;
}
