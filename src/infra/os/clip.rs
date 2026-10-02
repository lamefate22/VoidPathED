use crate::contract::ClipboardService;
use crate::error::CoreError;

#[derive(Default, Clone)]
pub struct OsClipboard;

impl OsClipboard {
    pub fn new() -> Self {
        Self
    }
}

impl ClipboardService for OsClipboard {
    fn set_text(&self, text: &str) -> Result<(), CoreError> {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::System::DataExchange::{
                CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
            };
            use windows::Win32::System::Memory::{
                GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock,
            };

            OpenClipboard(None)
                .map_err(|e| CoreError::Clipboard(format!("OpenClipboard failed: {}", e)))?;

            let _ = EmptyClipboard();

            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes_len = wide.len() * std::mem::size_of::<u16>();

            let h_global = match GlobalAlloc(GMEM_MOVEABLE, bytes_len) {
                Ok(h) => h,
                Err(e) => {
                    let _ = CloseClipboard();
                    return Err(CoreError::Clipboard(format!("GlobalAlloc failed: {}", e)));
                }
            };

            let ptr = GlobalLock(h_global);
            if ptr.is_null() {
                let _ = CloseClipboard();
                return Err(CoreError::Clipboard("GlobalLock failed".into()));
            }

            std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr as *mut u8, bytes_len);
            let _ = GlobalUnlock(h_global);

            const CF_UNICODETEXT: u32 = 13;
            if let Err(e) = SetClipboardData(CF_UNICODETEXT, Some(HANDLE(h_global.0))) {
                let _ = CloseClipboard();
                return Err(CoreError::Clipboard(format!(
                    "SetClipboardData failed: {}",
                    e
                )));
            }

            let _ = CloseClipboard();
            tracing::info!("Copied '{}' to clipboard", text);
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            tracing::info!("Clipboard copy (non-windows stub): {}", text);
            Ok(())
        }
    }
}
