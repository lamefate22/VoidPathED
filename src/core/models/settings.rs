use serde::{Serialize, Deserialize};

use crate::api::utils::bool_u8;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub search_settings: SearchSettings
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchSettings {
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
