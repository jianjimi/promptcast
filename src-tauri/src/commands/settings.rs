// commands/settings.rs
use serde_json::Value;
use tauri::{AppHandle, State};

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
    Ok(())
}
