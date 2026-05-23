use slint::ComponentHandle;

use crate::api::client::ApiClient;
use crate::core::utils::*;

mod error;
mod core;
mod api;

slint::include_modules!();

#[tokio::main]
async fn main() {
    let _guard = init_logger();
    tracing::info!("Starting VoidPath...");

    let init_window = InitWindow::new().unwrap();

    show_window(init_window, |win| {
        win.on_close_window({
            let w = win.as_weak();
            move || { w.unwrap().hide().unwrap(); }
        });

        let window_weak = win.as_weak();
        slint::spawn_local(async move {
            tracing::info!("Running API connectivity check...");
            let client = ApiClient::new();
            let win = window_weak.unwrap();

            match client.check_api_connectivity().await {
                Ok(_) => {
                    tracing::info!("Connectivity status: OK");
                    win.hide().unwrap();
                }
                Err(e) => {
                    tracing::error!("Connectivity status: ERR — {}", e);
                    win.set_is_loading(false);
                    win.set_is_error(true);
                    win.set_error_message(e.to_string().into());
                }
            }
        }).unwrap();
    });

    match init_config() {
        Ok(_config) => {
            tracing::info!("Config loaded successfully");
            let main_window = MainWindow::new().unwrap();
            show_window(main_window, |win| {
                win.on_close_window({
                    let w = win.as_weak();
                    move || { w.unwrap().hide().unwrap(); }
                });
            });
        }
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
        }
    }
}
