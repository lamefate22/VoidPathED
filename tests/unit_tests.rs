use voidpath_rs::app::{ActionHandler, GameTracker};
use voidpath_rs::contract::RouteCache;
use voidpath_rs::domain::config::{AppConfig, SearchConfig};
use voidpath_rs::domain::event::{GameEvent, ShipStatus};
use voidpath_rs::domain::route::{Commodity, PriceInfo, RouteStep, SystemInfo};
use voidpath_rs::domain::state::AppState;
use voidpath_rs::infra::fs::JsonRouteCache;
use voidpath_rs::infra::game::GameJournalWatcher;

#[test]
fn test_settings_conversion_and_defaults() {
    let settings = SearchConfig {
        system: "Sol".to_string(),
        station: "Galileo".to_string(),
        max_hops: 5,
        max_cargo: 720,
        max_hop_distance: 35,
        max_system_distance: 50_000,
        starting_capital: 10_000_000,
        max_price_age: 14400,
        allow_planetary: false,
        allow_player_owned: true,
        allow_prohibited: false,
        allow_restricted_access: false,
        requires_large_pad: true,
        unique: true,
        permit: false,
    };

    assert_eq!(settings.system, "Sol");
    assert_eq!(settings.station, "Galileo");
    assert_eq!(settings.max_hops, 5);
    assert_eq!(settings.max_cargo, 720);
    assert_eq!(settings.max_hop_distance, 35);
    assert!(settings.allow_player_owned);
    assert!(!settings.allow_planetary);
    assert!(settings.unique);

    let default_app = AppConfig::default();
    assert_eq!(default_app.general.hotkey, "Ctrl+Shift+V");
    assert!(default_app.general.auto_advance_route);
}

#[test]
fn test_journal_line_parsing() {
    // 1. Location Event
    let location_json = r#"{"timestamp":"2026-08-26T17:00:00Z","event":"Location","Docked":true,"StationName":"Jameson Memorial","StarSystem":"Shinrarta Dezhra"}"#;
    let event =
        GameJournalWatcher::parse_journal_line(location_json).expect("Should parse Location event");
    assert_eq!(
        event,
        GameEvent::Location {
            system: "Shinrarta Dezhra".to_string(),
            station: Some("Jameson Memorial".to_string()),
            docked: true,
        }
    );

    // 2. FSDJump Event
    let jump_json = r#"{"timestamp":"2026-08-26T17:05:00Z","event":"FSDJump","StarSystem":"Sol","JumpDist":12.4}"#;
    let event =
        GameJournalWatcher::parse_journal_line(jump_json).expect("Should parse FSDJump event");
    assert_eq!(
        event,
        GameEvent::Jump {
            system: "Sol".to_string(),
        }
    );

    // 3. Docked Event
    let docked_json = r#"{"timestamp":"2026-08-26T17:10:00Z","event":"Docked","StationName":"Galileo","StarSystem":"Sol"}"#;
    let event =
        GameJournalWatcher::parse_journal_line(docked_json).expect("Should parse Docked event");
    assert_eq!(
        event,
        GameEvent::Docked {
            system: "Sol".to_string(),
            station: "Galileo".to_string(),
        }
    );

    // 4. MarketBuy Event
    let buy_json = r#"{"timestamp":"2026-08-26T17:12:00Z","event":"MarketBuy","Type":"gold","Type_Localised":"Gold","Count":500,"BuyPrice":45000,"TotalCost":22500000}"#;
    let event =
        GameJournalWatcher::parse_journal_line(buy_json).expect("Should parse MarketBuy event");
    assert_eq!(
        event,
        GameEvent::MarketBuy {
            commodity: "Gold".to_string(),
            count: 500,
        }
    );

    // 5. MarketSell Event
    let sell_json = r#"{"timestamp":"2026-08-26T17:20:00Z","event":"MarketSell","Type":"gold","Type_Localised":"Gold","Count":500,"SellPrice":60000,"TotalSale":30000000,"Profit":7500000}"#;
    let event =
        GameJournalWatcher::parse_journal_line(sell_json).expect("Should parse MarketSell event");
    assert_eq!(
        event,
        GameEvent::MarketSell {
            commodity: "Gold".to_string(),
            count: 500,
            profit: Some(7500000),
        }
    );
}

