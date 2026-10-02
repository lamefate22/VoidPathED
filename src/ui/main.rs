use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::app::{ActionHandler, GameTracker, TradeCoordinator};
use crate::contract::{
    ClipboardService, ConfigStore, HotkeyListener, JournalWatcher, SpanshClient, StatusWatcher,
};
use crate::domain::config::AppConfig;
use crate::domain::state::AppState;
use crate::infra::os::window::{center_window_top, show_window};
use crate::ui::MainWindow;
use crate::ui::mapper::{
    format_distance_ls, format_distance_ly, format_location, format_profit, format_unit_profit,
};
use crate::ui::settings::show_settings_window;

fn lock_mutex<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match m.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub fn update_main_ui(win: &MainWindow, state: &AppState, config: &AppConfig) {
    let search_cfg = &config.search;

    win.set_journal_connected(state.journal_connected);
    win.set_is_searching_route(state.is_searching);

    let start_system = if !state.current_system.is_empty() {
        &state.current_system
    } else {
        &search_cfg.system
    };

    let start_station = if !state.current_station.is_empty() {
        &state.current_station
    } else {
        &search_cfg.station
    };

    win.set_starting_location_text(format_location(start_system, start_station).into());

    if let Some(err) = &state.last_error {
        win.set_error_text(err.clone().into());
    } else {
        win.set_error_text("".into());
    }

    if let Some(route) = &state.active_route
        && !route.is_empty()
    {
        win.set_is_route(true);
        let cur_idx = state.current_step_index;
        let total = route.len();

        win.set_current_hop((cur_idx + 1) as i32);
        win.set_total_hops(total as i32);
        win.set_total_profit(format_profit(state.total_route_profit()).into());

        if let Some(step) = route.get(cur_idx) {
            win.set_hop_distance(format_distance_ly(step.distance).into());
            win.set_hop_profit(format!("+{}", format_profit(step.total_profit)).into());

            // Source (Buy)
            win.set_source_system(step.source.system.clone().into());
            win.set_source_station(step.source.station.clone().into());
            win.set_source_dist(format_distance_ls(step.source.distance_to_arrival).into());

            // Destination (Sell)
            win.set_dest_system(step.destination.system.clone().into());
            win.set_dest_station(step.destination.station.clone().into());
            win.set_dest_dist(format_distance_ls(step.destination.distance_to_arrival).into());

            // Commodity
            if let Some(comm) = step.commodities.first() {
                win.set_commodity_name(comm.name.clone().into());
                win.set_commodity_amount(format!("{} t", comm.amount).into());
                win.set_buy_price(format_profit(comm.source_commodity.buy_price as u64).into());
                win.set_sell_price(
                    format_profit(comm.destination_commodity.sell_price as u64).into(),
                );
                win.set_unit_profit(format_unit_profit(comm.profit).into());
            }
        }
        return;
    }

    win.set_is_route(false);
}

pub struct MainWindowContext {
    pub config: Arc<Mutex<AppConfig>>,
    pub config_store: Arc<dyn ConfigStore>,
    pub state: Arc<Mutex<AppState>>,
    pub trade_coord: Arc<TradeCoordinator>,
    pub spansh: Arc<dyn SpanshClient>,
    pub clipboard: Arc<dyn ClipboardService>,
    pub hotkey: Arc<dyn HotkeyListener>,
    pub journal: Option<Arc<dyn JournalWatcher>>,
    pub status: Option<Arc<dyn StatusWatcher>>,
}

