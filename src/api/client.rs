use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::api::models::route::{SearchRoute, RouteJob};
use crate::api::models::trade::TradeRouteResult;
use crate::api::models::stations::FoundStation;
use crate::error;

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn check_api_connectivity(&self) -> Result<(), error::API> {
        let request = self.client.head(self.base_url.to_string());

        match request.send().await?.error_for_status() {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(error::API::RequestFailed(e))
            }
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, query: Option<&[(&str, &str)]>, headers: Option<HeaderMap>) -> Result<T, error::API> {
        let mut request = self.client.get(format!("{}{}", self.base_url, path));

        if let Some(h) = headers {
            request = request.headers(h);
        }

        if let Some(q) = query {
            request = request.query(q);
        }

        Ok(request.send().await?.error_for_status()?.json::<T>().await?)
    }

    async fn post<T: DeserializeOwned, B: serde::Serialize>(&self, path: &str, body: &B, headers: Option<HeaderMap>) -> Result<T, error::API> {
        let mut request = self.client.post(format!("{}{}", self.base_url, path));

        if let Some(h) = headers {
            request = request.headers(h);
        }

        Ok(request.form(body).send().await?.error_for_status()?.json::<T>().await?)
    }

    pub async fn search_stations(&self, q: &str) -> Result<Vec<FoundStation>, error::API> {
        self.get("/api/stations", Some(&[("q", q)]), None).await
    }

    pub async fn search_route(&self, body: &SearchRoute) -> Result<RouteJob, error::API> {
        let mut headers = HeaderMap::new();

        headers.insert("Origin", HeaderValue::from_static("https://spansh.co.uk"));
        headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));

        self.post("/api/trade/route", body, Some(headers)).await
    }

    pub async fn await_route(&self, job: &str) -> Result<TradeRouteResult, error::API> {
        let mut headers = HeaderMap::new();

        headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));

        self.get(&format!("/api/results/{}", job), None, None).await
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        let mut headers = HeaderMap::new();

        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.7324.122 Safari/537.36")
        );
        headers.insert("Referer", HeaderValue::from_static("https://spansh.co.uk/trade"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            base_url: "https://spansh.co.uk".to_string()
        }
    }
}
