use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::contract::SpanshClient;
use crate::domain::config::SearchConfig;
use crate::domain::route::{FoundStation, RouteStep};
use crate::error::ApiError;
use crate::infra::spansh::dto::{RouteJobResponse, SearchRouteRequest, TradeRouteResultResponse};

#[derive(Clone)]
pub struct SpanshHttpClient {
    client: Client,
    base_url: String,
}

impl SpanshHttpClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Default::default()
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: Option<&[(&str, &str)]>,
        headers: Option<HeaderMap>,
    ) -> Result<T, ApiError> {
        let mut request = self.client.get(format!("{}{}", self.base_url, path));

        if let Some(h) = headers {
            request = request.headers(h);
        }

        if let Some(q) = query {
            request = request.query(q);
        }

        Ok(request
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?)
    }

    async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: Option<HeaderMap>,
    ) -> Result<T, ApiError> {
        let mut request = self.client.post(format!("{}{}", self.base_url, path));

        if let Some(h) = headers {
            request = request.headers(h);
        }

        Ok(request
            .form(body)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?)
    }

    async fn await_route(
        &self,
        job: &str,
        poll_interval: Duration,
        timeout: Duration,
    ) -> Result<TradeRouteResultResponse, ApiError> {
        let start = tokio::time::Instant::now();
        let mut last_log = tokio::time::Instant::now();
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Requested-With",
            HeaderValue::from_static("XMLHttpRequest"),
        );

        loop {
            if start.elapsed() > timeout {
                return Err(ApiError::Timeout(timeout.as_secs()));
            }

            match self
                .get::<TradeRouteResultResponse>(
                    &format!("/api/results/{}", job),
                    None,
                    Some(headers.clone()),
                )
                .await
            {
                Ok(res) => {
                    if let Some(err) = &res.error {
                        return Err(ApiError::JobFailed(err.clone()));
                    }

                    if res.result.is_some() {
                        tracing::info!(
                            "Spansh route calculation completed in {:.1}s",
                            start.elapsed().as_secs_f32()
                        );
                        return Ok(res);
                    }

                    let status = res.status.as_deref().unwrap_or("unknown");
                    let state = res.state.as_deref().unwrap_or("unknown");

                    if status == "error" || state == "failed" {
                        return Err(ApiError::JobFailed(
                            res.error
                                .unwrap_or_else(|| "Unknown Spansh error".to_string()),
                        ));
                    }

                    if last_log.elapsed() >= Duration::from_secs(15) {
                        tracing::info!(
                            "Waiting for Spansh route calculation: job {} (state: {}, elapsed: {:.0}s)...",
                            job,
                            state,
                            start.elapsed().as_secs_f32()
                        );
                        last_log = tokio::time::Instant::now();
                    }
                }
                Err(e) => {
                    tracing::warn!("Transient error polling route job {}: {}", job, e);
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}

impl Default for SpanshHttpClient {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.7324.122 Safari/537.36",
            ),
        );
        headers.insert(
            "Referer",
            HeaderValue::from_static("https://spansh.co.uk/trade"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: "https://spansh.co.uk".to_string(),
        }
    }
}

impl SpanshClient for SpanshHttpClient {
    fn check_health(&self) -> crate::contract::spansh::BoxFuture<'_, Result<(), ApiError>> {
        Box::pin(async move {
            let request = self.client.head(self.base_url.clone());
            request.send().await?.error_for_status()?;
            Ok(())
        })
    }

    fn search_stations<'a>(
        &'a self,
        query: &'a str,
    ) -> crate::contract::spansh::BoxFuture<'a, Result<Vec<FoundStation>, ApiError>> {
        Box::pin(async move { self.get("/api/stations", Some(&[("q", query)]), None).await })
    }

    fn calculate_route<'a>(
        &'a self,
        config: &'a SearchConfig,
    ) -> crate::contract::spansh::BoxFuture<'a, Result<Vec<RouteStep>, ApiError>> {
        Box::pin(async move {
            let request_body = SearchRouteRequest::from(config);
            let mut headers = HeaderMap::new();
            headers.insert("Origin", HeaderValue::from_static("https://spansh.co.uk"));
            headers.insert(
                "X-Requested-With",
                HeaderValue::from_static("XMLHttpRequest"),
            );

            let job: RouteJobResponse = self
                .post("/api/trade/route", &request_body, Some(headers))
                .await?;

            let result = self
                .await_route(
                    &job.job,
                    Duration::from_millis(2000),
                    Duration::from_secs(300),
                )
                .await?;

            match result.result {
                Some(steps) if !steps.is_empty() => Ok(steps),
                _ => Err(ApiError::NoRouteFound),
            }
        })
    }
}
