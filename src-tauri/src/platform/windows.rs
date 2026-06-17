// platform/windows.rs — Win32 焦点跟踪与目标激活。
//
// 关键差异 vs macOS：
//   - Windows 默认拒绝非前台进程调用 SetForegroundWindow（仅闪烁任务栏）。
//     用 AttachThreadInput 把我们的输入队列附加到目标线程后再切，绕过限制。
//   - 跟踪 HWND 比 PID 更准（多窗口应用如 VS Code/Chrome）；HWND 不是 Send，
//     因此对外暴露成 isize。
//   - WS_EX_NOACTIVATE 会让 webview 拿不到键盘焦点 —— 主 drawer 不能用它。
use tauri::WebviewWindow;
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber;
use windows_sys::Win32::System::Threading::AttachThreadInput;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GetForegroundWindow, GetWindowLongPtrW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow,
    SetWindowLongPtrW, ShowWindow, GWL_EXSTYLE, SW_RESTORE, WS_EX_TOOLWINDOW,
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
        // 仅加 WS_EX_TOOLWINDOW（不在 Alt-Tab 列表）。
        // 不加 WS_EX_NOACTIVATE — 会让 webview 永远拿不到键盘焦点。
        let cur = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let new_style = cur | (WS_EX_TOOLWINDOW as isize);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
    }
    tracing::info!("Windows ex-style applied to {}", window.label());
}

/// 剪贴板序列号；每次内容变化都会递增。轮询它判断是否有新复制。
pub fn clipboard_sequence() -> i64 {
    unsafe { GetClipboardSequenceNumber() as i64 }
}

pub fn foreground_pid() -> Option<i32> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid as *mut u32);
        if pid == 0 {
            None
        } else {
            Some(pid as i32)
        }
    }
}

/// 返回当前前台窗口的 HWND（编码为 isize；HWND 不是 Send）。
pub fn foreground_hwnd() -> Option<isize> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            None
        } else {
            Some(hwnd as isize)
        }
    }
}

/// 把指定 HWND 拉到前台。返回是否成功。
/// 用 AttachThreadInput 绕过 SetForegroundWindow 的前台进程限制。
pub fn activate_hwnd(hwnd_raw: isize) -> bool {
    if hwnd_raw == 0 {
        return false;
    }
    let target: HWND = hwnd_raw as HWND;
    unsafe {
        if IsWindowVisible(target) == 0 {
            tracing::warn!("activate_hwnd: target hwnd not visible");
            return false;
        }

        let mut pid: u32 = 0;
        let target_tid = GetWindowThreadProcessId(target, &mut pid as *mut u32);
        let fg = GetForegroundWindow();
        let fg_tid = if fg.is_null() {
            0
        } else {
            GetWindowThreadProcessId(fg, std::ptr::null_mut())
        };
        let our_tid = windows_sys::Win32::System::Threading::GetCurrentThreadId();

        // 如果窗口最小化了先还原。
        if IsIconic(target) != 0 {
            ShowWindow(target, SW_RESTORE);
        }

        // 把三方线程附加到一起，输入队列共享 → 才能 SetForegroundWindow。
        let attached_fg = if fg_tid != 0 && fg_tid != our_tid {
            AttachThreadInput(our_tid, fg_tid, 1) != 0
        } else {
            false
        };
        let attached_tg = if target_tid != 0 && target_tid != our_tid && target_tid != fg_tid {
            AttachThreadInput(our_tid, target_tid, 1) != 0
        } else {
            false
        };

        let ok = SetForegroundWindow(target) != 0;
        BringWindowToTop(target);

        if attached_tg {
            AttachThreadInput(our_tid, target_tid, 0);
        }
        if attached_fg {
            AttachThreadInput(our_tid, fg_tid, 0);
        }

        tracing::info!(ok, target_tid, fg_tid, "activate_hwnd");
        ok
    }
}

/// 备用路径：通过 PID 找一个顶层窗口激活（HWND 不可用时）。
pub fn activate_pid(pid: i32) -> bool {
    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let slot = lparam as *mut (i32, isize);
        let target_pid = unsafe { (*slot).0 };
        let mut pid: u32 = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut pid as *mut u32);
        }
        if pid as i32 == target_pid && unsafe { IsWindowVisible(hwnd) } != 0 {
            // 优先取没有 owner 的顶层窗口（主窗口而非弹出/工具窗）。
            unsafe { (*slot).1 = hwnd as isize };
            return 0;
        }
        1
    }
    let mut slot: (i32, isize) = (pid, 0);
    unsafe {
        EnumWindows(Some(enum_proc), &mut slot as *mut _ as LPARAM);
    }
    if slot.1 == 0 {
        tracing::warn!(pid, "activate_pid: no visible top-level window for pid");
        return false;
    }
    activate_hwnd(slot.1)
}
