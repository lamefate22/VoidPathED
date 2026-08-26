use std::time::Duration;
use crate::api::client::ApiClient;
use crate::api::models::route::SearchRoute;
use crate::api::models::trade::RouteStep;
use crate::core::models::settings::SearchSettings;
use crate::error;

pub struct TradeService {
    client: ApiClient,
}

impl TradeService {
    pub fn new() -> Self {
        Self {
            client: ApiClient::new(),
        }
    }

    pub async fn find_route(&self, settings: &SearchSettings) -> Result<Vec<RouteStep>, error::API> {
        let route_query: SearchRoute = settings.into();
        tracing::info!(
            "Submitting route search to Spansh: System={}, Station={}, MaxHops={}, Cargo={}",
            route_query.system,
            route_query.station,
            route_query.max_hops,
            route_query.max_cargo
        );

        let job = self.client.search_route(&route_query).await?;
        tracing::info!("Spansh route job created: ID={}, Status={}", job.job, job.status);

        let result = self
            .client
            .await_route(&job.job, Duration::from_millis(1500), Duration::from_secs(45))
            .await?;

        match result.result {
            Some(steps) => {
                if steps.is_empty() {
                    tracing::warn!("Spansh returned 0 route steps");
                    Err(error::API::NoRouteFound)
                } else {
                    tracing::info!("Successfully received route with {} steps", steps.len());
                    Ok(steps)
                }
            }
            None => Err(error::API::NoRouteFound),
        }
    }
}

impl Default for TradeService {
    fn default() -> Self {
        Self::new()
    }
}
