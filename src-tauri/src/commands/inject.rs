// commands/inject.rs — 剪贴板写入 + 模拟粘贴。
//
// 流程：写剪贴板 → 隐藏抽屉 → 等 ~80ms 让焦点回到原窗口 → enigo 模拟 ⌘V/Ctrl+V。
// 任一步失败回退仅复制，前端 toast 提示。
use std::{thread, time::Duration};

use enigo::{Enigo, Key, Keyboard, Settings as EnigoSettings, Direction};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

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
    write_clipboard(&content)
}

#[tauri::command]
pub fn inject_paste(app: AppHandle, content: String) -> AppResult<InjectResult> {
    // 1) 写入剪贴板
    if let Err(e) = write_clipboard(&content) {
        tracing::error!(error = %e, "clipboard write failed");
        return Ok(InjectResult {
            ok: false,
            fallback: None,
            message: Some(format!("clipboard write failed: {e}")),
        });
    }
    tracing::info!(len = content.len(), "clipboard written; injecting");

    // 2) 因为 drawer 是 NSPanel/NoActivate 风格，不会抢焦点
    //    直接模拟粘贴；轻微等待让系统切换排序稳一些。
    thread::sleep(Duration::from_millis(40));

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
    // 不论结果，隐藏 drawer 让用户回到目标应用。
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
    // macOS：尝试创建 Enigo，失败大概率是辅助功能未授权。
    // Windows：基本不需要权限，恒返回 true。
    #[cfg(target_os = "macos")]
    {
        Enigo::new(&EnigoSettings::default()).is_ok()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}
