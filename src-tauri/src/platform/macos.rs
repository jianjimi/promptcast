// platform/macos.rs — 把抽屉的 NSWindow 改造为非激活面板（NSPanel 风）。
//
// 关键：
//   1. nonactivatingPanel + setHidesOnDeactivate(false) → 显示时不抢焦点
//   2. setMovable + setMovableByWindowBackground → 用户可以拖动窗口
//      （对 transparent + decorations:false 的窗口必需，否则光靠 webkit-app-region
//       在 NSPanel 上不响应）
//   3. setLevel(NSFloatingWindowLevel) → 浮在普通应用窗口之上，但允许同应用内
//      其他窗口（编辑/设置）激活时上来
//   4. CollectionBehavior：跟随所有 Space + 全屏副窗
use objc2::msg_send;
use objc2::runtime::AnyObject;
use tauri::WebviewWindow;

// NSWindowStyleMask
const NS_WINDOW_STYLE_MASK_BORDERLESS: u64 = 0;
const NS_WINDOW_STYLE_MASK_RESIZABLE: u64 = 1 << 3;
const NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL: u64 = 1 << 7;

// NSWindowCollectionBehavior
const NS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
const NS_COLLECTION_BEHAVIOR_FULLSCREEN_AUXILIARY: u64 = 1 << 8;

// NSWindowLevel — 数值越大越靠前。
//   normal=0, floating=3, status=25, popUpMenu=101
const NS_FLOATING_WINDOW_LEVEL: i64 = 3;

pub fn apply(window: &WebviewWindow) {
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
        // styleMask：borderless | resizable | nonactivatingPanel
        let mask: u64 = NS_WINDOW_STYLE_MASK_BORDERLESS
            | NS_WINDOW_STYLE_MASK_RESIZABLE
            | NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL;
        let _: () = msg_send![ns_window, setStyleMask: mask];

        let behavior: u64 = NS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES
            | NS_COLLECTION_BEHAVIOR_FULLSCREEN_AUXILIARY;
        let _: () = msg_send![ns_window, setCollectionBehavior: behavior];

        let _: () = msg_send![ns_window, setHidesOnDeactivate: false];

        // 让用户能拖动：从空白处拖动整窗（webkit-app-region:drag 在
        // NSPanel/borderless 下不可靠，这是 macOS 端的兜底）。
        let _: () = msg_send![ns_window, setMovable: true];
        let _: () = msg_send![ns_window, setMovableByWindowBackground: true];

        // 浮在普通窗口之上但不到 status 级别，编辑/设置窗口激活时仍能上来。
        let _: () = msg_send![ns_window, setLevel: NS_FLOATING_WINDOW_LEVEL];
    }

    tracing::info!("macOS panel style applied to {}", window.label());
}
