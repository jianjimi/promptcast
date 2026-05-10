// commands/window.rs — drawer 显隐 + 共享 editor/preview/settings 窗口的复用。
//
// 重要变更：之前用 `WebviewWindowBuilder::build()` 运行时建窗在 Windows
// + Tauri 2.11 上会挂住（builder.build() 永不返回，子窗白屏）。改为
// 在 tauri.conf.json 预声明所有窗口（initially visible:false），运行期
// 只做 show/hide + 通过 webview.eval 切 hash 路由把 id 带入。
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct WindowInfo {
    pub label: String,
}

/// 把单例窗口拉到前面。可选地先把 hash 切换到 `target_hash`
/// (例如 "#/editor/42")，让前端路由跳过去。
fn show_singleton(
    app: &AppHandle,
    label: &str,
    target_hash: Option<&str>,
) -> AppResult<WindowInfo> {
    let win = app
        .get_webview_window(label)
        .ok_or_else(|| AppError::NotFound(format!("window '{label}' not declared in tauri.conf")))?;

    if let Some(hash) = target_hash {
        // hash 用 JSON 字符串字面量转义，避免反斜杠 / 引号注入。
        let escaped = serde_json::to_string(hash)
            .map_err(|e| AppError::Internal(format!("escape hash: {e}")))?;
        let js = format!(
            "if (location.hash !== {0}) {{ location.hash = {0}; }}",
            escaped
        );
        if let Err(e) = win.eval(&js) {
            tracing::warn!(label, error = %e, "eval hash failed");
        }
    }

    let _ = win.show();
    let _ = win.unminimize();
    let _ = win.set_focus();
    tracing::info!(label, target_hash, "window shown");
    Ok(WindowInfo {
        label: label.to_string(),
    })
}

#[tauri::command]
pub fn window_show_drawer(app: AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("drawer") {
        let _ = w.show();
        let _ = w.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub fn window_hide_drawer(app: AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("drawer") {
        let _ = w.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn window_set_pin(app: AppHandle, pinned: bool) -> AppResult<()> {
    if let Some(w) = app.get_webview_window("drawer") {
        let _ = w.set_always_on_top(pinned);
    }
    Ok(())
}

#[tauri::command]
pub fn window_open_preview(app: AppHandle, id: i64) -> AppResult<WindowInfo> {
    let hash = format!("#/preview/{id}");
    show_singleton(&app, "preview", Some(&hash))
}

#[tauri::command]
pub fn window_open_editor(app: AppHandle, id: Option<i64>) -> AppResult<WindowInfo> {
    tracing::info!(?id, "window_open_editor invoked");
    let hash = match id {
        Some(i) => format!("#/editor/{i}"),
        None => "#/editor".to_string(),
    };
    show_singleton(&app, "editor", Some(&hash))
}

#[tauri::command]
pub fn window_open_settings(app: AppHandle) -> AppResult<WindowInfo> {
    show_singleton(&app, "settings", Some("#/settings"))
}