#[test]
fn test_ship_status_flags() {
    // Docked flag (1 << 0)
    let status_docked = ShipStatus::from_flags(1);
    assert!(status_docked.docked);
    assert!(!status_docked.supercruise);

    // Supercruise flag (1 << 4 = 16)
    let status_sc = ShipStatus::from_flags(16);
    assert!(!status_sc.docked);
    assert!(status_sc.supercruise);

    // Hyperspace flag (1 << 30)
    let status_hyper = ShipStatus::from_flags(1 << 30);
    assert!(status_hyper.in_hyperspace);
}

#[test]
fn test_app_state_navigation() {
    let mut state = AppState::new();
    assert_eq!(state.total_steps(), 0);
    assert_eq!(state.current_step_index, 0);

    let dummy_step1 = RouteStep {
        distance: 14.5,
        total_profit: 2_500_000,
        cumulative_profit: 2_500_000,
        source: SystemInfo {
            system: "Sol".to_string(),
            station: "Galileo".to_string(),
            distance_to_arrival: Some(340),
        },
        destination: SystemInfo {
            system: "Alpha Centauri".to_string(),
            station: "Hutton Orbital".to_string(),
            distance_to_arrival: Some(6784400),
        },
        commodities: vec![Commodity {
            name: "Gold".to_string(),
            amount: 500,
            profit: 5000,
            total_profit: 2_500_000,
            source_commodity: PriceInfo {
                buy_price: 45000,
                sell_price: 0,
                supply: 10000,
                demand: 0,
            },
            destination_commodity: PriceInfo {
                buy_price: 0,
                sell_price: 50000,
                supply: 0,
                demand: 10000,
            },
        }],
    };

    let dummy_step2 = RouteStep {
        distance: 10.2,
        total_profit: 3_000_000,
        cumulative_profit: 5_500_000,
        source: SystemInfo {
            system: "Alpha Centauri".to_string(),
            station: "Hutton Orbital".to_string(),
            distance_to_arrival: Some(6784400),
        },
        destination: SystemInfo {
            system: "Sol".to_string(),
            station: "Galileo".to_string(),
            distance_to_arrival: Some(340),
        },
        commodities: vec![],
    };

    state.set_route(vec![dummy_step1.clone(), dummy_step2]);
    assert_eq!(state.total_steps(), 2);
    assert_eq!(state.current_step_index, 0);
    assert_eq!(state.total_route_profit(), 5_500_000);

    // Navigate forward using ActionHandler
    assert!(ActionHandler::next_hop(&mut state));
    assert_eq!(state.current_step_index, 1);
    assert!(!ActionHandler::next_hop(&mut state));

    // Navigate backward
    assert!(ActionHandler::prev_hop(&mut state));
    assert_eq!(state.current_step_index, 0);
    assert!(!ActionHandler::prev_hop(&mut state));

    // Clear
    ActionHandler::clear_route(&mut state);
    assert_eq!(state.total_steps(), 0);
    assert!(state.active_route.is_none());
}

