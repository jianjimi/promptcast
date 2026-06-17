// commands/folders.rs
use tauri::{AppHandle, State};

use crate::db::{self, DbState};
use crate::error::AppResult;
use crate::events;
use crate::models::folder::Folder;

#[tauri::command]
pub fn folders_list(db: State<'_, DbState>) -> AppResult<Vec<Folder>> {
    let conn = db.0.lock();
    db::folders::list(&conn)
}

#[tauri::command]
pub fn folders_create(app: AppHandle, db: State<'_, DbState>, name: String) -> AppResult<Folder> {
    let f = {
        let conn = db.0.lock();
        db::folders::create(&conn, &name)?
    };
    events::emit_folders_changed(&app);
    Ok(f)
}

#[tauri::command]
pub fn folders_rename(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
    name: String,
) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::folders::rename(&conn, id, &name)?;
    }
    events::emit_folders_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn folders_delete(app: AppHandle, db: State<'_, DbState>, id: i64) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::folders::delete(&conn, id)?;
    }
    events::emit_folders_changed(&app);
    events::emit_prompts_changed(&app); // 关联 prompt 的 folder_id 被置 NULL
    Ok(())
}

#[tauri::command]
pub fn folders_reorder(
    app: AppHandle,
    db: State<'_, DbState>,
    ordered_ids: Vec<i64>,
) -> AppResult<()> {
    {
        let mut conn = db.0.lock();
        db::folders::reorder(&mut conn, &ordered_ids)?;
    }
    events::emit_folders_changed(&app);
    Ok(())
}
