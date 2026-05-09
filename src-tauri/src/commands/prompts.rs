// commands/prompts.rs
use tauri::State;

use crate::db::{self, DbState};
use crate::error::AppResult;
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
    db: State<'_, DbState>,
    draft: PromptDraft,
) -> AppResult<Prompt> {
    let mut conn = db.0.lock();
    db::prompts::create(&mut conn, draft)
}

#[tauri::command]
pub fn prompts_update(
    db: State<'_, DbState>,
    id: i64,
    draft: PromptDraft,
) -> AppResult<Prompt> {
    let mut conn = db.0.lock();
    db::prompts::update(&mut conn, id, draft)
}

#[tauri::command]
pub fn prompts_delete(db: State<'_, DbState>, id: i64) -> AppResult<()> {
    let conn = db.0.lock();
    db::prompts::delete(&conn, id)
}

#[tauri::command]
pub fn prompts_toggle_favorite(
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<Prompt> {
    let conn = db.0.lock();
    db::prompts::toggle_favorite(&conn, id)
}

#[tauri::command]
pub fn prompts_toggle_pin(
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<Prompt> {
    let conn = db.0.lock();
    db::prompts::toggle_pin(&conn, id)
}

#[tauri::command]
pub fn prompts_record_use(db: State<'_, DbState>, id: i64) -> AppResult<()> {
    let conn = db.0.lock();
    db::prompts::record_use(&conn, id)
}
