use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::contract::HotkeyListener;
use crate::error::CoreError;

#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, RegisterHotKey,
    UnregisterHotKey,
};

#[cfg(target_os = "windows")]
pub fn parse_hotkey(s: &str) -> Option<(HOT_KEY_MODIFIERS, u32)> {
    let mut modifiers = MOD_NOREPEAT.0;
    let mut vk_key: Option<u32> = None;

    for part in s.split('+').map(|p| p.trim().to_uppercase()) {
        match part.as_str() {
            "CTRL" | "CONTROL" => modifiers |= MOD_CONTROL.0,
            "SHIFT" => modifiers |= MOD_SHIFT.0,
            "ALT" => modifiers |= MOD_ALT.0,
            "WIN" | "WINDOWS" | "SUPER" => modifiers |= MOD_WIN.0,
            f if f.starts_with('F') && f.len() > 1 => {
                if let Ok(n) = f[1..].parse::<u32>()
                    && (1..=24).contains(&n)
                {
                    vk_key = Some(0x70 + (n - 1)); // VK_F1 is 0x70
                }
            }
            "SPACE" => vk_key = Some(0x20),
            "TAB" => vk_key = Some(0x09),
            "ESC" | "ESCAPE" => vk_key = Some(0x1B),
            single if single.len() == 1 => {
                if let Some(ch) = single.chars().next()
                    && ch.is_ascii_alphanumeric()
                {
                    vk_key = Some(ch.to_ascii_uppercase() as u32);
                }
            }
            _ => {}
        }
    }

    vk_key.map(|key| (HOT_KEY_MODIFIERS(modifiers), key))
}

pub struct OsHotkeyListener {
    is_running: Arc<AtomicBool>,
}

impl OsHotkeyListener {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for OsHotkeyListener {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyListener for OsHotkeyListener {
    fn register(
        &self,
        shortcut: &str,
        callback: Box<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<(), CoreError> {
        self.stop();
        self.is_running.store(true, Ordering::SeqCst);

        #[cfg(target_os = "windows")]
        {
            use std::time::Duration;
            use windows::Win32::UI::WindowsAndMessaging::{
                DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_HOTKEY,
            };

            let running_clone = Arc::clone(&self.is_running);
            let callback = Arc::new(callback);
            let (modifiers, vk_key) = parse_hotkey(shortcut).unwrap_or_else(|| {
                tracing::warn!(
                    "Failed to parse hotkey '{}', falling back to Ctrl+Shift+V",
                    shortcut
                );
                (
                    HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0 | MOD_NOREPEAT.0),
                    0x56, // 'V'
                )
            });

            let shortcut_display = shortcut.to_string();

            std::thread::spawn(move || unsafe {
                const HOTKEY_ID: i32 = 0x5601;

                if let Err(e) = RegisterHotKey(None, HOTKEY_ID, modifiers, vk_key) {
                    tracing::warn!(
                        "Failed to register global hotkey ({}): {}",
                        shortcut_display,
                        e
                    );
                    return;
                }

                tracing::info!(
                    "Global hotkey ({}) successfully registered",
                    shortcut_display
                );

                let mut msg = MSG::default();
                while running_clone.load(Ordering::SeqCst) {
                    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                        if msg.message == WM_HOTKEY && msg.wParam.0 == HOTKEY_ID as usize {
                            tracing::info!("Global hotkey triggered -> toggling overlay");
                            callback();
                        }
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }

                let _ = UnregisterHotKey(None, HOTKEY_ID);
                tracing::info!("Global hotkey unregistered");
            });
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (shortcut, callback);
            tracing::info!("Global hotkeys are currently Windows-only");
        }

        Ok(())
    }

    fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}
