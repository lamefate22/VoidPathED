use serde::{Deserialize, Serialize};

use crate::api::utils::bool_u8;

// API Path: api/trade/route (request)
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRoute {
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
    pub max_hops: u16
}

// API Path: api/trade/route (response)
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteJob {
    pub job: String,
    pub status: String
}
