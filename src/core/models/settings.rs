use serde::{Deserialize, Serialize};

use crate::api::models::route::SearchRoute;
use crate::api::utils::bool_u8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub search_settings: SearchSettings,
    #[serde(default)]
    pub app_settings: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    #[serde(default = "default_auto_advance")]
    pub auto_advance_route: bool,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default)]
    pub journal_path: String,
}

fn default_auto_advance() -> bool {
    true
}

fn default_hotkey() -> String {
    "Ctrl+Shift+V".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_advance_route: default_auto_advance(),
            hotkey: default_hotkey(),
            journal_path: String::new(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            search_settings: SearchSettings::default(),
            app_settings: AppSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub max_hops: u16,
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            allow_restricted_access: false,
            allow_player_owned: true,
            requires_large_pad: true,
            allow_prohibited: false,
            allow_planetary: false,
            unique: false,
            permit: false,
            max_system_distance: 100_000_000,
            max_hop_distance: 20,
            max_price_age: 28800,
            max_cargo: 500,
            starting_capital: 1_000_000,
            station: "Galileo".to_string(),
            system: "Sol".to_string(),
            max_hops: 10,
        }
    }
}

impl From<SearchSettings> for SearchRoute {
    fn from(s: SearchSettings) -> Self {
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
            station: s.station,
            system: s.system,
            max_cargo: s.max_cargo,
            max_hops: s.max_hops,
        }
    }
}

impl From<&SearchSettings> for SearchRoute {
    fn from(s: &SearchSettings) -> Self {
        s.clone().into()
    }
}
