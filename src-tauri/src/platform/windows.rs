// platform/windows.rs — 设 WS_EX_NOACTIVATE 让抽屉不抢焦点。
use tauri::WebviewWindow;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub fn apply(window: &WebviewWindow) {
    let hwnd: HWND = match window.hwnd() {
        Ok(h) => h.0 as HWND,
        Err(e) => {
            tracing::warn!("hwnd unavailable: {e}");
            return;
        }
    };
    unsafe {
        let cur = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let new_style = cur
            | (WS_EX_NOACTIVATE as isize)
            | (WS_EX_TOOLWINDOW as isize);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
    }
    tracing::info!("Windows ex-style WS_EX_NOACTIVATE applied to {}", window.label());
}
