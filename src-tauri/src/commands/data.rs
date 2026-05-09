// commands/data.rs — 导入 / 导出 JSON。
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::db::{self, DbState};
use crate::error::{AppError, AppResult};
use crate::models::{folder::Folder, prompt::Prompt, tag::Tag};

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    pub exported_at: i64,
    pub folders: Vec<Folder>,
    pub tags: Vec<Tag>,
    pub prompts: Vec<Prompt>,
    pub settings: serde_json::Map<String, Value>,
}

#[tauri::command]
pub fn data_export_json(db: State<'_, DbState>) -> AppResult<String> {
    let conn = db.0.lock();
    let folders = db::folders::list(&conn)?;
    let tags = db::tags::list(&conn)?;
    let prompts = db::prompts::list(
        &conn,
        crate::models::prompt::SortMode::Created,
    )?;
    let settings_obj = db::settings::get_all(&conn)?;
    let settings_value = serde_json::to_value(&settings_obj)?;
    let settings = match settings_value {
        Value::Object(m) => m,
        _ => serde_json::Map::new(),
    };
    let snap = Snapshot {
        version: 1,
        exported_at: chrono::Utc::now().timestamp_millis(),
        folders,
        tags,
        prompts,
        settings,
    };
    Ok(serde_json::to_string_pretty(&snap)?)
}

#[derive(Debug, Deserialize)]
pub struct ImportArgs {
    pub json: String,
    pub mode: String, // "merge" | "replace"
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub inserted: u32,
    pub updated: u32,
}

#[tauri::command]
pub fn data_import_json(
    db: State<'_, DbState>,
    args: ImportArgs,
) -> AppResult<ImportResult> {
    let snap: Snapshot = serde_json::from_str(&args.json)?;
    if snap.version != 1 {
        return Err(AppError::InvalidInput(format!(
            "unsupported snapshot version {}",
            snap.version
        )));
    }
    let mut conn = db.0.lock();
    let tx = conn.transaction().map_err(|e| AppError::Db(e.to_string()))?;

    if args.mode == "replace" {
        for t in ["prompt_tags", "prompts", "tags", "folders"] {
            tx.execute(&format!("DELETE FROM {t}"), [])
                .map_err(|e| AppError::Db(e.to_string()))?;
        }
    }

    let mut inserted = 0u32;
    for f in &snap.folders {
        tx.execute(
            "INSERT OR IGNORE INTO folders (id, name, sort_order, created_at) \
             VALUES (?1, ?2, ?3, ?4)",
            params![f.id, f.name, f.sort_order, f.created_at],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
        inserted += 1;
    }
    for t in &snap.tags {
        tx.execute(
            "INSERT OR IGNORE INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
            params![t.id, t.name, t.color],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
        inserted += 1;
    }
    for p in &snap.prompts {
        tx.execute(
            "INSERT OR IGNORE INTO prompts \
             (id, title, content, folder_id, is_favorite, is_pinned, use_count, \
              last_used_at, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                p.id, p.title, p.content, p.folder_id,
                p.is_favorite as i64, p.is_pinned as i64,
                p.use_count, p.last_used_at, p.created_at, p.updated_at,
            ],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
        for tid in &p.tag_ids {
            tx.execute(
                "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
                params![p.id, tid],
            )
            .map_err(|e| AppError::Db(e.to_string()))?;
        }
        inserted += 1;
    }

    tx.commit().map_err(|e| AppError::Db(e.to_string()))?;
    Ok(ImportResult { inserted, updated: 0 })
}
