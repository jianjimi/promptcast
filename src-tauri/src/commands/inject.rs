// commands/inject.rs — 剪贴板写入 + 模拟粘贴。
//
// macOS 关键点：
//   - 必须在 系统设置 → 隐私与安全 → 辅助功能 里授权 Prompt Hub
//   - AXIsProcessTrusted() 是真实判定；Enigo::new() 成功 ≠ 已授权
//   - 抽屉是 NSPanel + nonactivatingPanel，**不会抢焦点**，
//     所以原应用的输入框 caret 始终保持，注入直接打过去即可
use std::{thread, time::Duration};

use enigo::{Enigo, Key, Keyboard, Settings as EnigoSettings, Direction};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::platform::permissions;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InjectResult {
    pub ok: bool,
    pub fallback: Option<String>, // "clipboard" 或 None
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

    // 1) 写入剪贴板
    if let Err(e) = write_clipboard(&content) {
        tracing::error!(error = %e, "clipboard write failed");
        return Ok(InjectResult {
            ok: false,
            fallback: None,
            message: Some(format!("clipboard write failed: {e}")),
        });
    }
    tracing::info!("clipboard written");

    // 2) 检查 macOS 辅助功能权限（这是注入失败的最常见原因）
    let trusted = permissions::is_trusted();
    tracing::info!(trusted, "AXIsProcessTrusted");
    if !trusted {
        tracing::warn!("accessibility NOT granted; paste will be silently dropped");
        if let Some(w) = app.get_webview_window("drawer") {
            let _ = w.hide();
        }
        return Ok(InjectResult {
            ok: false,
            fallback: Some("clipboard".to_string()),
            message: Some(
                "macOS 辅助功能未授权 · 请在 系统设置 → 隐私与安全 → 辅助功能 里勾选 Prompt Hub".into()
            ),
        });
    }

    // 3) NSPanel 不抢焦点，所以原应用的输入框 caret 还在那。
    //    短暂等待让系统切换稳一些（剪贴板 / 焦点链）。
    thread::sleep(Duration::from_millis(40));

    // 4) 模拟 Cmd/Ctrl+V
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

    // 5) 不论成败，隐藏抽屉
    if let Some(w) = app.get_webview_window("drawer") {
        let _ = w.hide();
    }

    match result {
        Ok(()) => {
            tracing::info!("inject ok");
            Ok(InjectResult { ok: true, fallback: None, message: None })
        }
        Err(msg) => {
            tracing::warn!(error = %msg, "inject failed; falling back to clipboard");
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

/// 调系统弹窗：未授权时引导用户去开权限。
#[tauri::command]
pub fn permissions_request_accessibility() -> bool {
    let trusted = permissions::prompt_trust();
    tracing::info!(trusted, "permissions_request_accessibility");
    trusted
}
