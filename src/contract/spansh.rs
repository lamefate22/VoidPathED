use std::future::Future;
use std::pin::Pin;

use crate::domain::config::SearchConfig;
use crate::domain::route::{FoundStation, RouteStep};
use crate::error::ApiError;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait SpanshClient: Send + Sync {
    fn check_health(&self) -> BoxFuture<'_, Result<(), ApiError>>;
    fn search_stations<'a>(
        &'a self,
        query: &'a str,
    ) -> BoxFuture<'a, Result<Vec<FoundStation>, ApiError>>;
    fn calculate_route<'a>(
        &'a self,
        config: &'a SearchConfig,
    ) -> BoxFuture<'a, Result<Vec<RouteStep>, ApiError>>;
}
