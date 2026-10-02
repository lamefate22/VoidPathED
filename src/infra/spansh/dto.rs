use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::domain::config::SearchConfig;

pub mod bool_u8 {
    use super::*;

    pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(if *value { 1 } else { 0 })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(value != 0)
    }
}

// Request: POST /api/trade/route
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchRouteRequest {
    #[serde(with = "bool_u8")]
    pub allow_restricted_access: bool,
    #[serde(with = "bool_u8")]
    pub requires_large_pad: bool,
    #[serde(with = "bool_u8")]
    pub allow_player_owned: bool,
    #[serde(with = "bool_u8")]
    pub allow_prohibited: bool,
    #[serde(with = "bool_u8")]
    pub allow_planetary: bool,
    #[serde(with = "bool_u8")]
    pub unique: bool,
    #[serde(with = "bool_u8")]
    pub permit: bool,
    pub max_system_distance: u32,
    pub max_hop_distance: u16,
    pub starting_capital: u64,
    pub max_price_age: u32,
    pub station: String,
    pub system: String,
    pub max_cargo: u16,
    pub max_hops: u16,
}

impl From<&SearchConfig> for SearchRouteRequest {
    fn from(s: &SearchConfig) -> Self {
        Self {
            allow_restricted_access: s.allow_restricted_access,
            requires_large_pad: s.requires_large_pad,
            allow_player_owned: s.allow_player_owned,
            allow_prohibited: s.allow_prohibited,
            allow_planetary: s.allow_planetary,
            unique: s.unique,
            permit: s.permit,
            max_system_distance: s.max_system_distance,
            max_hop_distance: s.max_hop_distance,
            starting_capital: s.starting_capital,
            max_price_age: s.max_price_age,
            station: s.station.clone(),
            system: s.system.clone(),
            max_cargo: s.max_cargo,
            max_hops: s.max_hops,
        }
    }
}

// Response: POST /api/trade/route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteJobResponse {
    pub job: String,
    pub status: String,
}

// Response: GET /api/results/{job}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRouteResultResponse {
    pub job: String,
    pub state: Option<String>,
    pub status: Option<String>,
    pub result: Option<Vec<crate::domain::route::RouteStep>>,
    pub error: Option<String>,
}
