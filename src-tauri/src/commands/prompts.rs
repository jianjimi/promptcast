// commands/prompts.rs
use tauri::{AppHandle, State};

use crate::db::{self, DbState};
use crate::error::AppResult;
use crate::events;
use crate::models::prompt::{Prompt, PromptDraft, SortMode};

#[tauri::command]
pub fn prompts_list(
    db: State<'_, DbState>,
    sort: SortMode,
) -> AppResult<Vec<Prompt>> {
    let conn = db.0.lock();
    db::prompts::list(&conn, sort)
}

#[tauri::command]
pub fn prompts_get(db: State<'_, DbState>, id: i64) -> AppResult<Prompt> {
    let conn = db.0.lock();
    db::prompts::get(&conn, id)
}

#[tauri::command]
pub fn prompts_create(
    app: AppHandle,
    db: State<'_, DbState>,
    draft: PromptDraft,
) -> AppResult<Prompt> {
    let p = {
        let mut conn = db.0.lock();
        db::prompts::create(&mut conn, draft)?
    };
    tracing::info!(id = p.id, title = %p.title, "prompt created");
    events::emit_prompts_changed(&app);
    Ok(p)
}

#[tauri::command]
pub fn prompts_update(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
    draft: PromptDraft,
) -> AppResult<Prompt> {
    let p = {
        let mut conn = db.0.lock();
        db::prompts::update(&mut conn, id, draft)?
    };
    tracing::info!(id = p.id, "prompt updated");
    events::emit_prompts_changed(&app);
    Ok(p)
}

#[tauri::command]
pub fn prompts_delete(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::prompts::delete(&conn, id)?;
    }
    tracing::info!(id, "prompt deleted");
    events::emit_prompts_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn prompts_toggle_favorite(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<Prompt> {
    let p = {
        let conn = db.0.lock();
        db::prompts::toggle_favorite(&conn, id)?
    };
    events::emit_prompts_changed(&app);
    Ok(p)
}

#[tauri::command]
pub fn prompts_toggle_pin(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<Prompt> {
    let p = {
        let conn = db.0.lock();
        db::prompts::toggle_pin(&conn, id)?
    };
    events::emit_prompts_changed(&app);
    Ok(p)
}

#[tauri::command]
pub fn prompts_record_use(db: State<'_, DbState>, id: i64) -> AppResult<()> {
    let conn = db.0.lock();
    db::prompts::record_use(&conn, id)
}
