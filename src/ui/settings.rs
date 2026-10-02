use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::contract::{ConfigStore, HotkeyListener, SpanshClient};
use crate::domain::config::AppConfig;
use crate::domain::state::AppState;
use crate::infra::os::window::show_child_window_centered;
use crate::ui::mapper::format_location;
use crate::ui::{MainWindow, SettingsWindow, StationSuggestion};

fn lock_mutex<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match m.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub fn show_settings_window(
    config: Arc<Mutex<AppConfig>>,
    config_store: Arc<dyn ConfigStore>,
    state: Arc<Mutex<AppState>>,
    spansh: Arc<dyn SpanshClient>,
    hotkey: Arc<dyn HotkeyListener>,
    toggle_overlay: Arc<dyn Fn() + Send + Sync + 'static>,
    main_window_weak: slint::Weak<MainWindow>,
) -> slint::Weak<SettingsWindow> {
    let window = match SettingsWindow::new() {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Failed to create SettingsWindow: {}", e);
            return slint::Weak::default();
        }
    };

    // Populate Settings Window fields from config
    {
        let cfg = lock_mutex(&config);
        let s = &cfg.search;
        let g = &cfg.general;

        window.set_starting_point(format_location(&s.system, &s.station).into());
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

        window.set_auto_advance_route(g.auto_advance_route);
        window.set_auto_sync_ship(g.auto_sync_ship);
        window.set_auto_copy_system(g.auto_copy_system);
        window.set_sound_enabled(g.sound_enabled);
        window.set_hotkey_str(g.hotkey.clone().into());
        window.set_journal_path(g.journal_path.clone().into());

        let st = lock_mutex(&state);
        if let Some(ship) = &st.current_ship {
            let name = ship.ship_name.trim();
            let label = if name.is_empty() || name.eq_ignore_ascii_case(&ship.ship_type) {
                ship.ship_type.clone()
            } else {
                format!("\"{}\" ({})", name, ship.ship_type)
            };
            let info = format!(
                "{} ({}t | {:.1} LY | {:?})",
                label, ship.cargo_capacity, ship.max_jump_range, ship.pad_size
            );
            window.set_detected_ship_info(info.into());
        }
    }

    let weak_win = window.as_weak();
    show_child_window_centered(
        window,
        move |win| {
            // Drag window callback
            win.on_window_drag_started(|| {
                crate::infra::os::window::drag_window("VoidPath ED Settings");
            });

            // Close window callback
            win.on_close_window({
                let w = win.as_weak();
                let hotkey = Arc::clone(&hotkey);
                let toggle = Arc::clone(&toggle_overlay);
                let config = Arc::clone(&config);
                move || {
                    if let Some(w) = w.upgrade() {
                        if w.get_is_recording_hotkey() {
                            w.set_is_recording_hotkey(false);
                        }
                        let cfg = lock_mutex(&config);
                        let t = toggle.clone();
                        let _ = hotkey.register(&cfg.general.hotkey, Box::new(move || t()));
                        let _ = w.hide();
                    }
                }
            });

            // Hotkey recording started callback (unregisters hotkey so keys reach SettingsWindow)
            win.on_hotkey_recording_started({
                let hotkey = Arc::clone(&hotkey);
                move || {
                    hotkey.stop();
                }
            });

            // Hotkey recording ended callback (restores active hotkey)
            win.on_hotkey_recording_ended({
                let hotkey = Arc::clone(&hotkey);
                let toggle = Arc::clone(&toggle_overlay);
                let config = Arc::clone(&config);
                let w = win.as_weak();
                move || {
                    let current = if let Some(win) = w.upgrade() {
                        win.get_hotkey_str().to_string()
                    } else {
                        let cfg = lock_mutex(&config);
                        cfg.general.hotkey.clone()
                    };
                    let t = toggle.clone();
                    let _ = hotkey.register(&current, Box::new(move || t()));
                }
            });

            // Auto-detect current system/station from Journal state
            win.on_auto_detect_location({
                let state = Arc::clone(&state);
                let w = win.as_weak();
                move || {
                    let st = lock_mutex(&state);
                    if !st.current_system.is_empty()
                        && let Some(win) = w.upgrade()
                    {
                        let system = &st.current_system;
                        let station = if !st.current_station.is_empty() {
                            &st.current_station
                        } else {
                            ""
                        };

                        win.set_selected_system(system.clone().into());
                        win.set_selected_station(station.to_string().into());
                        win.set_starting_point(format_location(system, station).into());
                        win.set_show_suggestions(false);
                        tracing::info!("Auto-detected ED location: {} / {}", system, station);
                    }
                }
            });

            // Station query input callback (debounced search via Spansh API)
            win.on_station_query_changed({
                let w = win.as_weak();
                let spansh = Arc::clone(&spansh);
                let request_id = Arc::new(std::sync::atomic::AtomicU64::new(0));

                move |query| {
                    let w = w.clone();
                    let spansh = Arc::clone(&spansh);
                    let request_id = Arc::clone(&request_id);
                    let current_id =
                        request_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

                    let spawn_res = slint::spawn_local(async move {
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

                        match spansh.search_stations(&query).await {
                            Ok(stations) => {
                                if request_id.load(std::sync::atomic::Ordering::Relaxed)
                                    != current_id
                                {
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
                    });
                    if let Err(e) = spawn_res {
                        tracing::error!("Failed to spawn station search task: {}", e);
                    }
                }
            });

            // Station selected callback
            win.on_station_selected({
                let w = win.as_weak();
                move |system, station| {
                    if let Some(win) = w.upgrade() {
                        win.set_selected_system(system.clone());
                        win.set_selected_station(station.clone());
                        win.set_starting_point(format_location(&system, &station).into());
                        win.set_show_suggestions(false);
                    }
                }
            });

            // Save settings callback
            win.on_save({
                let w = win.as_weak();
                let config = Arc::clone(&config);
                let config_store = Arc::clone(&config_store);
                let state = Arc::clone(&state);
                let hotkey = Arc::clone(&hotkey);
                let toggle_overlay = Arc::clone(&toggle_overlay);
                let main_weak = main_window_weak.clone();

                move || {
                    let win = match w.upgrade() {
                        Some(w) => w,
                        None => return,
                    };

                    let mut cfg = lock_mutex(&config);
                    let s = &mut cfg.search;

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

                    let saved_hotkey = win.get_hotkey_str().to_string();
                    {
                        let g = &mut cfg.general;
                        g.auto_advance_route = win.get_auto_advance_route();
                        g.auto_sync_ship = win.get_auto_sync_ship();
                        g.auto_copy_system = win.get_auto_copy_system();
                        g.sound_enabled = win.get_sound_enabled();
                        g.hotkey = saved_hotkey.clone();
                        g.journal_path = win.get_journal_path().to_string();
                    }

                    if let Err(e) = config_store.save(&cfg) {
                        tracing::error!("Failed to save config: {}", e);
                    } else {
                        tracing::info!("Config saved successfully");
                    }

                    // Re-register hotkey with updated configuration
                    let t = toggle_overlay.clone();
                    let _ = hotkey.register(&saved_hotkey, Box::new(move || t()));

                    // Update main UI with updated config
                    if let Some(main_win) = main_weak.upgrade() {
                        let st = lock_mutex(&state);
                        crate::ui::main::update_main_ui(&main_win, &st, &cfg);
                    }

                    let _ = win.hide();
                }
            });

            // Browse custom journal path callback
            win.on_browse_journal_path({
                let w = win.as_weak();
                move || {
                    let w = w.clone();
                    std::thread::spawn(move || {
                        if let Some(folder) = crate::infra::os::window::pick_folder() {
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(win) = w.upgrade() {
                                    win.set_journal_path(folder.into());
                                }
                            });
                        }
                    });
                }
            });

            // Hotkey recording callback
            win.on_hotkey_key_pressed({
                let w = win.as_weak();
                let hotkey = Arc::clone(&hotkey);
                let toggle = Arc::clone(&toggle_overlay);
                let config = Arc::clone(&config);
                move |text, ctrl, alt, shift, win_mod| {
                    if let Some(win) = w.upgrade() {
                        if text == "\u{1b}" {
                            win.set_is_recording_hotkey(false);
                            let cfg = lock_mutex(&config);
                            let t = toggle.clone();
                            let _ = hotkey.register(&cfg.general.hotkey, Box::new(move || t()));
                            return;
                        }
                        if let Some(shortcut) = crate::ui::mapper::format_hotkey_from_event(
                            &text, ctrl, alt, shift, win_mod,
                        ) {
                            win.set_hotkey_str(shortcut.clone().into());
                            win.set_is_recording_hotkey(false);
                            let t = toggle.clone();
                            let _ = hotkey.register(&shortcut, Box::new(move || t()));
                        }
                    }
                }
            });
        },
        460,
        580,
    );
    weak_win
}
