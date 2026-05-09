// commands/tags.rs
use tauri::{AppHandle, State};

use crate::db::{self, DbState};
use crate::error::AppResult;
use crate::events;
use crate::models::tag::Tag;

#[tauri::command]
pub fn tags_list(db: State<'_, DbState>) -> AppResult<Vec<Tag>> {
    let conn = db.0.lock();
    db::tags::list(&conn)
}

#[tauri::command]
pub fn tags_create(
    app: AppHandle,
    db: State<'_, DbState>,
    name: String,
    color: Option<String>,
) -> AppResult<Tag> {
    let t = {
        let conn = db.0.lock();
        db::tags::create(&conn, &name, color.as_deref())?
    };
    events::emit_tags_changed(&app);
    Ok(t)
}

#[tauri::command]
pub fn tags_rename(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
    name: String,
) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::tags::rename(&conn, id, &name)?;
    }
    events::emit_tags_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn tags_delete(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::tags::delete(&conn, id)?;
    }
    events::emit_tags_changed(&app);
    events::emit_prompts_changed(&app);
    Ok(())
}
