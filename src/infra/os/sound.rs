use crate::contract::SoundPlayer;

#[derive(Default)]
pub struct Win32SoundPlayer;

impl Win32SoundPlayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBeep(uType: u32) -> i32;
}

const MB_OK: u32 = 0x0000_0000;
const MB_ICONASTERISK: u32 = 0x0000_0040;
const MB_ICONEXCLAMATION: u32 = 0x0000_0030;

impl SoundPlayer for Win32SoundPlayer {
    fn play_success(&self) {
        #[cfg(target_os = "windows")]
        unsafe {
            let _ = MessageBeep(MB_OK);
        }
    }

    fn play_advance(&self) {
        #[cfg(target_os = "windows")]
        unsafe {
            let _ = MessageBeep(MB_ICONASTERISK);
        }
    }

    fn play_error(&self) {
        #[cfg(target_os = "windows")]
        unsafe {
            let _ = MessageBeep(MB_ICONEXCLAMATION);
        }
    }
}
