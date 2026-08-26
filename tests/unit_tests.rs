use voidpath_rs::api::models::route::SearchRoute;
use voidpath_rs::core::models::settings::SearchSettings;
use voidpath_rs::core::models::state::AppState;
use voidpath_rs::core::services::journal::{JournalEvent, JournalWatcher};

#[test]
fn test_settings_conversion() {
    let settings = SearchSettings {
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

    let route: SearchRoute = (&settings).into();
    assert_eq!(route.system, "Sol");
    assert_eq!(route.station, "Galileo");
    assert_eq!(route.max_hops, 5);
    assert_eq!(route.max_cargo, 720);
    assert_eq!(route.max_hop_distance, 35);
    assert!(route.allow_player_owned);
    assert!(!route.allow_planetary);
    assert!(route.unique);
}

#[test]
fn test_journal_line_parsing() {
    // 1. Location Event
    let location_json = r#"{"timestamp":"2026-08-26T17:00:00Z","event":"Location","Docked":true,"StationName":"Jameson Memorial","StarSystem":"Shinrarta Dezhra"}"#;
    let event = JournalWatcher::parse_journal_line(location_json).expect("Should parse Location event");
    assert_eq!(
        event,
        JournalEvent::Location {
            system: "Shinrarta Dezhra".to_string(),
            station: Some("Jameson Memorial".to_string()),
            docked: true,
        }
    );

    // 2. FSDJump Event
    let jump_json = r#"{"timestamp":"2026-08-26T17:05:00Z","event":"FSDJump","StarSystem":"Sol","JumpDist":12.4}"#;
    let event = JournalWatcher::parse_journal_line(jump_json).expect("Should parse FSDJump event");
    assert_eq!(
        event,
        JournalEvent::Jump {
            system: "Sol".to_string(),
        }
    );

    // 3. Docked Event
    let docked_json = r#"{"timestamp":"2026-08-26T17:10:00Z","event":"Docked","StationName":"Galileo","StarSystem":"Sol"}"#;
    let event = JournalWatcher::parse_journal_line(docked_json).expect("Should parse Docked event");
    assert_eq!(
        event,
        JournalEvent::Docked {
            system: "Sol".to_string(),
            station: "Galileo".to_string(),
        }
    );

    // 4. MarketBuy Event
    let buy_json = r#"{"timestamp":"2026-08-26T17:12:00Z","event":"MarketBuy","Type":"gold","Type_Localised":"Gold","Count":500,"BuyPrice":45000,"TotalCost":22500000}"#;
    let event = JournalWatcher::parse_journal_line(buy_json).expect("Should parse MarketBuy event");
    assert_eq!(
        event,
        JournalEvent::MarketBuy {
            commodity: "Gold".to_string(),
            count: 500,
        }
    );

    // 5. MarketSell Event
    let sell_json = r#"{"timestamp":"2026-08-26T17:20:00Z","event":"MarketSell","Type":"gold","Type_Localised":"Gold","Count":500,"SellPrice":60000,"TotalSale":30000000,"Profit":7500000}"#;
    let event = JournalWatcher::parse_journal_line(sell_json).expect("Should parse MarketSell event");
    assert_eq!(
        event,
        JournalEvent::MarketSell {
            commodity: "Gold".to_string(),
            count: 500,
            profit: Some(7500000),
        }
    );
}

#[test]
fn test_app_state_navigation() {
    use voidpath_rs::api::models::trade::{Commodity, PriceInfo, RouteStep, SystemInfo};

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

    state.set_route(vec![dummy_step1, dummy_step2]);
    assert_eq!(state.total_steps(), 2);
    assert_eq!(state.current_step_index, 0);
    assert_eq!(state.total_route_profit(), 5_500_000);

    // Navigate forward
    assert!(state.next_step());
    assert_eq!(state.current_step_index, 1);
    assert!(!state.next_step()); // cannot go past last step

    // Navigate backward
    assert!(state.prev_step());
    assert_eq!(state.current_step_index, 0);
    assert!(!state.prev_step()); // cannot go before first step

    // Clear
    state.clear_route();
    assert_eq!(state.total_steps(), 0);
    assert!(state.active_route.is_none());
}
