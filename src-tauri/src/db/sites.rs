// db/sites.rs — 网址快捷。favicon 抓取在 commands/sites.rs 里通过 reqwest。
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rusqlite::{params, Connection};

use super::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::site::Site;

fn map_row_basic(
    r: &rusqlite::Row<'_>,
) -> rusqlite::Result<(
    i64,
    String,
    String,
    Option<Vec<u8>>,
    Option<String>,
    Option<i64>,
    i64,
    i64,
)> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get::<_, Option<Vec<u8>>>(3)?,
        r.get::<_, Option<String>>(4)?,
        r.get::<_, Option<i64>>(5)?,
        r.get(6)?,
        r.get(7)?,
    ))
}

fn to_data_uri(blob: Option<Vec<u8>>, mime: Option<String>) -> Option<String> {
    let bytes = blob?;
    let mime = mime.unwrap_or_else(|| "image/png".to_string());
    Some(format!("data:{mime};base64,{}", B64.encode(bytes)))
}

pub fn list(conn: &Connection) -> AppResult<Vec<Site>> {
    let sql = "SELECT id, name, url, favicon_blob, favicon_mime, favicon_fetched_at, \
               sort_order, created_at FROM sites ORDER BY sort_order ASC, id ASC";
    let mut stmt = conn.prepare(sql).map_err(|e| AppError::Db(e.to_string()))?;
    let rows = stmt
        .query_map([], map_row_basic)
        .map_err(|e| AppError::Db(e.to_string()))?;
    let mut out = Vec::new();
    for r in rows {
        let (id, name, url, blob, mime, fetched, so, ca) =
            r.map_err(|e| AppError::Db(e.to_string()))?;
        out.push(Site {
            id,
            name,
            url,
            favicon_data_uri: to_data_uri(blob, mime),
            favicon_fetched_at: fetched,
            sort_order: so,
            created_at: ca,
        });
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Site> {
    let sql = "SELECT id, name, url, favicon_blob, favicon_mime, favicon_fetched_at, \
               sort_order, created_at FROM sites WHERE id = ?1";
    let (id, name, url, blob, mime, fetched, so, ca) = conn
        .query_row(sql, params![id], map_row_basic)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("site {id}")),
            other => AppError::Db(other.to_string()),
        })?;
    Ok(Site {
        id,
        name,
        url,
        favicon_data_uri: to_data_uri(blob, mime),
        favicon_fetched_at: fetched,
        sort_order: so,
        created_at: ca,
    })
}

pub fn create(conn: &Connection, name: &str, url: &str) -> AppResult<Site> {
    let name = name.trim();
    let url = url.trim();
    if name.is_empty() || url.is_empty() {
        return Err(AppError::InvalidInput("name/url empty".into()));
    }
    let max_order: i64 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM sites", [], |r| {
            r.get(0)
        })
        .map_err(|e| AppError::Db(e.to_string()))?;
    let now = now_ms();
    conn.execute(
        "INSERT INTO sites (name, url, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, url, max_order + 1, now],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, name: &str, url: &str) -> AppResult<Site> {
    let n = conn
        .execute(
            "UPDATE sites SET name = ?1, url = ?2 WHERE id = ?3",
            params![name.trim(), url.trim(), id],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("site {id}")));
    }
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    let n = conn
        .execute("DELETE FROM sites WHERE id = ?1", params![id])
        .map_err(|e| AppError::Db(e.to_string()))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("site {id}")));
    }
    Ok(())
}

pub fn reorder(conn: &mut Connection, ordered_ids: &[i64]) -> AppResult<()> {
    let tx = conn
        .transaction()
        .map_err(|e| AppError::Db(e.to_string()))?;
    for (i, id) in ordered_ids.iter().enumerate() {
        tx.execute(
            "UPDATE sites SET sort_order = ?1 WHERE id = ?2",
            params![i as i64, id],
        )
        .map_err(|e| AppError::Db(e.to_string()))?;
    }
    tx.commit().map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}

pub fn set_favicon(
    conn: &Connection,
    id: i64,
    blob: Option<&[u8]>,
    mime: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "UPDATE sites SET favicon_blob = ?1, favicon_mime = ?2, favicon_fetched_at = ?3 \
         WHERE id = ?4",
        params![blob, mime, now_ms(), id],
    )
    .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}
