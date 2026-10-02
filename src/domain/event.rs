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
        let clean = ship_type
            .trim()
            .to_ascii_lowercase()
            .replace(['-', '_', ' '], "");

        // 1. Check Large ships
        if clean.contains("panther")
            || clean.contains("anaconda")
            || clean.contains("beluga")
            || clean.contains("cutter")
            || clean.contains("corvette")
            || clean.contains("type9")
            || clean.contains("type10")
            || clean.contains("type7")
            || clean.contains("orca")
            || clean.contains("clipper")
            || clean.contains("empiretrader")
            || clean.contains("caspian")
            || clean.contains("lynx")
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
            || clean.contains("independanttrader")
            || clean.contains("type6")
            || clean.contains("type8")
            || clean.contains("type11")
            || clean.contains("mamba")
            || clean.contains("corsair")
        {
            Self::Medium
        } else {
            Self::Small
        }
    }
}

pub fn canonical_ship_name(ship_type: &str) -> String {
    let clean = ship_type
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ' '], "");

    match clean.as_str() {
        // Large ships
        "panthermkii" | "panther" | "pantherclipper" | "pantherclippermkii" => {
            "Panther Clipper Mk II".to_string()
        }
        "anaconda" => "Anaconda".to_string(),
        "beluga" | "belugaliner" => "Beluga Liner".to_string(),
        "cutter" | "empirecutter" | "imperialcutter" => "Imperial Cutter".to_string(),
        "federationcorvette" | "corvette" | "federalcorvette" => "Federal Corvette".to_string(),
        "empiretrader" | "imperialclipper" | "clipper" => "Imperial Clipper".to_string(),
        "orca" => "Orca".to_string(),
        "type7" | "type7transporter" => "Type-7 Transporter".to_string(),
        "type9" | "type9heavy" => "Type-9 Heavy".to_string(),
        "type9military" | "type10" | "type10defender" => "Type-10 Defender".to_string(),
        "caspian" | "caspianexplorer" => "Caspian Explorer".to_string(),
        "lynx" | "lynxhighliner" => "Lynx Highliner".to_string(),

        // Medium ships
        "python" => "Python".to_string(),
        "pythonnx" | "pythonmkii" | "python2" => "Python Mk II".to_string(),
        "mandalay" => "Mandalay".to_string(),
        "type8" | "type8transporter" => "Type-8 Transporter".to_string(),
        "type11" | "type11prospector" => "Type-11 Prospector".to_string(),
        "kraitmkii" | "krait" => "Krait Mk II".to_string(),
        "kraitlight" | "kraitphantom" | "phantom" => "Krait Phantom".to_string(),
        "ferdelance" | "fdl" => "Fer-de-Lance".to_string(),
        "mamba" => "Mamba".to_string(),
        "asp" | "aspexplorer" => "Asp Explorer".to_string(),
        "aspscout" => "Asp Scout".to_string(),
        "type6" | "type6transporter" | "submersible" => "Type-6 Transporter".to_string(),
        "keelback" | "independanttrader" => "Keelback".to_string(),
        "typex" | "alliancechieftain" | "chieftain" => "Alliance Chieftain".to_string(),
        "typex2" | "alliancechallenger" | "challenger" => "Alliance Challenger".to_string(),
        "typex3" | "alliancecrusader" | "crusader" => "Alliance Crusader".to_string(),
        "federationdropship" | "dropship" | "federaldropship" => "Federal Dropship".to_string(),
        "federationassaultship" | "assault" | "federalassaultship" => {
            "Federal Assault Ship".to_string()
        }
        "federationgunship" | "gunship" | "federalgunship" => "Federal Gunship".to_string(),
        "corsair" => "Corsair".to_string(),

        // Small ships
        "sidewinder" | "sidewindermki" => "Sidewinder Mk I".to_string(),
        "eagle" | "eaglemkii" => "Eagle Mk II".to_string(),
        "empireeagle" | "imperialeagle" => "Imperial Eagle".to_string(),
        "hauler" => "Hauler".to_string(),
        "adder" => "Adder".to_string(),
        "viper" | "vipermkiii" => "Viper Mk III".to_string(),
        "vipermkiv" => "Viper Mk IV".to_string(),
        "cobra" | "cobramkiii" => "Cobra Mk III".to_string(),
        "cobramkiv" => "Cobra Mk IV".to_string(),
        "cobramkv" => "Cobra Mk V".to_string(),
        "diamondback" | "diamondbackscout" | "dbs" => "Diamondback Scout".to_string(),
        "diamondbackxl" | "diamondbackexplorer" | "dbx" => "Diamondback Explorer".to_string(),
        "dolphin" => "Dolphin".to_string(),
        "empirecourier" | "courier" | "imperialcourier" => "Imperial Courier".to_string(),
        "vulture" => "Vulture".to_string(),
        "kestrel" | "kestrelmkii" => "Kestrel Mk II".to_string(),

        // Fallback for unknown ships
        _ => {
            let spaced = ship_type.replace(['_', '-'], " ");
            spaced
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_landing_pads() {
        assert_eq!(
            LandingPadSize::from_ship_type("panthermkii"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("Panther Clipper Mk II"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("anaconda"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("beluga"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("cutter"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("federation_corvette"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("empire_trader"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("type9"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("type9_military"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("type7"),
            LandingPadSize::Large
        );
        assert_eq!(
            LandingPadSize::from_ship_type("orca"),
            LandingPadSize::Large
        );
    }

    #[test]
    fn test_medium_landing_pads() {
        assert_eq!(
            LandingPadSize::from_ship_type("mandalay"),
            LandingPadSize::Medium
        );
        assert_eq!(
            LandingPadSize::from_ship_type("python"),
            LandingPadSize::Medium
        );
        assert_eq!(
            LandingPadSize::from_ship_type("python_nx"),
            LandingPadSize::Medium
        );
        assert_eq!(
            LandingPadSize::from_ship_type("type8"),
            LandingPadSize::Medium
        );
        assert_eq!(
            LandingPadSize::from_ship_type("type6"),
            LandingPadSize::Medium
        );
        assert_eq!(
            LandingPadSize::from_ship_type("krait_mkii"),
            LandingPadSize::Medium
        );
        assert_eq!(
            LandingPadSize::from_ship_type("ferdelance"),
            LandingPadSize::Medium
        );
    }

    #[test]
    fn test_small_landing_pads() {
        assert_eq!(
            LandingPadSize::from_ship_type("sidewinder"),
            LandingPadSize::Small
        );
        assert_eq!(
            LandingPadSize::from_ship_type("hauler"),
            LandingPadSize::Small
        );
        assert_eq!(
            LandingPadSize::from_ship_type("empire_courier"),
            LandingPadSize::Small
        );
        assert_eq!(
            LandingPadSize::from_ship_type("dolphin"),
            LandingPadSize::Small
        );
    }

    #[test]
    fn test_canonical_ship_name() {
        assert_eq!(canonical_ship_name("panthermkii"), "Panther Clipper Mk II");
        assert_eq!(canonical_ship_name("empire_trader"), "Imperial Clipper");
        assert_eq!(canonical_ship_name("mandalay"), "Mandalay");
        assert_eq!(
            canonical_ship_name("federation_corvette"),
            "Federal Corvette"
        );
    }
}
