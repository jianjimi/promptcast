// platform/macos.rs — 留作 M5 进一步实现（NSPanel + nonactivatingPanel）。
// 当前为 no-op：透明 + decorations:false 已能给到接近 panel 的体验。
use tauri::WebviewWindow;

pub fn apply(_w: &WebviewWindow) {
    // no-op
}
