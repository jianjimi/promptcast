// commands/settings.rs
use serde_json::Value;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::db::{self, DbState};
use crate::error::AppResult;
use crate::events;
use crate::models::settings::Settings;

#[tauri::command]
pub fn settings_get_all(db: State<'_, DbState>) -> AppResult<Settings> {
    let conn = db.0.lock();
    db::settings::get_all(&conn)
}

#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    db: State<'_, DbState>,
    key: String,
    value: Value,
) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::settings::set(&conn, &key, &value)?;
    }
    tracing::info!(%key, value = %value, "settings updated");
    events::emit_settings_changed(&app, &key);
    if key == "theme" {
        if let Some(s) = value.as_str() {
            events::emit_theme_changed(&app, s);
        }
    }
    if key == "auto_start" {
        if let Some(enabled) = value.as_bool() {
            apply_autostart(&app, enabled);
        }
    }
    Ok(())
}

/// 把 auto_start 状态同步到系统（macOS LaunchAgent / Windows 注册表）。
/// 仅在状态确实需要变更时才调用，避免 disable 一个不存在的项导致 ENOENT。
pub fn apply_autostart(app: &AppHandle, enabled: bool) {
    let mgr = app.autolaunch();
    let current = match mgr.is_enabled() {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "autostart is_enabled failed; skipping sync");
            return;
        }
    };
    if current == enabled {
        return;
    }
    let res = if enabled { mgr.enable() } else { mgr.disable() };
    match res {
        Ok(()) => tracing::info!(enabled, "autostart applied"),
        Err(e) => tracing::error!(enabled, error = %e, "autostart apply failed"),
    }
}
