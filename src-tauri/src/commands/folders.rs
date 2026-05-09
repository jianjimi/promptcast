// commands/folders.rs
use tauri::State;

use crate::db::{self, DbState};
use crate::error::AppResult;
use crate::models::folder::Folder;

#[tauri::command]
pub fn folders_list(db: State<'_, DbState>) -> AppResult<Vec<Folder>> {
    let conn = db.0.lock();
    db::folders::list(&conn)
}

#[tauri::command]
pub fn folders_create(db: State<'_, DbState>, name: String) -> AppResult<Folder> {
    let conn = db.0.lock();
    db::folders::create(&conn, &name)
}

#[tauri::command]
pub fn folders_rename(db: State<'_, DbState>, id: i64, name: String) -> AppResult<()> {
    let conn = db.0.lock();
    db::folders::rename(&conn, id, &name)
}

#[tauri::command]
pub fn folders_delete(db: State<'_, DbState>, id: i64) -> AppResult<()> {
    let conn = db.0.lock();
    db::folders::delete(&conn, id)
}

#[tauri::command]
pub fn folders_reorder(
    db: State<'_, DbState>,
    ordered_ids: Vec<i64>,
) -> AppResult<()> {
    let mut conn = db.0.lock();
    db::folders::reorder(&mut conn, &ordered_ids)
}
