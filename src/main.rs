use std::sync::{Arc, Mutex};

use voidpath_rs::app::TradeCoordinator;
use voidpath_rs::contract::{ConfigStore, JournalWatcher, StatusWatcher};
use voidpath_rs::domain::state::AppState;
use voidpath_rs::infra::fs::{JsonRouteCache, TomlConfigStore, get_app_dir, init_logger};
use voidpath_rs::infra::game::{GameJournalWatcher, GameStatusWatcher, resolve_journal_dir};
use voidpath_rs::infra::os::{OsClipboard, OsHotkeyListener};
use voidpath_rs::infra::spansh::SpanshHttpClient;
use voidpath_rs::ui::{show_init_window, show_main_window};

#[tokio::main]
async fn main() {
    let _guard = init_logger();
    tracing::info!("Starting VoidPath ED v1.0...");

    // 1. Initialize Infrastructure Adapters (Dependency Injection)
    let spansh = Arc::new(SpanshHttpClient::new());
    let config_path = get_app_dir().join("config.toml");
    let cache_path = get_app_dir().join("route_cache.json");

    let config_store = Arc::new(TomlConfigStore::new(config_path));
    let route_cache = Arc::new(JsonRouteCache::new(cache_path));
    let trade_coord = Arc::new(TradeCoordinator::new(
        spansh.clone(),
        Some(route_cache.clone()),
    ));
    let clipboard = Arc::new(OsClipboard::new());
    let hotkey = Arc::new(OsHotkeyListener::new());
    let sound = Arc::new(voidpath_rs::infra::os::Win32SoundPlayer::new());
    let tray = Arc::new(voidpath_rs::infra::os::Win32TrayManager::new());

    // 2. Load Configuration
    let config = match config_store.load() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("Failed to load configuration: {}", e);
            Default::default()
        }
    };

    // 3. Resolve Game Watchers (Journal & Status.json)
    let journal_dir = resolve_journal_dir(if config.general.journal_path.is_empty() {
        None
    } else {
        Some(&config.general.journal_path)
    });

    let journal_watcher: Option<Arc<dyn JournalWatcher>> = journal_dir
        .as_ref()
        .map(|dir| Arc::new(GameJournalWatcher::new(dir.clone())) as Arc<dyn JournalWatcher>);

    let status_watcher: Option<Arc<dyn StatusWatcher>> = journal_dir
        .as_ref()
        .map(|dir| Arc::new(GameStatusWatcher::new(dir.clone())) as Arc<dyn StatusWatcher>);

    // 4. Initialize State & Restore Cached Route if Available
    let mut app_state = AppState::new();
    if let Ok(Some((steps, current_step))) = trade_coord.load_cached_route() {
        tracing::info!(
            "Restored active route from cache: {} steps, current step {}",
            steps.len(),
            current_step
        );
        app_state.set_route(steps);
        app_state.current_step_index = current_step;
    }

    let config = Arc::new(Mutex::new(config));
    let state = Arc::new(Mutex::new(app_state));

    // 5. Connectivity Splash Check
    show_init_window(spansh.clone());

    // 6. Launch Main Window Overlay
    show_main_window(voidpath_rs::ui::MainWindowContext {
        config,
        config_store,
        state,
        trade_coord,
        spansh,
        clipboard,
        hotkey,
        sound,
        tray,
        journal: journal_watcher,
        status: status_watcher,
    });
}
