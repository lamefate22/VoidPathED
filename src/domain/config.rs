use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    pub search: SearchConfig,
    #[serde(default)]
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    #[serde(default = "default_auto_advance")]
    pub auto_advance_route: bool,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default)]
    pub journal_path: String,
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
}

fn default_auto_advance() -> bool {
    true
}

fn default_hotkey() -> String {
    "Ctrl+Shift+V".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            auto_advance_route: default_auto_advance(),
            hotkey: default_hotkey(),
            journal_path: String::new(),
            window_x: None,
            window_y: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchConfig {
    pub allow_restricted_access: bool,
    pub requires_large_pad: bool,
    pub allow_player_owned: bool,
    pub allow_prohibited: bool,
    pub allow_planetary: bool,
    pub unique: bool,
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

impl Default for SearchConfig {
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
