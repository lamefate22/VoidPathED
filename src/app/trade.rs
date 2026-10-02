use std::sync::Arc;

use crate::contract::{RouteCache, SpanshClient};
use crate::domain::config::SearchConfig;
use crate::domain::route::{FoundStation, RouteStep};
use crate::error::{ApiError, CoreError};

#[derive(Clone)]
pub struct TradeCoordinator {
    spansh: Arc<dyn SpanshClient>,
    cache: Option<Arc<dyn RouteCache>>,
}

impl TradeCoordinator {
    pub fn new(spansh: Arc<dyn SpanshClient>, cache: Option<Arc<dyn RouteCache>>) -> Self {
        Self { spansh, cache }
    }

    pub async fn find_route(&self, config: &SearchConfig) -> Result<Vec<RouteStep>, ApiError> {
        tracing::info!(
            "Requesting route from Spansh: System={}, Station={}, MaxHops={}, Cargo={}",
            config.system,
            config.station,
            config.max_hops,
            config.max_cargo
        );

        let steps = self.spansh.calculate_route(config).await?;
        if let Some(ref cache) = self.cache
            && let Err(e) = cache.save_route(&steps, 0)
        {
            tracing::warn!("Failed to cache newly calculated route: {}", e);
        }
        Ok(steps)
    }

    pub async fn search_stations(&self, query: &str) -> Result<Vec<FoundStation>, ApiError> {
        self.spansh.search_stations(query).await
    }

    pub fn load_cached_route(&self) -> Result<Option<(Vec<RouteStep>, usize)>, CoreError> {
        if let Some(ref cache) = self.cache {
            cache.load_route()
        } else {
            Ok(None)
        }
    }

    pub fn save_cached_step(&self, steps: &[RouteStep], current: usize) {
        if let Some(ref cache) = self.cache
            && let Err(e) = cache.save_route(steps, current)
        {
            tracing::warn!("Failed to save cached route step: {}", e);
        }
    }

    pub fn clear_cached_route(&self) {
        if let Some(ref cache) = self.cache {
            let _ = cache.clear_route();
        }
    }
}
