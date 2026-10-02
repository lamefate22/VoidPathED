use slint::ComponentHandle;
use std::sync::Arc;

use crate::contract::SpanshClient;
use crate::infra::os::window::show_window;
use crate::ui::InitWindow;

pub fn show_init_window(spansh: Arc<dyn SpanshClient>) {
    let window = match InitWindow::new() {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Failed to instantiate InitWindow: {}", e);
            return;
        }
    };

    show_window(window, move |win| {
        win.on_close_window({
            let w = win.as_weak();
            move || {
                if let Some(w) = w.upgrade() {
                    let _ = w.hide();
                }
            }
        });

        let window_weak = win.as_weak();
        let spansh = Arc::clone(&spansh);

        let spawn_res = slint::spawn_local(async move {
            tracing::info!("Checking Spansh API connection...");
            let Some(win) = window_weak.upgrade() else {
                return;
            };

            match spansh.check_health().await {
                Ok(_) => {
                    tracing::info!("Spansh API: OK");
                    let _ = win.hide();
                }
                Err(e) => {
                    tracing::error!("Spansh API error: {}", e);
                    win.set_is_loading(false);
                    win.set_is_error(true);
                    win.set_error_message(e.to_string().into());
                }
            }
        });

        if let Err(e) = spawn_res {
            tracing::error!("Failed to spawn API check task: {}", e);
        }
    });
}
