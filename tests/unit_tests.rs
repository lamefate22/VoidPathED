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

#[test]
fn test_ship_loadout_parsing_and_pad_sizes() {
    use voidpath_rs::domain::event::LandingPadSize;
    use voidpath_rs::infra::game::GameJournalWatcher;

    assert_eq!(
        LandingPadSize::from_ship_type("Anaconda"),
        LandingPadSize::Large
    );
    assert_eq!(
        LandingPadSize::from_ship_type("Type9"),
        LandingPadSize::Large
    );
    assert_eq!(
        LandingPadSize::from_ship_type("type9_military"),
        LandingPadSize::Large
    );
    assert_eq!(
        LandingPadSize::from_ship_type("cutter"),
        LandingPadSize::Large
    );
    assert_eq!(
        LandingPadSize::from_ship_type("Python"),
        LandingPadSize::Medium
    );
    assert_eq!(
        LandingPadSize::from_ship_type("krait_mkii"),
        LandingPadSize::Medium
    );
    assert_eq!(
        LandingPadSize::from_ship_type("sidewinder"),
        LandingPadSize::Small
    );
    assert_eq!(
        LandingPadSize::from_ship_type("panthermkii"),
        LandingPadSize::Large
    );
    assert_eq!(
        LandingPadSize::from_ship_type("empire_trader"),
        LandingPadSize::Large
    );
    assert_eq!(
        LandingPadSize::from_ship_type("hauler"),
        LandingPadSize::Small
    );

    let loadout_line = r#"{
        "timestamp":"2026-10-02T12:00:00Z",
        "event":"Loadout",
        "Ship":"type9",
        "ShipName":"TITAN EXPRESS",
        "ShipIdent":"TX-99",
        "CargoCapacity":752,
        "MaxJumpRange":28.65
    }"#;

    let event = GameJournalWatcher::parse_journal_line(loadout_line).expect("Should parse Loadout");
    if let GameEvent::Loadout(loadout) = event {
        assert_eq!(loadout.ship_type, "Type-9 Heavy");
        assert_eq!(loadout.ship_name, "TITAN EXPRESS");
        assert_eq!(loadout.ship_ident, "TX-99");
        assert_eq!(loadout.cargo_capacity, 752);
        assert!((loadout.max_jump_range - 28.65).abs() < 0.01);
        assert_eq!(loadout.pad_size, LandingPadSize::Large);
    } else {
        panic!("Expected GameEvent::Loadout");
    }
}

#[test]
fn test_cargo_parsing_and_tracker_sync() {
    use voidpath_rs::domain::event::LandingPadSize;
    use voidpath_rs::infra::game::GameJournalWatcher;

    let cargo_line = r#"{
        "timestamp":"2026-10-02T12:05:00Z",
        "event":"Cargo",
        "Vessel":"Ship",
        "Count":720,
        "Inventory":[
            {"Name":"gold", "Name_Localised":"Gold", "Count":500, "Stolen":0},
            {"Name":"silver", "Name_Localised":"Silver", "Count":220, "Stolen":0}
        ]
    }"#;

    let event = GameJournalWatcher::parse_journal_line(cargo_line).expect("Should parse Cargo");
    let mut state = AppState::new();
    let outcome = GameTracker::handle_event(&mut state, &event, true);

    assert!(outcome.changed);
    assert!(!outcome.step_advanced);
    assert!(!outcome.ship_updated);
    let cargo = state
        .current_cargo
        .as_ref()
        .expect("Cargo hold should be populated");
    assert_eq!(cargo.count, 720);
    assert_eq!(cargo.items.len(), 2);
    assert_eq!(cargo.items[0].name, "gold");
    assert_eq!(cargo.items[0].count, 500);

    // Test Loadout sync
    let loadout_line = r#"{
        "timestamp":"2026-10-02T12:00:00Z",
        "event":"Loadout",
        "Ship":"python",
        "ShipName":"PYTHON RUNNER",
        "CargoCapacity":280,
        "MaxJumpRange":34.2
    }"#;

    let loadout_event =
        GameJournalWatcher::parse_journal_line(loadout_line).expect("Parse loadout");
    let loadout_outcome = GameTracker::handle_event(&mut state, &loadout_event, true);

    assert!(loadout_outcome.changed);
    assert!(loadout_outcome.ship_updated);
    let ship = state.current_ship.expect("Ship should be present");
    assert_eq!(ship.cargo_capacity, 280);
    assert_eq!(ship.pad_size, LandingPadSize::Medium);
}

