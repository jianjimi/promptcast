// commands/inject.rs — 剪贴板写入 + 模拟粘贴。
//
// 关键流程（Raycast 同款）：
//   1) 写剪贴板
//   2) AXIsProcessTrusted() 检查；未授权直接 fallback
//   3) hide drawer（释放 key window）
//   4) activate(LastFrontmost.pid) ← 把当时按快捷键时的 frontmost 应用拉回前台
//   5) sleep ~120ms 让系统切焦稳定
//   6) enigo 模拟 Cmd/Ctrl+V
use std::{thread, time::Duration};

use enigo::{Enigo, Key, Keyboard, Settings as EnigoSettings, Direction};
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

fn write_clipboard(text: &str) -> AppResult<()> {
    let mut cb = arboard::Clipboard::new()
        .map_err(|e| AppError::Internal(format!("clipboard: {e}")))?;
    cb.set_text(text)
        .map_err(|e| AppError::Internal(format!("clipboard set: {e}")))?;
    Ok(())
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

    // 1) 剪贴板
    if let Err(e) = write_clipboard(&content) {
        tracing::error!(error = %e, "clipboard write failed");
        return Ok(InjectResult {
            ok: false, fallback: None,
            message: Some(format!("clipboard write failed: {e}")),
        });
    }

    // 2) 权限
    let trusted = permissions::is_trusted();
    tracing::info!(trusted, "AXIsProcessTrusted");
    if !trusted {
        if let Some(w) = app.get_webview_window("drawer") {
            let _ = w.hide();
        }
        return Ok(InjectResult {
            ok: false,
            fallback: Some("clipboard".to_string()),
            message: Some(
                "macOS 辅助功能未授权 · 设置 → 权限诊断 → 请求授权".into(),
            ),
        });
    }

    // 3) 隐藏抽屉（释放 key window）
    if let Some(w) = app.get_webview_window("drawer") {
        let _ = w.hide();
    }

    // 4) 激活之前保存的 frontmost app（按下快捷键瞬间记下的目标）
    let target_pid = app
        .try_state::<LastFrontmost>()
        .and_then(|s| *s.0.lock());
    match target_pid {
        Some(pid) => {
            tracing::info!(pid, "activating saved frontmost app");
            platform::activate_app_by_pid(pid);
        }
        None => {
            tracing::warn!("no saved frontmost pid; paste may go to wrong window");
        }
    }

    // 5) 等焦点切换稳定。120ms 比之前 40ms 更稳，特别是浏览器渲染进程。
    thread::sleep(Duration::from_millis(120));

    // 6) Cmd/Ctrl+V
    let modifier = if cfg!(target_os = "macos") { Key::Meta } else { Key::Control };
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
    match do_paste(modifier) {
        Ok(()) => {
            tracing::info!("inject ok");
            Ok(InjectResult { ok: true, fallback: None, message: None })
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
