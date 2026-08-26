use std::sync::{Arc, Mutex};
use std::time::Duration;
use slint::ComponentHandle;

use crate::api::client::ApiClient;
use crate::core::models::state::AppState;
use crate::core::services::clipboard;
use crate::core::services::hotkey::HotkeyManager;
use crate::core::services::journal::{JournalEvent, JournalWatcher};
use crate::core::services::trade::TradeService;
use crate::core::settings::TOMLoader;
use crate::core::utils::{center_window_top, show_child_window_centered, show_window};

slint::include_modules!();

fn format_number(val: u64) -> String {
    let s = val.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

fn update_main_ui(win: &MainWindow, state: &AppState, config: &TOMLoader) {
    let search_s = &config.settings.search_settings;

    win.set_journal_connected(state.journal_connected);
    win.set_is_searching_route(state.is_searching);

    let start_system = if !state.current_system.is_empty() {
        &state.current_system
    } else {
        &search_s.system
    };

    let start_station = if !state.current_station.is_empty() {
        &state.current_station
    } else {
        &search_s.station
    };

    win.set_starting_location_text(format!("{} / {}", start_system, start_station).into());

    if let Some(err) = &state.last_error {
        win.set_error_text(err.clone().into());
    } else {
        win.set_error_text("".into());
    }

    if let Some(route) = &state.active_route {
        if !route.is_empty() {
            win.set_is_route(true);
            let cur_idx = state.current_step_index;
            let total = route.len();

            win.set_current_hop((cur_idx + 1) as i32);
            win.set_total_hops(total as i32);
            win.set_total_profit(format!("{} CR", format_number(state.total_route_profit())).into());

            if let Some(step) = route.get(cur_idx) {
                win.set_hop_distance(format!("{:.1} LY", step.distance).into());
                win.set_hop_profit(format!("+{} CR", format_number(step.total_profit)).into());

                // Source (Buy)
                win.set_source_system(step.source.system.clone().into());
                win.set_source_station(step.source.station.clone().into());
                if let Some(dist) = step.source.distance_to_arrival {
                    win.set_source_dist(format!("{} ls", format_number(dist as u64)).into());
                } else {
                    win.set_source_dist("-".into());
                }

                // Destination (Sell)
                win.set_dest_system(step.destination.system.clone().into());
                win.set_dest_station(step.destination.station.clone().into());
                if let Some(dist) = step.destination.distance_to_arrival {
                    win.set_dest_dist(format!("{} ls", format_number(dist as u64)).into());
                } else {
                    win.set_dest_dist("-".into());
                }

                // Commodity
                if let Some(comm) = step.commodities.first() {
                    win.set_commodity_name(comm.name.clone().into());
                    win.set_commodity_amount(format!("{} t", comm.amount).into());
                    win.set_buy_price(format!("{} CR", format_number(comm.source_commodity.buy_price as u64)).into());
                    win.set_sell_price(format!("{} CR", format_number(comm.destination_commodity.sell_price as u64)).into());
                    win.set_unit_profit(format!("+{} CR/t", format_number(comm.profit)).into());
                }
            }
            return;
        }
    }

    win.set_is_route(false);
}

pub fn open_init_window() {
    let window = InitWindow::new().unwrap();

    show_window(window, |win| {
        win.on_close_window({
            let w = win.as_weak();
            move || {
                let _ = w.unwrap().hide();
            }
        });

        let window_weak = win.as_weak();
        slint::spawn_local(async move {
            tracing::info!("Running API connectivity check...");
            let client = ApiClient::new();
            let win = window_weak.unwrap();

            match client.check_api_connectivity().await {
                Ok(_) => {
                    tracing::info!("Connectivity status: OK");
                    let _ = win.hide();
                }
                Err(e) => {
                    tracing::error!("Connectivity status: ERR - {}", e);
                    win.set_is_loading(false);
                    win.set_is_error(true);
                    win.set_error_message(e.to_string().into());
                }
            }
        })
        .unwrap();
    });
}

pub fn open_main_window(config: Arc<Mutex<TOMLoader>>, state: Arc<Mutex<AppState>>) {
    let window = MainWindow::new().unwrap();

    // Initial state render
    {
        let cfg = config.lock().unwrap();
        let st = state.lock().unwrap();
        update_main_ui(&window, &st, &cfg);
    }

    // Start Journal Watcher
    let journal_state = Arc::clone(&state);
    let journal_config = Arc::clone(&config);
    let journal_win_weak = window.as_weak();

    let custom_journal_path = {
        let cfg = config.lock().unwrap();
        cfg.settings.app_settings.journal_path.clone()
    };

    if let Some(watcher) = JournalWatcher::new(if custom_journal_path.is_empty() {
        None
    } else {
        Some(&custom_journal_path)
    }) {
        let state_c = Arc::clone(&journal_state);
        let config_c = Arc::clone(&journal_config);
        let win_weak_c = journal_win_weak.clone();

        watcher.start(move |event| {
            let mut st = state_c.lock().unwrap();
            let cfg = config_c.lock().unwrap();
            st.journal_connected = true;

            let auto_advance = cfg.settings.app_settings.auto_advance_route;

            match &event {
                JournalEvent::Location { system, station, docked } => {
                    st.current_system = system.clone();
                    if let Some(st_name) = station {
                        st.current_station = st_name.clone();
                    }
                    st.is_docked = *docked;
                }
                JournalEvent::Jump { system } => {
                    st.current_system = system.clone();
                    st.is_docked = false;
                }
                JournalEvent::Docked { system, station } => {
                    st.current_system = system.clone();
                    st.current_station = station.clone();
                    st.is_docked = true;

                    // Check if player docked at current destination station to auto-advance
                    if auto_advance {
                        if let Some(step) = st.current_step() {
                            if step.destination.system.eq_ignore_ascii_case(system)
                                && step.destination.station.eq_ignore_ascii_case(station)
                            {
                                tracing::info!("Auto-advancing route hop on arrival at {} / {}", system, station);
                                st.next_step();
                            }
                        }
                    }
                }
                JournalEvent::Undocked { .. } => {
                    st.is_docked = false;
                }
                JournalEvent::MarketSell { .. } => {
                    if auto_advance {
                        if let Some(step) = st.current_step() {
                            if step.destination.system.eq_ignore_ascii_case(&st.current_system) {
                                tracing::info!("Auto-advancing route hop on market sell at {}", st.current_system);
                                st.next_step();
                            }
                        }
                    }
                }
                _ => {}
            }

            let state_snapshot = st.clone();
            let config_snapshot = cfg.clone();
            let win_weak = win_weak_c.clone();

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(win) = win_weak.upgrade() {
                    update_main_ui(&win, &state_snapshot, &config_snapshot);
                }
            });
        });
    }

    // Start Global Hotkey Listener (Ctrl + Shift + V)
    let hotkey_win_weak = window.as_weak();
    let is_visible = Arc::new(Mutex::new(true));
    let is_visible_clone = Arc::clone(&is_visible);

    let _hotkey_manager = HotkeyManager::start(move || {
        let win_weak = hotkey_win_weak.clone();
        let is_visible = Arc::clone(&is_visible_clone);

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(win) = win_weak.upgrade() {
                let mut vis = is_visible.lock().unwrap();
                if *vis {
                    let _ = win.hide();
                    *vis = false;
                } else {
                    let _ = win.show();
                    center_window_top(&win);
                    *vis = true;
                }
            }
        });
    });

    show_window(window, |win| {
        // Close window callback
        win.on_close_window({
            let w = win.as_weak();
            move || {
                if let Some(w) = w.upgrade() {
                    let _ = w.hide();
                }
            }
        });

        // Open Settings callback
        win.on_open_settings({
            let config = Arc::clone(&config);
            let state = Arc::clone(&state);
            let main_weak = win.as_weak();
            move || {
                open_settings_window(Arc::clone(&config), Arc::clone(&state), main_weak.clone());
            }
        });

        // Next Hop callback
        win.on_next_hop({
            let state = Arc::clone(&state);
            let config = Arc::clone(&config);
            let w = win.as_weak();
            move || {
                let mut st = state.lock().unwrap();
                st.next_step();
                let cfg = config.lock().unwrap();
                if let Some(win) = w.upgrade() {
                    update_main_ui(&win, &st, &cfg);
                }
            }
        });

        // Prev Hop callback
        win.on_prev_hop({
            let state = Arc::clone(&state);
            let config = Arc::clone(&config);
            let w = win.as_weak();
            move || {
                let mut st = state.lock().unwrap();
                st.prev_step();
                let cfg = config.lock().unwrap();
                if let Some(win) = w.upgrade() {
                    update_main_ui(&win, &st, &cfg);
                }
            }
        });

        // Copy Target Destination System to Clipboard
        win.on_copy_dest_system({
            let state = Arc::clone(&state);
            let w = win.as_weak();
            move || {
                let st = state.lock().unwrap();
                if let Some(step) = st.current_step() {
                    let dest_system = step.destination.system.clone();
                    if let Err(e) = clipboard::set_clipboard_text(&dest_system) {
                        tracing::error!("Failed to copy to clipboard: {}", e);
                    } else if let Some(win) = w.upgrade() {
                        win.set_notification_message(format!("Copied '{}'", dest_system).into());
                        win.set_show_notification(true);

                        let w_clone = w.clone();
                        slint::spawn_local(async move {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            if let Some(win) = w_clone.upgrade() {
                                win.set_show_notification(false);
                            }
                        })
                        .unwrap();
                    }
                }
            }
        });

        // Clear Route callback
        win.on_clear_route({
            let state = Arc::clone(&state);
            let config = Arc::clone(&config);
            let w = win.as_weak();
            move || {
                let mut st = state.lock().unwrap();
                st.clear_route();
                let cfg = config.lock().unwrap();
                if let Some(win) = w.upgrade() {
                    update_main_ui(&win, &st, &cfg);
                }
            }
        });

        // Search Route callback
        win.on_search_route({
            let state = Arc::clone(&state);
            let config = Arc::clone(&config);
            let w = win.as_weak();

            move || {
                let mut search_settings = {
                    let cfg = config.lock().unwrap();
                    cfg.settings.search_settings.clone()
                };

                // Override start system & station with live ED Journal location if available
                {
                    let st = state.lock().unwrap();
                    if !st.current_system.is_empty() {
                        search_settings.system = st.current_system.clone();
                    }
                    if !st.current_station.is_empty() {
                        search_settings.station = st.current_station.clone();
                    }
                }

                {
                    let mut st = state.lock().unwrap();
                    st.is_searching = true;
                    st.last_error = None;
                    let cfg = config.lock().unwrap();
                    if let Some(win) = w.upgrade() {
                        update_main_ui(&win, &st, &cfg);
                    }
                }

                let state_clone = Arc::clone(&state);
                let config_clone = Arc::clone(&config);
                let w_clone = w.clone();

                slint::spawn_local(async move {
                    let trade_service = TradeService::new();
                    match trade_service.find_route(&search_settings).await {
                        Ok(steps) => {
                            tracing::info!("Found route with {} hops", steps.len());
                            let mut st = state_clone.lock().unwrap();
                            st.set_route(steps);
                        }
                        Err(e) => {
                            tracing::error!("Route search failed: {}", e);
                            let mut st = state_clone.lock().unwrap();
                            st.is_searching = false;
                            st.last_error = Some(format!("Search failed: {}", e));
                        }
                    }

                    let st = state_clone.lock().unwrap();
                    let cfg = config_clone.lock().unwrap();
                    if let Some(win) = w_clone.upgrade() {
                        update_main_ui(&win, &st, &cfg);
                    }
                })
                .unwrap();
            }
        });
    });
}

