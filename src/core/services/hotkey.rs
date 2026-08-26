use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(target_os = "windows")]
use std::time::Duration;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_HOTKEY,
};

#[allow(dead_code)]
pub struct HotkeyManager {
    is_running: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl HotkeyManager {
    pub fn start<F>(callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let is_running = Arc::new(AtomicBool::new(true));

        #[cfg(target_os = "windows")]
        {
            let running_clone = Arc::clone(&is_running);
            let callback = Arc::new(callback);

            std::thread::spawn(move || unsafe {
                const HOTKEY_ID: i32 = 0x5601;
                let modifiers = HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0 | MOD_NOREPEAT.0);
                let vk_key = 0x56; // 'V' key

                if let Err(e) = RegisterHotKey(None, HOTKEY_ID, modifiers, vk_key) {
                    tracing::warn!("Failed to register global hotkey (Ctrl+Shift+V): {}", e);
                    return;
                }

                tracing::info!("Global hotkey (Ctrl+Shift+V) successfully registered");

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
            let _ = callback;
            tracing::info!("Global hotkeys are currently Windows-only");
        }

        Self { is_running }
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}
