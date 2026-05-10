// commands/window.rs — 抽屉显隐 + 按需创建预览/编辑/设置窗口。
use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct WindowInfo {
    pub label: String,
}

fn ensure_window(
    app: &AppHandle,
    label: &str,
    url: &str,
    width: f64,
    height: f64,
    title: &str,
) -> AppResult<WindowInfo> {
    // 打开任何子窗口时先隐藏抽屉，避免它（floating level）盖住子窗口。
    if let Some(drawer) = app.get_webview_window("drawer") {
        let _ = drawer.hide();
    }
    if let Some(existing) = app.get_webview_window(label) {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(WindowInfo {
            label: label.to_string(),
        });
    }
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(width, height)
        .resizable(true)
        .center()
        .build()
        .map_err(|e| AppError::Internal(format!("create window {label}: {e}")))?;
    tracing::info!(label, "sub-window created");
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
    ensure_window(
        &app,
        &format!("preview-{id}"),
        &format!("index.html#/preview/{id}"),
        540.0,
        640.0,
        "预览 · Prompt Hub",
    )
}

#[tauri::command]
pub fn window_open_editor(app: AppHandle, id: Option<i64>) -> AppResult<WindowInfo> {
    tracing::info!(?id, "window_open_editor invoked");
    let (label, route) = match id {
        Some(i) => (format!("editor-{i}"), format!("index.html#/editor/{i}")),
        None => ("editor-new".to_string(), "index.html#/editor".to_string()),
    };
    ensure_window(&app, &label, &route, 600.0, 560.0, "编辑 · Prompt Hub")
}

#[tauri::command]
pub fn window_open_settings(app: AppHandle) -> AppResult<WindowInfo> {
    ensure_window(
        &app,
        "settings",
        "index.html#/settings",
        680.0,
        560.0,
        "设置 · Prompt Hub",
    )
}