#[test]
fn test_multi_commodity_flow_and_tracking() {
    let mut state = AppState::new();

    let step1 = RouteStep {
        distance: 12.5,
        total_profit: 2_500_000,
        cumulative_profit: 2_500_000,
        source: SystemInfo {
            system: "Sol".to_string(),
            station: "Galileo".to_string(),
            distance_to_arrival: Some(150),
        },
        destination: SystemInfo {
            system: "Alpha Centauri".to_string(),
            station: "Hutton Orbital".to_string(),
            distance_to_arrival: Some(500),
        },
        commodities: vec![
            Commodity {
                name: "Gold".to_string(),
                amount: 400,
                profit: 4000,
                total_profit: 1_600_000,
                source_commodity: PriceInfo {
                    buy_price: 45000,
                    sell_price: 0,
                    supply: 1000,
                    demand: 0,
                },
                destination_commodity: PriceInfo {
                    buy_price: 0,
                    sell_price: 49000,
                    supply: 0,
                    demand: 1000,
                },
            },
            Commodity {
                name: "Silver".to_string(),
                amount: 320,
                profit: 2812,
                total_profit: 900_000,
                source_commodity: PriceInfo {
                    buy_price: 30000,
                    sell_price: 0,
                    supply: 1000,
                    demand: 0,
                },
                destination_commodity: PriceInfo {
                    buy_price: 0,
                    sell_price: 32812,
                    supply: 0,
                    demand: 1000,
                },
            },
        ],
    };

    let step2 = RouteStep {
        distance: 8.0,
        total_profit: 1_200_000,
        cumulative_profit: 3_700_000,
        source: SystemInfo {
            system: "Alpha Centauri".to_string(),
            station: "Hutton Orbital".to_string(),
            distance_to_arrival: Some(500),
        },
        destination: SystemInfo {
            system: "Barnard's Star".to_string(),
            station: "Boston Base".to_string(),
            distance_to_arrival: Some(250),
        },
        commodities: vec![],
    };

    state.set_route(vec![step1, step2]);
    assert_eq!(state.current_step_index, 0);
    assert_eq!(state.current_commodity_index, 0);
    assert!(state.step_commodities_bought.is_empty());
    assert!(state.step_commodities_sold.is_empty());

    // 1. Commodity cycling
    state.cycle_commodity();
    assert_eq!(state.current_commodity_index, 1);
    state.cycle_commodity();
    assert_eq!(state.current_commodity_index, 0);

    // 2. Buying the first commodity (Gold)
    let buy_gold = GameEvent::MarketBuy {
        commodity: "Gold".to_string(),
        count: 400,
    };
    let outcome = GameTracker::handle_event(&mut state, &buy_gold, true);
    assert!(outcome.changed);
    assert!(!outcome.step_advanced);
    assert!(state.step_commodities_bought.contains("Gold"));
    assert_eq!(
        state.current_commodity_index, 1,
        "Should automatically switch to the next unbought commodity (Silver)"
    );

    // 3. Buying the second commodity (Silver)
    let buy_silver = GameEvent::MarketBuy {
        commodity: "Silver".to_string(),
        count: 320,
    };
    let outcome = GameTracker::handle_event(&mut state, &buy_silver, true);
    assert!(outcome.changed);
    assert!(!outcome.step_advanced);
    assert!(state.step_commodities_bought.contains("Silver"));

    // 4. Arrive and dock at destination station
    state.current_system = "Alpha Centauri".to_string();
    let dock_event = GameEvent::Docked {
        system: "Alpha Centauri".to_string(),
        station: "Hutton Orbital".to_string(),
    };
    let outcome = GameTracker::handle_event(&mut state, &dock_event, true);
    assert!(outcome.changed);
    assert!(
        !outcome.step_advanced,
        "Docking should NOT auto-advance when commodities are loaded and unsold"
    );
    assert_eq!(state.current_step_index, 0);

    // 5. Sell first commodity (Gold)
    let sell_gold = GameEvent::MarketSell {
        commodity: "Gold".to_string(),
        count: 400,
        profit: Some(1_600_000),
    };
    let outcome = GameTracker::handle_event(&mut state, &sell_gold, true);
    assert!(outcome.changed);
    assert!(
        !outcome.step_advanced,
        "Selling 1 of 2 commodities should NOT auto-advance yet"
    );
    assert_eq!(state.current_step_index, 0);
    assert!(state.step_commodities_sold.contains("Gold"));
    assert_eq!(
        state.current_commodity_index, 1,
        "Should automatically switch to the next unsold commodity (Silver)"
    );

    // 6. Sell second commodity (Silver) -> All sold, should auto-advance!
    let sell_silver = GameEvent::MarketSell {
        commodity: "Silver".to_string(),
        count: 320,
        profit: Some(900_000),
    };
    let outcome = GameTracker::handle_event(&mut state, &sell_silver, true);
    assert!(outcome.changed);
    assert!(
        outcome.step_advanced,
        "Selling the final commodity SHOULD trigger auto-advance to step 1"
    );
    assert_eq!(state.current_step_index, 1);
    assert_eq!(state.current_commodity_index, 0);
    assert!(state.step_commodities_bought.is_empty());
    assert!(state.step_commodities_sold.is_empty());
}
