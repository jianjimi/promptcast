// platform/macos.rs — NSPanel 改造 + 跨应用焦点跟踪。
//
// 核心思路（Raycast 同款）：
//   1. 抽屉是 NSPanel + nonactivatingPanel：显示时不会改变系统的 frontmost app
//   2. 但在 show() 之后我们 **手动 makeKeyAndOrderFront** —— 这能让面板成为
//      key window 接收键盘输入，但不会抢 frontmost。这样用户：
//        • 抽屉里能用 ↑↓ Enter
//        • 浏览器仍是 frontmost（关键！）
//   3. 按下全局快捷键的瞬间，记住 NSWorkspace.frontmostApplication 的 PID
//   4. inject 时：hide drawer → activate 那个 PID → sleep → 模拟 Cmd+V
//
//   这样即使中途用户用鼠标点了抽屉（这会让 NSApp 短暂激活），我们也能
//   在注入前把目标应用拉回前台。
use objc2::{class, msg_send};
use objc2::runtime::AnyObject;
use tauri::WebviewWindow;

const NS_WINDOW_STYLE_MASK_BORDERLESS: u64 = 0;
const NS_WINDOW_STYLE_MASK_RESIZABLE: u64 = 1 << 3;
const NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL: u64 = 1 << 7;

const NS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
const NS_COLLECTION_BEHAVIOR_FULLSCREEN_AUXILIARY: u64 = 1 << 8;

const NS_FLOATING_WINDOW_LEVEL: i64 = 3;

const NS_APPLICATION_ACTIVATE_IGNORING_OTHER_APPS: u64 = 1 << 1;

pub fn apply_panel_style(window: &WebviewWindow) {
    let ns_window: *mut AnyObject = match window.ns_window() {
        Ok(handle) => handle as *mut AnyObject,
        Err(e) => {
            tracing::warn!("ns_window unavailable: {e}");
            return;
        }
    };
    if ns_window.is_null() {
        tracing::warn!("ns_window is null");
        return;
    }
    unsafe {
        let mask: u64 = NS_WINDOW_STYLE_MASK_BORDERLESS
            | NS_WINDOW_STYLE_MASK_RESIZABLE
            | NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL;
        let _: () = msg_send![ns_window, setStyleMask: mask];
        let behavior: u64 = NS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES
            | NS_COLLECTION_BEHAVIOR_FULLSCREEN_AUXILIARY;
        let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
        let _: () = msg_send![ns_window, setHidesOnDeactivate: false];
        let _: () = msg_send![ns_window, setMovable: true];
        let _: () = msg_send![ns_window, setMovableByWindowBackground: true];
        let _: () = msg_send![ns_window, setLevel: NS_FLOATING_WINDOW_LEVEL];
    }
    tracing::info!("macOS panel style applied to {}", window.label());
}

/// 让面板成为 key window：键盘事件路由到它。
/// 对 nonactivatingPanel 不会激活整个 NSApp。
pub fn make_key_and_order_front(window: &WebviewWindow) {
    let ns_window: *mut AnyObject = match window.ns_window() {
        Ok(h) => h as *mut AnyObject,
        Err(_) => return,
    };
    if ns_window.is_null() { return; }
    unsafe {
        let nil: *const AnyObject = std::ptr::null();
        let _: () = msg_send![ns_window, makeKeyAndOrderFront: nil];
    }
}

/// 当前 frontmost app 的 PID（NSWorkspace.shared.frontmostApplication.processIdentifier）。
pub fn frontmost_app_pid() -> Option<i32> {
    unsafe {
        let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace.is_null() { return None; }
        let app: *mut AnyObject = msg_send![workspace, frontmostApplication];
        if app.is_null() { return None; }
        let pid: i32 = msg_send![app, processIdentifier];
        if pid <= 0 { None } else { Some(pid) }
    }
}

/// 通过 PID 把某应用拉到前台。
pub fn activate_app_by_pid(pid: i32) -> bool {
    unsafe {
        let app: *mut AnyObject = msg_send![
            class!(NSRunningApplication),
            runningApplicationWithProcessIdentifier: pid
        ];
        if app.is_null() {
            tracing::warn!(pid, "runningApplicationWithProcessIdentifier returned nil");
            return false;
        }
        let opts: u64 = NS_APPLICATION_ACTIVATE_IGNORING_OTHER_APPS;
        let activated: bool = msg_send![app, activateWithOptions: opts];
        tracing::info!(pid, activated, "activate_app_by_pid");
        activated
    }
}
