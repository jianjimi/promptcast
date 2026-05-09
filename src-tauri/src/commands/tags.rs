// commands/tags.rs
use tauri::State;

use crate::db::{self, DbState};
use crate::error::AppResult;
use crate::models::tag::Tag;

#[tauri::command]
pub fn tags_list(db: State<'_, DbState>) -> AppResult<Vec<Tag>> {
    let conn = db.0.lock();
    db::tags::list(&conn)
}

#[tauri::command]
pub fn tags_create(
    db: State<'_, DbState>,
    name: String,
    color: Option<String>,
) -> AppResult<Tag> {
    let conn = db.0.lock();
    db::tags::create(&conn, &name, color.as_deref())
}

#[tauri::command]
pub fn tags_rename(db: State<'_, DbState>, id: i64, name: String) -> AppResult<()> {
    let conn = db.0.lock();
    db::tags::rename(&conn, id, &name)
}

#[tauri::command]
pub fn tags_delete(db: State<'_, DbState>, id: i64) -> AppResult<()> {
    let conn = db.0.lock();
    db::tags::delete(&conn, id)
}
