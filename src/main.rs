use std::sync::{Arc, Mutex};

use crate::core::models::state::AppState;
use crate::core::utils::{init_config, init_logger};
use crate::show::{open_init_window, open_main_window};

mod api;
mod core;
mod error;
mod show;

#[tokio::main]
async fn main() {
    let _guard = init_logger();
    tracing::info!("Starting VoidPath ED...");

    // Show initial connectivity splash check
    open_init_window();

    match init_config() {
        Ok(config) => {
            tracing::info!("Config loaded successfully");
            let config = Arc::new(Mutex::new(config));
            let state = Arc::new(Mutex::new(AppState::new()));
            open_main_window(config, state);
        }
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
        }
    }
}
