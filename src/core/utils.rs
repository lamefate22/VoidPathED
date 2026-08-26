use std::path::PathBuf;
use tracing_appender::non_blocking::{NonBlockingBuilder, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

#[cfg(target_os = "windows")]
use windows::core::HSTRING;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetSystemMetrics, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE,
    SM_CXSCREEN, SM_CYSCREEN, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

use crate::core::settings::TOMLoader;
use crate::error;

pub fn get_app_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn init_logger() -> WorkerGuard {
    let logs_dir = get_app_dir().join("logs");
    let _ = std::fs::create_dir_all(&logs_dir);

    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("voidpath")
        .filename_suffix("log")
        .max_log_files(3)
        .build(&logs_dir)
        .expect("Failed to init rolling file appender");

    let (non_blocking, guard) = NonBlockingBuilder::default().finish(appender);

    fmt().with_writer(non_blocking).with_ansi(false).init();

    guard
}

pub fn init_config() -> Result<TOMLoader, error::Core> {
    let filepath = get_app_dir().join("config.toml");
    let mut config = TOMLoader::new(filepath);
    if !config.settings_path.exists() {
        config.save()?;
    } else if let Err(e) = config.load() {
        tracing::warn!("Failed to parse existing config, falling back to default: {}", e);
        let _ = config.save();
    }
    Ok(config)
}

#[cfg(target_os = "windows")]
pub fn hide_window_from_taskbar(title: &str) {
    unsafe {
        let title_h = HSTRING::from(title);
        if let Ok(hwnd) = FindWindowW(None, windows::core::PCWSTR(title_h.as_ptr())) {
            if hwnd != HWND::default() {
                let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let new_ex_style = (ex_style | (WS_EX_TOOLWINDOW.0 as isize)) & !(WS_EX_APPWINDOW.0 as isize);
                let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
                );
                tracing::info!("Applied WS_EX_TOOLWINDOW to hide '{}' from taskbar", title);
            }
        }
    }
}

pub fn show_window<T, F>(window: T, setup: F)
where
    T: slint::ComponentHandle + 'static,
    F: FnOnce(&T),
{
    window.show().unwrap();
    center_window_top(&window);

    #[cfg(target_os = "windows")]
    {
        let _ = slint::invoke_from_event_loop(|| {
            hide_window_from_taskbar("VoidPath ED");
        });
    }

    setup(&window);
    window.run().unwrap();
}

pub fn show_child_window_centered<T, F>(window: T, setup: F, win_w: i32, win_h: i32)
where
    T: slint::ComponentHandle + 'static,
    F: FnOnce(&T),
{
    window.show().unwrap();

    #[cfg(target_os = "windows")]
    {
        let window_weak = window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window_weak.upgrade() {
                let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };

                let x = (screen_w - win_w) / 2;
                let y = (screen_h - win_h) / 2;

                window.window().set_position(slint::PhysicalPosition::new(x, y));
            }
            hide_window_from_taskbar("VoidPath ED Settings");
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (win_w, win_h);
    }

    setup(&window);
}

pub fn center_window_top<T: slint::ComponentHandle + 'static>(window: &T) {
    #[cfg(target_os = "windows")]
    {
        let window_weak = window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window_weak.upgrade() {
                let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                let win_w = window.window().size().width as i32;

                let x = (screen_w - win_w) / 2;
                let y = 0;

                window.window().set_position(slint::PhysicalPosition::new(x, y));
            }
            hide_window_from_taskbar("VoidPath ED");
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
    }
}
