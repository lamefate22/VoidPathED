#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GWL_EXSTYLE, GetSystemMetrics, GetWindowLongPtrW, SM_CXSCREEN, SM_CYSCREEN,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetWindowLongPtrW,
    SetWindowPos, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
};
#[cfg(target_os = "windows")]
use windows::core::HSTRING;

#[cfg(target_os = "windows")]
pub fn hide_window_from_taskbar(title: &str) {
    unsafe {
        let title_h = HSTRING::from(title);
        if let Ok(hwnd) = FindWindowW(None, windows::core::PCWSTR(title_h.as_ptr()))
            && hwnd != HWND::default()
        {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let new_ex_style =
                (ex_style | (WS_EX_TOOLWINDOW.0 as isize)) & !(WS_EX_APPWINDOW.0 as isize);
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
            tracing::debug!("Applied WS_EX_TOOLWINDOW to hide '{}' from taskbar", title);
        }
    }
}

#[cfg(target_os = "windows")]
pub fn set_click_through(title: &str, enable: bool) {
    unsafe {
        let title_h = HSTRING::from(title);
        if let Ok(hwnd) = FindWindowW(None, windows::core::PCWSTR(title_h.as_ptr()))
            && hwnd != HWND::default()
        {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let new_ex_style = if enable {
                ex_style | (WS_EX_TRANSPARENT.0 as isize)
            } else {
                ex_style & !(WS_EX_TRANSPARENT.0 as isize)
            };
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
            tracing::info!("Applied click-through ({}) to '{}'", enable, title);
        }
    }
}

#[cfg(target_os = "windows")]
pub fn drag_window(title: &str) {
    unsafe {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
        use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_SYSCOMMAND};

        let title_h = HSTRING::from(title);
        if let Ok(hwnd) = FindWindowW(None, windows::core::PCWSTR(title_h.as_ptr()))
            && hwnd != HWND::default()
        {
            let _ = ReleaseCapture();
            let _ = SendMessageW(hwnd, WM_SYSCOMMAND, Some(WPARAM(0xF012)), Some(LPARAM(0)));
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn drag_window(_title: &str) {}

pub fn pick_folder() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = 'Select Elite Dangerous Journal Directory'; if ($f.ShowDialog() -eq 'OK') { Write-Output $f.SelectedPath }",
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() && std::path::Path::new(&s).exists() {
                return Some(s);
            }
        }
    }
    None
}

pub fn position_window_at<T: slint::ComponentHandle + 'static>(
    window: &T,
    saved_x: Option<i32>,
    saved_y: Option<i32>,
) {
    #[cfg(target_os = "windows")]
    {
        let window_weak = window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window_weak.upgrade() {
                let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                let win_w = window.window().size().width as i32;

                let x = saved_x.unwrap_or_else(|| (screen_w - win_w) / 2);
                let y = saved_y.unwrap_or(0);

                window
                    .window()
                    .set_position(slint::PhysicalPosition::new(x, y));
            }
            hide_window_from_taskbar("VoidPath ED");
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, saved_x, saved_y);
    }
}

pub fn show_window<T, F>(window: T, setup: F)
where
    T: slint::ComponentHandle + 'static,
    F: FnOnce(&T),
{
    if let Err(e) = window.show() {
        tracing::error!("Failed to show window: {}", e);
    }
    center_window_top(&window);

    setup(&window);
    if let Err(e) = window.run() {
        tracing::error!("Slint event loop error: {}", e);
    }
}

pub fn show_child_window_centered<T, F>(window: T, setup: F, win_w: i32, win_h: i32)
where
    T: slint::ComponentHandle + 'static,
    F: FnOnce(&T),
{
    if let Err(e) = window.show() {
        tracing::error!("Failed to show child window: {}", e);
    }

    #[cfg(target_os = "windows")]
    {
        let window_weak = window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(window) = window_weak.upgrade() {
                let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };

                let x = (screen_w - win_w) / 2;
                let y = (screen_h - win_h) / 2;

                window
                    .window()
                    .set_position(slint::PhysicalPosition::new(x, y));
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

                window
                    .window()
                    .set_position(slint::PhysicalPosition::new(x, y));
            }
            hide_window_from_taskbar("VoidPath ED");
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
    }
}
