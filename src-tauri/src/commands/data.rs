// commands/data.rs — 导入 / 导出 JSON。
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::db::{self, DbState};
use crate::error::{AppError, AppResult};
use crate::models::{folder::Folder, prompt::Prompt, site::Site, tag::Tag};

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    pub exported_at: i64,
    pub folders: Vec<Folder>,
    pub tags: Vec<Tag>,
    pub prompts: Vec<Prompt>,
    #[serde(default)]
    pub sites: Vec<Site>,
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
    let sites = db::sites::list(&conn)?;
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
        sites,
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
        // 顺序：依赖项先删；sites 与 prompts 无外键关系但也应清。
        for t in ["prompt_tags", "prompts", "tags", "folders", "sites"] {
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
    for s in &snap.sites {
        // 注意：favicon_data_uri 需还原为 blob+mime 才能写库；MVP 先不导入二进制，
        // 用户在导入后可在设置里点 ↻ 重新抓取。
        tx.execute(
            "INSERT OR IGNORE INTO sites (id, name, url, sort_order, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![s.id, s.name, s.url, s.sort_order, s.created_at],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
        inserted += 1;
    }

    tx.commit().map_err(|e| AppError::Db(e.to_string()))?;
    Ok(ImportResult { inserted, updated: 0 })
}
