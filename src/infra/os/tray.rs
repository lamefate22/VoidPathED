use crate::contract::TrayManager;

#[derive(Default)]
pub struct Win32TrayManager;

impl Win32TrayManager {
    pub fn new() -> Self {
        Self
    }
}

impl TrayManager for Win32TrayManager {
    fn show_tray_icon(&self) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::Shell::{
                NIF_ICON, NIF_TIP, NIM_ADD, NOTIFYICONDATAW, Shell_NotifyIconW,
            };
            use windows::Win32::UI::WindowsAndMessaging::{
                FindWindowW, IDI_APPLICATION, LoadIconW,
            };
            use windows::core::HSTRING;

            let title_h = HSTRING::from("VoidPath ED");
            if let Ok(hwnd) = FindWindowW(None, windows::core::PCWSTR(title_h.as_ptr()))
                && hwnd != HWND::default()
            {
                let hicon = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();
                let mut nid = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd,
                    uID: 1001,
                    uFlags: NIF_ICON | NIF_TIP,
                    hIcon: hicon,
                    ..Default::default()
                };
                let tip: Vec<u16> = "VoidPath ED\0".encode_utf16().collect();
                let len = tip.len().min(nid.szTip.len());
                nid.szTip[..len].copy_from_slice(&tip[..len]);
                let _ = Shell_NotifyIconW(NIM_ADD, &nid);
            }
        }
    }

    fn remove_tray_icon(&self) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::Shell::{NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW};
            use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
            use windows::core::HSTRING;

            let title_h = HSTRING::from("VoidPath ED");
            if let Ok(hwnd) = FindWindowW(None, windows::core::PCWSTR(title_h.as_ptr()))
                && hwnd != HWND::default()
            {
                let nid = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd,
                    uID: 1001,
                    ..Default::default()
                };
                let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
            }
        }
    }
}
