// commands/inject.rs — 剪贴板写入 + 模拟粘贴。
//
// 流程：
//   1) 保存当前剪贴板（用于稍后还原）
//   2) 写新内容
//   3) AXIsProcessTrusted() 检查（macOS）；未授权直接 fallback
//   4) hide drawer（释放 key window）
//   5) 激活之前保存的目标窗口（Windows 优先 HWND，macOS 用 PID）
//   6) sleep ~120ms 让系统切焦稳定
//   7) enigo 模拟 Cmd/Ctrl+V
//   8) 异步线程 ~600ms 后还原原剪贴板
use std::{thread, time::Duration};

use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::platform::{self, permissions};
use crate::LastFrontmost;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InjectResult {
    pub ok: bool,
    pub fallback: Option<String>,
    pub message: Option<String>,
}

fn read_clipboard() -> Option<String> {
    arboard::Clipboard::new().ok().and_then(|mut cb| cb.get_text().ok())
}

fn write_clipboard(text: &str) -> AppResult<()> {
    let mut cb = arboard::Clipboard::new()
        .map_err(|e| AppError::Internal(format!("clipboard: {e}")))?;
    cb.set_text(text)
        .map_err(|e| AppError::Internal(format!("clipboard set: {e}")))?;
    Ok(())
}

/// 在后台线程延时还原剪贴板。失败仅打日志，不影响主流程。
fn schedule_clipboard_restore(prev: Option<String>, just_pasted: String, delay_ms: u64) {
    let Some(prev) = prev else { return };
    if prev == just_pasted {
        return;
    }
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay_ms));
        if let Ok(mut cb) = arboard::Clipboard::new() {
            // 仅在剪贴板内容仍是我们注入的那段时还原 —— 避免覆盖用户期间手动复制的内容。
            let still_ours = cb.get_text().ok().as_deref() == Some(just_pasted.as_str());
            if still_ours {
                if let Err(e) = cb.set_text(prev) {
                    tracing::warn!(error = %e, "clipboard restore failed");
                } else {
                    tracing::info!("clipboard restored");
                }
            } else {
                tracing::info!("clipboard changed by user; skipping restore");
            }
        }
    });
}

#[tauri::command]
pub fn inject_copy_only(content: String) -> AppResult<()> {
    let len = content.len();
    write_clipboard(&content)?;
    tracing::info!(len, "clipboard written (copy_only)");
    Ok(())
}

#[tauri::command]
pub fn inject_paste(app: AppHandle, content: String) -> AppResult<InjectResult> {
    let len = content.len();
    tracing::info!(len, "inject_paste begin");

    // 1) 保存当前剪贴板，便于注入后还原
    let prev_clipboard = read_clipboard();

    // 2) 写新内容
    if let Err(e) = write_clipboard(&content) {
        tracing::error!(error = %e, "clipboard write failed");
        return Ok(InjectResult {
            ok: false,
            fallback: None,
            message: Some(format!("clipboard write failed: {e}")),
        });
    }

    // 3) 权限（macOS：AXIsProcessTrusted；Windows：恒 true）
    let trusted = permissions::is_trusted();
    tracing::info!(trusted, "permissions check");
    if !trusted {
        if let Some(w) = app.get_webview_window("drawer") {
            let _ = w.hide();
        }
        let msg = if cfg!(target_os = "macos") {
            "macOS 辅助功能未授权 · 设置 → 权限诊断 → 请求授权"
        } else {
            "键盘模拟权限不足 · 已复制到剪贴板"
        };
        return Ok(InjectResult {
            ok: false,
            fallback: Some("clipboard".to_string()),
            message: Some(msg.into()),
        });
    }

    // 4) 隐藏抽屉（释放 key window）
    if let Some(w) = app.get_webview_window("drawer") {
        let _ = w.hide();
    }

    // 5) 激活之前保存的目标窗口
    let target = app
        .try_state::<LastFrontmost>()
        .map(|s| *s.0.lock())
        .unwrap_or_default();
    match (target.hwnd, target.pid) {
        (Some(hwnd), pid) => {
            tracing::info!(hwnd, ?pid, "activating saved target window");
            platform::activate_window_by_handle(hwnd, pid);
        }
        (None, Some(pid)) => {
            tracing::info!(pid, "activating saved frontmost pid");
            platform::activate_app_by_pid(pid);
        }
        (None, None) => {
            tracing::warn!("no saved target; paste may go to wrong window");
        }
    }

    // 6) 等焦点切换稳定。120ms 比之前 40ms 更稳，特别是浏览器渲染进程。
    thread::sleep(Duration::from_millis(120));

    // 7) Cmd/Ctrl+V
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };
    fn do_paste(modifier: Key) -> Result<(), String> {
        let mut e = Enigo::new(&EnigoSettings::default())
            .map_err(|err| format!("enigo init: {err}"))?;
        e.key(modifier, Direction::Press)
            .map_err(|err| format!("press: {err}"))?;
        e.key(Key::Unicode('v'), Direction::Click)
            .map_err(|err| format!("v: {err}"))?;
        e.key(modifier, Direction::Release)
            .map_err(|err| format!("release: {err}"))?;
        Ok(())
    }
    let result = do_paste(modifier);

    // 8) 不论成功失败都尝试还原剪贴板（粘贴失败时还原更应该）
    schedule_clipboard_restore(prev_clipboard, content, 600);

    match result {
        Ok(()) => {
            tracing::info!("inject ok");
            Ok(InjectResult {
                ok: true,
                fallback: None,
                message: None,
            })
        }
        Err(msg) => {
            tracing::warn!(error = %msg, "inject paste sim failed");
            Ok(InjectResult {
                ok: false,
                fallback: Some("clipboard".to_string()),
                message: Some(msg),
            })
        }
    }
}

#[tauri::command]
pub fn permissions_check_accessibility() -> bool {
    let trusted = permissions::is_trusted();
    tracing::info!(trusted, "permissions_check_accessibility");
    trusted
}

#[tauri::command]
pub fn permissions_request_accessibility() -> bool {
    let trusted = permissions::prompt_trust();
    tracing::info!(trusted, "permissions_request_accessibility");
    trusted
}
