// commands/clipboard.rs — 剪贴板历史的 IPC（列表 / 删除 / 清空）。
// 写入由 lib.rs 的后台监听线程负责，这里只读和删。
use tauri::{AppHandle, State};

use crate::db::clipboard::ClipEntry;
use crate::db::{self, DbState};
use crate::error::AppResult;
use crate::events;

#[tauri::command]
pub fn clipboard_list(db: State<'_, DbState>, limit: Option<i64>) -> AppResult<Vec<ClipEntry>> {
    let conn = db.0.lock();
    db::clipboard::list(&conn, limit.unwrap_or(500))
}

#[tauri::command]
pub fn clipboard_delete(app: AppHandle, db: State<'_, DbState>, id: i64) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::clipboard::delete(&conn, id)?;
    }
    events::emit_clipboard_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn clipboard_clear(app: AppHandle, db: State<'_, DbState>) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::clipboard::clear(&conn)?;
    }
    events::emit_clipboard_changed(&app);
    Ok(())
}
