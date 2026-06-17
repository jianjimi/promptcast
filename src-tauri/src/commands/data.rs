// commands/data.rs — 导入 / 导出 JSON。
use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, State};

use crate::db::{self, DbState};
use crate::error::{AppError, AppResult};
use crate::events;
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

/// 把整库导出成 Snapshot（纯函数，便于测试 round-trip）。
pub fn export_snapshot(conn: &rusqlite::Connection) -> AppResult<Snapshot> {
    let folders = db::folders::list(conn)?;
    let tags = db::tags::list(conn)?;
    let prompts = db::prompts::list(conn, crate::models::prompt::SortMode::Created)?;
    let sites = db::sites::list(conn)?;
    let settings_obj = db::settings::get_all(conn)?;
    let settings = match serde_json::to_value(&settings_obj)? {
        Value::Object(m) => m,
        _ => serde_json::Map::new(),
    };
    Ok(Snapshot {
        version: 1,
        exported_at: chrono::Utc::now().timestamp_millis(),
        folders,
        tags,
        prompts,
        sites,
        settings,
    })
}

#[tauri::command]
pub fn data_export_json(db: State<'_, DbState>) -> AppResult<String> {
    let conn = db.0.lock();
    let snap = export_snapshot(&conn)?;
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

/// 解析 `data:<mime>;base64,<data>` → (blob, mime)。失败返回 None。
fn parse_data_uri(uri: Option<&str>) -> Option<(Vec<u8>, String)> {
    let uri = uri?;
    let rest = uri.strip_prefix("data:")?;
    let (meta, b64) = rest.split_once(',')?;
    let mime = meta.split(';').next().unwrap_or("image/png").to_string();
    let bytes = B64.decode(b64).ok()?;
    Some((bytes, mime))
}

/// 把 Snapshot 写入库（id 重映射 + settings + favicon 还原）。返回新插入条数。
/// 不在此 commit，便于命令层包事务、测试层直接驱动 round-trip。
fn import_snapshot(tx: &rusqlite::Transaction, snap: &Snapshot, replace: bool) -> AppResult<u32> {
    let dberr = |e: rusqlite::Error| AppError::Db(e.to_string());

    if replace {
        for t in [
            "prompt_tags",
            "prompts",
            "tags",
            "folders",
            "sites",
            "settings",
        ] {
            tx.execute(&format!("DELETE FROM {t}"), []).map_err(dberr)?;
        }
    }

    let mut inserted = 0u32;

    // folders：不保留源 id，建立 old->new 映射；同名复用现有（name UNIQUE）。
    let mut folder_map: HashMap<i64, i64> = HashMap::new();
    for f in &snap.folders {
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM folders WHERE name = ?1",
                params![f.name],
                |r| r.get(0),
            )
            .optional()
            .map_err(dberr)?;
        let new_id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO folders (name, sort_order, created_at) VALUES (?1, ?2, ?3)",
                    params![f.name, f.sort_order, f.created_at],
                )
                .map_err(dberr)?;
                inserted += 1;
                tx.last_insert_rowid()
            }
        };
        folder_map.insert(f.id, new_id);
    }

    // tags：同样按 name 去重，建立映射。
    let mut tag_map: HashMap<i64, i64> = HashMap::new();
    for t in &snap.tags {
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM tags WHERE name = ?1",
                params![t.name],
                |r| r.get(0),
            )
            .optional()
            .map_err(dberr)?;
        let new_id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO tags (name, color) VALUES (?1, ?2)",
                    params![t.name, t.color],
                )
                .map_err(dberr)?;
                inserted += 1;
                tx.last_insert_rowid()
            }
        };
        tag_map.insert(t.id, new_id);
    }

    // prompts：不保留源 id；folder_id / tag_ids 通过映射重写（解决两库 id 冲突丢数据）。
    for p in &snap.prompts {
        let new_folder = p.folder_id.and_then(|fid| folder_map.get(&fid).copied());
        tx.execute(
            "INSERT INTO prompts \
             (title, content, folder_id, is_favorite, is_pinned, use_count, \
              last_used_at, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                p.title,
                p.content,
                new_folder,
                p.is_favorite as i64,
                p.is_pinned as i64,
                p.use_count,
                p.last_used_at,
                p.created_at,
                p.updated_at,
            ],
        )
        .map_err(dberr)?;
        let new_pid = tx.last_insert_rowid();
        for old_tid in &p.tag_ids {
            if let Some(new_tid) = tag_map.get(old_tid) {
                tx.execute(
                    "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
                    params![new_pid, new_tid],
                )
                .map_err(dberr)?;
            }
        }
        inserted += 1;
    }

    // sites：不保留源 id；favicon 从 data URI 还原成 blob+mime（之前导入丢图标）。
    for s in &snap.sites {
        tx.execute(
            "INSERT INTO sites (name, url, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![s.name, s.url, s.sort_order, s.created_at],
        )
        .map_err(dberr)?;
        let new_sid = tx.last_insert_rowid();
        if let Some((blob, mime)) = parse_data_uri(s.favicon_data_uri.as_deref()) {
            tx.execute(
                "UPDATE sites SET favicon_blob = ?1, favicon_mime = ?2, favicon_fetched_at = ?3 \
                 WHERE id = ?4",
                params![blob, mime, s.favicon_fetched_at, new_sid],
            )
            .map_err(dberr)?;
        }
        inserted += 1;
    }

    // settings：还原 key/value（之前完全没导入 → hotkey/theme/sort 等全丢）。
    for (k, v) in &snap.settings {
        let val_str = serde_json::to_string(v)?;
        tx.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![k, val_str],
        )
        .map_err(dberr)?;
    }

    Ok(inserted)
}

