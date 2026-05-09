// platform/macos.rs — 把抽屉的 NSWindow 改造为 NSPanel + nonactivatingPanel。
//
// 这样按下全局快捷键时，抽屉浮窗弹出但 **不会激活 Prompt Hub 应用**，
// 之前聚焦的输入框继续保持焦点 → 注入流程能拿到正确的目标。
use objc2::msg_send;
use objc2::runtime::AnyObject;
use tauri::WebviewWindow;

// NSWindowStyleMask 常量
const NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL: u64 = 1 << 7;
const NS_WINDOW_STYLE_MASK_BORDERLESS: u64 = 0;
const NS_WINDOW_STYLE_MASK_RESIZABLE: u64 = 1 << 3;
// NSWindowCollectionBehavior
const NS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
const NS_COLLECTION_BEHAVIOR_FULLSCREEN_AUXILIARY: u64 = 1 << 8;

pub fn apply(window: &WebviewWindow) {
    // 拿到底层 NSWindow 指针
    let ns_window: *mut AnyObject = match window.ns_window() {
        Ok(handle) => handle as *mut AnyObject,
        Err(e) => {
            tracing::warn!("ns_window unavailable: {e}");
            return;
        }
    };
    if ns_window.is_null() {
        tracing::warn!("ns_window is null; skip panel conversion");
        return;
    }

    unsafe {
        // 设置 styleMask 为 borderless | resizable | nonactivatingPanel
        let mask: u64 = NS_WINDOW_STYLE_MASK_BORDERLESS
            | NS_WINDOW_STYLE_MASK_RESIZABLE
            | NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL;
        let _: () = msg_send![ns_window, setStyleMask: mask];

        // 让窗口在所有 Space + 全屏应用上方显示
        let behavior: u64 = NS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES
            | NS_COLLECTION_BEHAVIOR_FULLSCREEN_AUXILIARY;
        let _: () = msg_send![ns_window, setCollectionBehavior: behavior];

        // 抽屉本身不激活应用（关键）
        let _: () = msg_send![ns_window, setHidesOnDeactivate: false];
    }

    tracing::info!("macOS panel style applied to {}", window.label());
}
