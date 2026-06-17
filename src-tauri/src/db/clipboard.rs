// db/clipboard.rs — 剪贴板历史仓储（仅文本）。
// 后台监听线程检测到剪贴板变化时 insert；前端按分类 chip 列出。
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::{AppError, AppResult};
use super::now_ms;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ClipEntry {
    pub id: i64,
    pub content: String,
    pub char_count: i64,
    pub created_at: i64,
}

fn latest_content(conn: &Connection) -> AppResult<Option<String>> {
    match conn.query_row(
        "SELECT content FROM clipboard_history ORDER BY id DESC LIMIT 1",
        [],
        |r| r.get::<_, String>(0),
    ) {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Db(e.to_string())),
    }
}

/// 插入一条剪贴板记录。
/// - 与「最新一条」内容相同则跳过（连续重复去重），返回 false。
/// - 插入后把表裁剪到最近 `limit` 条（limit<=0 表示不裁剪）。
pub fn insert(conn: &Connection, content: &str, limit: i64) -> AppResult<bool> {
    if let Some(latest) = latest_content(conn)? {
        if latest == content {
            return Ok(false);
        }
    }
    let chars = content.chars().count() as i64;
    conn.execute(
        "INSERT INTO clipboard_history (content, char_count, created_at) VALUES (?1, ?2, ?3)",
        params![content, chars, now_ms()],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;

    if limit > 0 {
        conn.execute(
            "DELETE FROM clipboard_history WHERE id NOT IN \
             (SELECT id FROM clipboard_history ORDER BY id DESC LIMIT ?1)",
            params![limit],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
    }
    Ok(true)
}

pub fn list(conn: &Connection, limit: i64) -> AppResult<Vec<ClipEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, content, char_count, created_at \
             FROM clipboard_history ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(ClipEntry {
                id: r.get(0)?,
                content: r.get(1)?,
                char_count: r.get(2)?,
                created_at: r.get(3)?,
            })
        })
        .map_err(|e| AppError::Db(e.to_string()))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| AppError::Db(e.to_string()))?);
    }
    Ok(out)
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])
        .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}

pub fn clear(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM clipboard_history", [])
        .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}
