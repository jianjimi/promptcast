// platform/windows.rs — Win32 焦点跟踪。
use tauri::WebviewWindow;
use windows_sys::Win32::Foundation::{HWND, BOOL};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
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
    tracing::info!("Windows ex-style applied to {}", window.label());
}

pub fn foreground_pid() -> Option<i32> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() { return None; }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid as *mut u32);
        if pid == 0 { None } else { Some(pid as i32) }
    }
}

pub fn activate_pid(pid: i32) -> bool {
    // Win32 没有直接通过 PID 激活的 API；遍历窗口找匹配 PID 的顶层窗口。
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, IsWindowVisible,
    };
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: isize) -> BOOL {
        let target_pid = lparam as i32;
        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid as *mut u32); }
        if pid as i32 == target_pid && unsafe { IsWindowVisible(hwnd) } != 0 {
            unsafe { SetForegroundWindow(hwnd); }
            return 0; // 终止枚举
        }
        1
    }
    unsafe {
        EnumWindows(Some(enum_proc), pid as isize);
    }
    true
}
