// platform — 跨平台窗口与注入封装。
// 焦点不抢的实现因平台而异：
//   - macOS：把 NSWindow 转 NSPanel + nonactivatingPanel style mask
//   - Windows：SetWindowLongPtr 加 WS_EX_NOACTIVATE
// MVP 暂时实现为最小版本：主流程依赖"按 Enter → 立即 hide → 等 80ms → 模拟粘贴"
// 来规避焦点冲突；NSPanel/EX_NOACTIVATE 留给后续打磨。
use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn apply_panel_style(_window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    macos::apply(_window);
    #[cfg(target_os = "windows")]
    windows::apply(_window);
}