#[test]
fn test_game_tracker_auto_advance_smart_logic() {
    let mut state = AppState::new();
    let step = RouteStep {
        distance: 15.0,
        total_profit: 1_000_000,
        cumulative_profit: 1_000_000,
        source: SystemInfo {
            system: "Sol".to_string(),
            station: "Galileo".to_string(),
            distance_to_arrival: Some(100),
        },
        destination: SystemInfo {
            system: "Shinrarta Dezhra".to_string(),
            station: "Jameson Memorial".to_string(),
            distance_to_arrival: Some(200),
        },
        commodities: vec![Commodity {
            name: "Bertrandite".to_string(),
            amount: 720,
            profit: 2000,
            total_profit: 1_440_000,
            source_commodity: PriceInfo {
                buy_price: 20000,
                sell_price: 0,
                supply: 5000,
                demand: 0,
            },
            destination_commodity: PriceInfo {
                buy_price: 0,
                sell_price: 22000,
                supply: 0,
                demand: 5000,
            },
        }],
    };

    let step2 = step.clone();
    state.set_route(vec![step, step2]);
    assert_eq!(state.current_step_index, 0);

    // 1. Selling UNRELATED commodity (e.g. Gold) does NOT auto-advance
    state.current_system = "Shinrarta Dezhra".to_string();
    let sell_unrelated = GameEvent::MarketSell {
        commodity: "Gold".to_string(),
        count: 10,
        profit: Some(50000),
    };
    GameTracker::handle_event(&mut state, &sell_unrelated, true);
    assert_eq!(
        state.current_step_index, 0,
        "Selling unrelated commodity should NOT advance hop"
    );

    // 2. Selling TARGET commodity (Bertrandite) at target system DOES auto-advance
    let sell_target = GameEvent::MarketSell {
        commodity: "Bertrandite".to_string(),
        count: 720,
        profit: Some(1440000),
    };
    GameTracker::handle_event(&mut state, &sell_target, true);
    assert_eq!(
        state.current_step_index, 1,
        "Selling target commodity SHOULD advance hop"
    );

    // Reset to step 0
    state.current_step_index = 0;

    // 3. Docking at destination station DOES auto-advance
    let dock_target = GameEvent::Docked {
        system: "Shinrarta Dezhra".to_string(),
        station: "Jameson Memorial".to_string(),
    };
    GameTracker::handle_event(&mut state, &dock_target, true);
    assert_eq!(
        state.current_step_index, 1,
        "Docking at target destination SHOULD advance hop"
    );
}

#[test]
fn test_route_cache_persistence() {
    let temp_dir = std::env::temp_dir();
    let cache_file = temp_dir.join("voidpath_test_route_cache.json");
    let cache = JsonRouteCache::new(cache_file.clone());

    let dummy_step = RouteStep {
        distance: 12.0,
        total_profit: 500_000,
        cumulative_profit: 500_000,
        source: SystemInfo {
            system: "A".to_string(),
            station: "A1".to_string(),
            distance_to_arrival: None,
        },
        destination: SystemInfo {
            system: "B".to_string(),
            station: "B1".to_string(),
            distance_to_arrival: None,
        },
        commodities: vec![],
    };

    cache
        .save_route(std::slice::from_ref(&dummy_step), 0)
        .expect("Should save route cache");

    let loaded = cache.load_route().expect("Should load route cache");
    assert!(loaded.is_some());
    let (steps, current) = loaded.unwrap();
    assert_eq!(steps.len(), 1);
    assert_eq!(current, 0);
    assert_eq!(steps[0].destination.system, "B");

    cache.clear_route().expect("Should clear route cache");
    let after_clear = cache.load_route().expect("Should load empty");
    assert!(after_clear.is_none());
}

#[cfg(target_os = "windows")]
#[test]
fn test_hotkey_parsing() {
    use voidpath_rs::infra::os::hotkey::parse_hotkey;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
    };

    let (mods, key) = parse_hotkey("Ctrl+Shift+V").expect("Should parse default hotkey");
    assert_eq!(mods.0, MOD_CONTROL.0 | MOD_SHIFT.0 | MOD_NOREPEAT.0);
    assert_eq!(key, 0x56); // 'V'

    let (mods, key) = parse_hotkey("Alt+F10").expect("Should parse Alt+F10");
    assert_eq!(mods.0, MOD_ALT.0 | MOD_NOREPEAT.0);
    assert_eq!(key, 0x79); // VK_F10 is 0x70 + 9 = 0x79

    let (mods, key) = parse_hotkey("ctrl + space").expect("Should parse ctrl + space");
    assert_eq!(mods.0, MOD_CONTROL.0 | MOD_NOREPEAT.0);
    assert_eq!(key, 0x20); // VK_SPACE

    assert!(parse_hotkey("invalid_combo_???").is_none());
}
