use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameEvent {
    Location {
        system: String,
        station: Option<String>,
        docked: bool,
    },
    Jump {
        system: String,
    },
    Docked {
        system: String,
        station: String,
    },
    Undocked {
        station: String,
    },
    MarketBuy {
        commodity: String,
        count: u32,
    },
    MarketSell {
        commodity: String,
        count: u32,
        profit: Option<i64>,
    },
    Status(ShipStatus),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ShipStatus {
    pub docked: bool,
    pub landed: bool,
    pub landing_gear: bool,
    pub supercruise: bool,
    pub in_hyperspace: bool,
    pub in_danger: bool,
    pub cargo_scoop: bool,
    pub low_fuel: bool,
    pub has_lat_long: bool,
}

impl ShipStatus {
    pub const FLAG_DOCKED: u64 = 1 << 0;
    pub const FLAG_LANDED: u64 = 1 << 1;
    pub const FLAG_LANDING_GEAR: u64 = 1 << 2;
    pub const FLAG_SUPERCRUISE: u64 = 1 << 4;
    pub const FLAG_CARGO_SCOOP: u64 = 1 << 9;
    pub const FLAG_IN_DANGER: u64 = 1 << 22;
    pub const FLAG_IN_HYPERSPACE: u64 = 1 << 30;

    pub fn from_flags(flags: u64) -> Self {
        Self {
            docked: (flags & Self::FLAG_DOCKED) != 0,
            landed: (flags & Self::FLAG_LANDED) != 0,
            landing_gear: (flags & Self::FLAG_LANDING_GEAR) != 0,
            supercruise: (flags & Self::FLAG_SUPERCRUISE) != 0,
            in_hyperspace: (flags & Self::FLAG_IN_HYPERSPACE) != 0,
            in_danger: (flags & Self::FLAG_IN_DANGER) != 0,
            cargo_scoop: (flags & Self::FLAG_CARGO_SCOOP) != 0,
            low_fuel: false,
            has_lat_long: false,
        }
    }
}
