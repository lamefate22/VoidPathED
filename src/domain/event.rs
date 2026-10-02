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
    Loadout(ShipLoadout),
    Cargo(CargoHold),
    Status(ShipStatus),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LandingPadSize {
    Small,
    Medium,
    Large,
}

impl LandingPadSize {
    pub fn from_ship_type(ship_type: &str) -> Self {
        let clean = ship_type.trim().to_ascii_lowercase();
        if clean.contains("anaconda")
            || clean.contains("beluga")
            || clean.contains("cutter")
            || clean.contains("corvette")
            || clean.contains("type9")
            || clean.contains("type10")
            || clean.contains("type7")
            || clean.contains("type_7")
            || clean.contains("orca")
            || clean.contains("clipper")
        {
            Self::Large
        } else if clean.contains("asp")
            || clean.contains("python")
            || clean.contains("krait")
            || clean.contains("ferdelance")
            || clean.contains("fdl")
            || clean.contains("chieftain")
            || clean.contains("crusader")
            || clean.contains("challenger")
            || clean.contains("dropship")
            || clean.contains("gunship")
            || clean.contains("assault")
            || clean.contains("keelback")
            || clean.contains("mandalay")
            || clean.contains("typex")
            || clean.contains("independant_trader")
        {
            Self::Medium
        } else {
            Self::Small
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipLoadout {
    pub ship_type: String,
    pub ship_name: String,
    pub ship_ident: String,
    pub cargo_capacity: u32,
    pub max_jump_range: f32,
    pub pad_size: LandingPadSize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CargoItem {
    pub name: String,
    pub name_localised: Option<String>,
    pub count: u32,
    pub stolen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CargoHold {
    pub count: u32,
    pub items: Vec<CargoItem>,
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