pub fn show_main_window(ctx: MainWindowContext) {
    let MainWindowContext {
        config,
        config_store,
        state,
        trade_coord,
        spansh,
        clipboard,
        hotkey,
        journal,
        status,
    } = ctx;
    let window = match MainWindow::new() {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Failed to instantiate MainWindow: {}", e);
            return;
        }
    };

    // Initial render
    {
        let cfg = lock_mutex(&config);
        let st = lock_mutex(&state);
        update_main_ui(&window, &st, &cfg);
    }

    // Start Journal Watcher
    if let Some(ref j) = journal {
        let state_c = Arc::clone(&state);
        let config_c = Arc::clone(&config);
        let win_weak_c = window.as_weak();

        let _ = j.start(Box::new(move |event| {
            let mut st = lock_mutex(&state_c);
            let cfg = lock_mutex(&config_c);

            let auto_advance = cfg.general.auto_advance_route;
            let changed = GameTracker::handle_event(&mut st, &event, auto_advance);

            if changed {
                let state_snapshot = st.clone();
                let config_snapshot = cfg.clone();
                let win_weak = win_weak_c.clone();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(win) = win_weak.upgrade() {
                        update_main_ui(&win, &state_snapshot, &config_snapshot);
                    }
                });
            }
        }));
    }

    // Start Status.json Watcher
    if let Some(ref s) = status {
        let state_c = Arc::clone(&state);
        let config_c = Arc::clone(&config);
        let win_weak_c = window.as_weak();

        let _ = s.start(Box::new(move |ship_status| {
            let mut st = lock_mutex(&state_c);
            let cfg = lock_mutex(&config_c);

            let changed = GameTracker::handle_status(&mut st, ship_status);
            if changed {
                let state_snapshot = st.clone();
                let config_snapshot = cfg.clone();
                let win_weak = win_weak_c.clone();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(win) = win_weak.upgrade() {
                        update_main_ui(&win, &state_snapshot, &config_snapshot);
                    }
                });
            }
        }));
    }

    // Register Global Hotkey
    let hotkey_win_weak = window.as_weak();
    let is_visible = Arc::new(Mutex::new(true));
    let is_visible_clone = Arc::clone(&is_visible);
    let hotkey_str = {
        let cfg = lock_mutex(&config);
        cfg.general.hotkey.clone()
    };

    let _ = hotkey.register(
        &hotkey_str,
        Box::new(move || {
            let win_weak = hotkey_win_weak.clone();
            let is_visible = Arc::clone(&is_visible_clone);

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(win) = win_weak.upgrade() {
                    let mut vis = lock_mutex(&is_visible);
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
        }),
    );

    show_window(window, move |win| {
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
            let config_store = Arc::clone(&config_store);
            let state = Arc::clone(&state);
            let spansh = Arc::clone(&spansh);
            let main_weak = win.as_weak();
            move || {
                show_settings_window(
                    Arc::clone(&config),
                    Arc::clone(&config_store),
                    Arc::clone(&state),
                    Arc::clone(&spansh),
                    main_weak.clone(),
                );
            }
        });

        // Next Hop callback
        win.on_next_hop({
            let state = Arc::clone(&state);
            let config = Arc::clone(&config);
            let trade_coord = Arc::clone(&trade_coord);
            let w = win.as_weak();
            move || {
                let mut st = lock_mutex(&state);
                ActionHandler::next_hop(&mut st);
                if let Some(ref route) = st.active_route {
                    trade_coord.save_cached_step(route, st.current_step_index);
                }
                let cfg = lock_mutex(&config);
                if let Some(win) = w.upgrade() {
                    update_main_ui(&win, &st, &cfg);
                }
            }
        });

        // Prev Hop callback
        win.on_prev_hop({
            let state = Arc::clone(&state);
            let config = Arc::clone(&config);
            let trade_coord = Arc::clone(&trade_coord);
            let w = win.as_weak();
            move || {
                let mut st = lock_mutex(&state);
                ActionHandler::prev_hop(&mut st);
                if let Some(ref route) = st.active_route {
                    trade_coord.save_cached_step(route, st.current_step_index);
                }
                let cfg = lock_mutex(&config);
                if let Some(win) = w.upgrade() {
                    update_main_ui(&win, &st, &cfg);
                }
            }
        });

        // Copy Target Destination System to Clipboard
        win.on_copy_dest_system({
            let state = Arc::clone(&state);
            let clipboard = Arc::clone(&clipboard);
            let w = win.as_weak();
            move || {
                let st = lock_mutex(&state);
                match ActionHandler::copy_target_system(&st, &*clipboard) {
                    Ok(dest_system) => {
                        if let Some(win) = w.upgrade() {
                            win.set_notification_message(
                                format!("Copied '{}'", dest_system).into(),
                            );
                            win.set_show_notification(true);

                            let w_clone = w.clone();
                            let spawn_res = slint::spawn_local(async move {
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                if let Some(win) = w_clone.upgrade() {
                                    win.set_show_notification(false);
                                }
                            });
                            if let Err(e) = spawn_res {
                                tracing::error!("Failed to spawn notification timer: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to copy to clipboard: {}", e);
                    }
                }
            }
        });

        // Clear Route callback
        win.on_clear_route({
            let state = Arc::clone(&state);
            let config = Arc::clone(&config);
            let trade_coord = Arc::clone(&trade_coord);
            let w = win.as_weak();
            move || {
                let mut st = lock_mutex(&state);
                ActionHandler::clear_route(&mut st);
                trade_coord.clear_cached_route();
                let cfg = lock_mutex(&config);
                if let Some(win) = w.upgrade() {
                    update_main_ui(&win, &st, &cfg);
                }
            }
        });

        // Search Route callback
        win.on_search_route({
            let state = Arc::clone(&state);
            let config = Arc::clone(&config);
            let trade_coord = Arc::clone(&trade_coord);
            let w = win.as_weak();

            move || {
                let mut search_cfg = {
                    let cfg = lock_mutex(&config);
                    cfg.search.clone()
                };

                // Override start location with live ED location if detected
                {
                    let st = lock_mutex(&state);
                    if let Some((system, station)) = ActionHandler::get_detected_location(&st) {
                        search_cfg.system = system;
                        if !station.is_empty() {
                            search_cfg.station = station;
                        }
                    }
                }

                {
                    let mut st = lock_mutex(&state);
                    st.is_searching = true;
                    st.last_error = None;
                    let cfg = lock_mutex(&config);
                    if let Some(win) = w.upgrade() {
                        update_main_ui(&win, &st, &cfg);
                    }
                }

                let state_clone = Arc::clone(&state);
                let config_clone = Arc::clone(&config);
                let trade_coord_clone = Arc::clone(&trade_coord);
                let w_clone = w.clone();

                let spawn_res = slint::spawn_local(async move {
                    match trade_coord_clone.find_route(&search_cfg).await {
                        Ok(steps) => {
                            tracing::info!("Found route with {} hops", steps.len());
                            let mut st = lock_mutex(&state_clone);
                            st.set_route(steps);
                        }
                        Err(e) => {
                            tracing::error!("Route search failed: {}", e);
                            let mut st = lock_mutex(&state_clone);
                            st.is_searching = false;
                            st.last_error = Some(format!("Search failed: {}", e));
                        }
                    }

                    let st = lock_mutex(&state_clone);
                    let cfg = lock_mutex(&config_clone);
                    if let Some(win) = w_clone.upgrade() {
                        update_main_ui(&win, &st, &cfg);
                    }
                });
                if let Err(e) = spawn_res {
                    tracing::error!("Failed to spawn route search task: {}", e);
                }
            }
        });
    });
}