pub fn open_settings_window(
    config: Arc<Mutex<TOMLoader>>,
    state: Arc<Mutex<AppState>>,
    main_window_weak: slint::Weak<MainWindow>,
) {
    let window = SettingsWindow::new().unwrap();

    // Populate Settings Window fields from config
    {
        let config_guard = config.lock().unwrap();
        let s = &config_guard.settings.search_settings;
        let a = &config_guard.settings.app_settings;

        window.set_starting_point(format!("{} / {}", s.system, s.station).into());
        window.set_selected_system(s.system.clone().into());
        window.set_selected_station(s.station.clone().into());

        window.set_max_hops(s.max_hops as i32);
        window.set_max_cargo(s.max_cargo as i32);
        window.set_max_hop_distance(s.max_hop_distance as i32);
        window.set_max_system_distance(s.max_system_distance as i32);
        window.set_starting_capital(s.starting_capital as i32);
        window.set_max_price_age(s.max_price_age as i32);

        window.set_allow_planetary(s.allow_planetary);
        window.set_allow_player_owned(s.allow_player_owned);
        window.set_allow_prohibited(s.allow_prohibited);
        window.set_allow_restricted_access(s.allow_restricted_access);
        window.set_requires_large_pad(s.requires_large_pad);
        window.set_unique(s.unique);
        window.set_permit(s.permit);

        window.set_auto_advance_route(a.auto_advance_route);
        window.set_hotkey_str(a.hotkey.clone().into());
        window.set_journal_path(a.journal_path.clone().into());
    }

    show_child_window_centered(
        window,
        |win| {
            // Close window callback
            win.on_close_window({
                let w = win.as_weak();
                move || {
                    if let Some(w) = w.upgrade() {
                        let _ = w.hide();
                    }
                }
            });

            // Auto-detect current system/station from Journal state
            win.on_auto_detect_location({
                let state = Arc::clone(&state);
                let w = win.as_weak();
                move || {
                    let st = state.lock().unwrap();
                    if !st.current_system.is_empty() {
                        if let Some(win) = w.upgrade() {
                            let system = &st.current_system;
                            let station = if !st.current_station.is_empty() {
                                &st.current_station
                            } else {
                                ""
                            };

                            win.set_selected_system(system.clone().into());
                            win.set_selected_station(station.to_string().into());
                            if station.is_empty() {
                                win.set_starting_point(system.clone().into());
                            } else {
                                win.set_starting_point(format!("{} / {}", system, station).into());
                            }
                            win.set_show_suggestions(false);
                            tracing::info!("Auto-detected ED location: {} / {}", system, station);
                        }
                    }
                }
            });

            // Station query input callback (debounced search via Spansh API)
            win.on_station_query_changed({
                let w = win.as_weak();
                let request_id = Arc::new(std::sync::atomic::AtomicU64::new(0));

                move |query| {
                    let w = w.clone();
                    let request_id = Arc::clone(&request_id);
                    let current_id =
                        request_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

                    slint::spawn_local(async move {
                        tokio::time::sleep(Duration::from_millis(600)).await;

                        if request_id.load(std::sync::atomic::Ordering::Relaxed) != current_id {
                            return;
                        }

                        let win = match w.upgrade() {
                            Some(w) => w,
                            None => return,
                        };

                        if query.trim().len() < 2 {
                            win.set_show_suggestions(false);
                            return;
                        }

                        let client = ApiClient::new();
                        match client.search_stations(&query).await {
                            Ok(stations) => {
                                if request_id.load(std::sync::atomic::Ordering::Relaxed) != current_id {
                                    return;
                                }

                                if stations.is_empty() {
                                    win.set_show_suggestions(false);
                                    return;
                                }

                                let suggestions: Vec<StationSuggestion> = stations
                                    .iter()
                                    .take(8)
                                    .map(|s| StationSuggestion {
                                        name: s.name.clone().into(),
                                        system: s.system.clone().into(),
                                    })
                                    .collect();

                                let model = std::rc::Rc::new(slint::VecModel::from(suggestions));
                                win.set_station_suggestions(model.into());
                                win.set_show_suggestions(true);
                            }
                            Err(e) => {
                                tracing::error!("Station search failed: {}", e);
                                win.set_show_suggestions(false);
                            }
                        }
                    })
                    .unwrap();
                }
            });

            // Station selected callback
            win.on_station_selected({
                let w = win.as_weak();
                move |system, station| {
                    if let Some(win) = w.upgrade() {
                        win.set_selected_system(system.clone());
                        win.set_selected_station(station.clone());
                        win.set_starting_point(format!("{} / {}", system, station).into());
                        win.set_show_suggestions(false);
                    }
                }
            });

            // Save settings callback
            win.on_save({
                let w = win.as_weak();
                let config = Arc::clone(&config);
                let state = Arc::clone(&state);
                let main_weak = main_window_weak.clone();

                move || {
                    let win = match w.upgrade() {
                        Some(w) => w,
                        None => return,
                    };

                    let mut config_guard = config.lock().unwrap();
                    let s = &mut config_guard.settings.search_settings;

                    s.system = win.get_selected_system().to_string();
                    s.station = win.get_selected_station().to_string();
                    s.max_hops = win.get_max_hops().max(1) as u16;
                    s.max_cargo = win.get_max_cargo().max(1) as u16;
                    s.max_hop_distance = win.get_max_hop_distance().max(1) as u16;
                    s.max_system_distance = win.get_max_system_distance().max(100) as u32;
                    s.starting_capital = win.get_starting_capital().max(1000) as u64;
                    s.max_price_age = win.get_max_price_age().max(60) as u32;

                    s.allow_planetary = win.get_allow_planetary();
                    s.allow_player_owned = win.get_allow_player_owned();
                    s.allow_prohibited = win.get_allow_prohibited();
                    s.allow_restricted_access = win.get_allow_restricted_access();
                    s.requires_large_pad = win.get_requires_large_pad();
                    s.unique = win.get_unique();
                    s.permit = win.get_permit();

                    let a = &mut config_guard.settings.app_settings;
                    a.auto_advance_route = win.get_auto_advance_route();
                    a.hotkey = win.get_hotkey_str().to_string();
                    a.journal_path = win.get_journal_path().to_string();

                    if let Err(e) = config_guard.save() {
                        tracing::error!("Failed to save config: {}", e);
                    } else {
                        tracing::info!("Config saved successfully");
                    }

                    // Update main UI with updated config
                    if let Some(main_win) = main_weak.upgrade() {
                        let st = state.lock().unwrap();
                        update_main_ui(&main_win, &st, &config_guard);
                    }

                    let _ = win.hide();
                }
            });
        },
        480,
        620,
    );
}
