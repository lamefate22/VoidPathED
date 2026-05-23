use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN};
use tracing_appender::non_blocking::{WorkerGuard, NonBlockingBuilder};
use tracing_appender::rolling::{Rotation, RollingFileAppender};
use tracing_subscriber::fmt;

use crate::core::settings::TOMLoader;
use crate::error;

pub fn init_logger() -> WorkerGuard {
    if !std::path::PathBuf::from("logs").exists() {
        std::fs::create_dir("logs").expect("Failed to create logs dir");
    }

    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("voidpath")
        .filename_suffix("log")
        .max_log_files(3)
        .build("logs")
        .expect("Failed to init rolling file appender");

    let (non_blocking, guard) = NonBlockingBuilder::default()
        .finish(appender);

    fmt().with_writer(non_blocking).with_ansi(false).init();

    guard
}

pub fn init_config() -> Result<TOMLoader, error::Core> {
    let filepath = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.toml")));

    if let Some(filepath) = filepath {
        let mut config = TOMLoader::new(filepath);
        if !config.settings_path.exists() {
            config.save()?;
        }
        config.load()?;
        Ok(config)
    } else {
        tracing::error!("Failed to save/load config on init");
        Err(error::Core::TOMLError())
    }
}

pub fn show_window<T, F>(window: T, setup: F)
where
    T: slint::ComponentHandle + 'static,
    F: FnOnce(&T),
{
    window.show().unwrap();
    center_window_top(&window);
    setup(&window);
    window.run().unwrap();
}

pub fn center_window_top<T: slint::ComponentHandle + 'static>(window: &T) {
    let window_weak = window.as_weak();
    slint::invoke_from_event_loop(move || {
        if let Some(window) = window_weak.upgrade() {
            {
                let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                let win_w = window.window().size().width as i32;

                let x = (screen_w - win_w) / 2;
                let y = 0;

                window.window().set_position(slint::PhysicalPosition::new(x, y));
            }
        }
    }).unwrap();
}