#[tauri::command]
pub fn data_import_json(
    app: AppHandle,
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
    let tx = conn
        .transaction()
        .map_err(|e| AppError::Db(e.to_string()))?;
    let inserted = import_snapshot(&tx, &snap, args.mode == "replace")?;
    tx.commit().map_err(|e| AppError::Db(e.to_string()))?;

    // 通知所有窗口刷新导入后的数据。
    events::emit_prompts_changed(&app);
    events::emit_folders_changed(&app);
    events::emit_tags_changed(&app);
    events::emit_sites_changed(&app);
    events::emit_settings_changed(&app, "*");

    Ok(ImportResult {
        inserted,
        updated: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, memory_conn};
    use crate::models::prompt::PromptDraft;

    #[test]
    fn roundtrip_remaps_ids_and_restores_settings() {
        // 源库：建文件夹 + 标签 + 带标签的 prompt + 站点 + 改两个设置。
        let mut src = memory_conn();
        let fid = db::folders::create(&src, "Work").unwrap().id;
        let tid = db::tags::create(&src, "ai", None).unwrap().id;
        db::prompts::create(
            &mut src,
            PromptDraft {
                title: "p1".into(),
                content: "c".into(),
                folder_id: Some(fid),
                tag_ids: vec![tid],
            },
        )
        .unwrap();
        db::settings::set(&src, "theme", &serde_json::json!("dark")).unwrap();
        db::settings::set(&src, "sort_mode", &serde_json::json!("title")).unwrap();
        let snap = export_snapshot(&src).unwrap();

        // 目标库：先塞一条会与源 id 冲突的数据（旧 INSERT OR IGNORE 会丢源数据）。
        let mut dst = memory_conn();
        db::folders::create(&dst, "Existing").unwrap(); // 占用 folders.id=1
        {
            let tx = dst.transaction().unwrap();
            import_snapshot(&tx, &snap, false).unwrap();
            tx.commit().unwrap();
        }

        // 源 prompt 应被导入，且 folder/tag 关系经重映射后仍正确。
        let prompts = db::prompts::list(&dst, crate::models::prompt::SortMode::Created).unwrap();
        let imported = prompts
            .iter()
            .find(|p| p.title == "p1")
            .expect("p1 imported despite id clash");
        let folders = db::folders::list(&dst).unwrap();
        let work = folders.iter().find(|f| f.name == "Work").unwrap();
        assert_eq!(imported.folder_id, Some(work.id), "folder_id remapped");
        assert_eq!(imported.tag_ids.len(), 1, "tag relation preserved");

        // settings 应被还原（之前完全丢失）。
        let s = db::settings::get_all(&dst).unwrap();
        assert!(matches!(s.theme, crate::models::settings::ThemeMode::Dark));
        assert!(matches!(
            s.sort_mode,
            crate::models::prompt::SortMode::Title
        ));
    }

    #[test]
    fn favicon_data_uri_roundtrips_to_blob() {
        let src = memory_conn();
        let sid = db::sites::create(&src, "Cnb", "https://cnb.cool")
            .unwrap()
            .id;
        db::sites::set_favicon(&src, sid, Some(b"\x89PNGdata"), Some("image/png")).unwrap();
        let snap = export_snapshot(&src).unwrap();
        assert!(snap.sites[0].favicon_data_uri.is_some());

        let mut dst = memory_conn();
        {
            let tx = dst.transaction().unwrap();
            import_snapshot(&tx, &snap, true).unwrap();
            tx.commit().unwrap();
        }
        // 导入后 favicon 应还原为 data URI（之前导入会丢图标）。
        let sites = db::sites::list(&dst).unwrap();
        assert_eq!(sites.len(), 1);
        assert!(sites[0]
            .favicon_data_uri
            .as_deref()
            .unwrap()
            .starts_with("data:image/png;base64,"));
    }
}
