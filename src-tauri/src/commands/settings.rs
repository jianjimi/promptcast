// commands/settings.rs
use serde_json::Value;
use tauri::State;

use crate::db::{self, DbState};
use crate::error::AppResult;
use crate::models::settings::Settings;

#[tauri::command]
pub fn settings_get_all(db: State<'_, DbState>) -> AppResult<Settings> {
    let conn = db.0.lock();
    db::settings::get_all(&conn)
}

#[tauri::command]
pub fn settings_set(
    db: State<'_, DbState>,
    key: String,
    value: Value,
) -> AppResult<()> {
    let conn = db.0.lock();
    db::settings::set(&conn, &key, &value)
}
