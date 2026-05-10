// platform — 跨平台窗口与焦点封装。
use tauri::WebviewWindow;

pub mod permissions;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn apply_panel_style(_window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    macos::apply_panel_style(_window);
    #[cfg(target_os = "windows")]
    windows::apply(_window);
}

/// 让窗口成为 key window 接收键盘事件。
pub fn make_key(_window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    macos::make_key_and_order_front(_window);
    // Windows 上 show() + set_focus() 已能 key 化，无需额外操作
}

/// 返回当前 frontmost / foreground app 的 PID。
pub fn frontmost_app_pid() -> Option<i32> {
    #[cfg(target_os = "macos")]
    { return macos::frontmost_app_pid(); }
    #[cfg(target_os = "windows")]
    { return windows::foreground_pid(); }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { None }
}

/// 把指定 PID 的进程拉到前台。
pub fn activate_app_by_pid(_pid: i32) -> bool {
    #[cfg(target_os = "macos")]
    { return macos::activate_app_by_pid(_pid); }
    #[cfg(target_os = "windows")]
    { return windows::activate_pid(_pid); }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { false }
}
