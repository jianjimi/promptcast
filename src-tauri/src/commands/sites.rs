// commands/sites.rs — 网址 CRUD + favicon 抓取 + 用浏览器打开。
//
// favicon 策略：
//   1) 先尝试 `<host>/favicon.ico`
//   2) 失败则 GET 主页，用 scraper 找 <link rel="icon"> 取最大尺寸
//   3) 都失败则保留空白
use std::time::Duration;

use scraper::{Html, Selector};
use tauri::{AppHandle, State};
use crate::events;
use tauri_plugin_opener::OpenerExt;
use url::Url;

use crate::db::{self, DbState};
use crate::error::{AppError, AppResult};
use crate::models::site::Site;

#[tauri::command]
pub fn sites_list(db: State<'_, DbState>) -> AppResult<Vec<Site>> {
    let conn = db.0.lock();
    db::sites::list(&conn)
}

#[tauri::command]
pub fn sites_create(
    app: AppHandle,
    db: State<'_, DbState>,
    name: String,
    url: String,
) -> AppResult<Site> {
    let site = {
        let conn = db.0.lock();
        db::sites::create(&conn, &name, &url)?
    };
    tracing::info!(id = site.id, url = %site.url, "site created");
    // 同步抓一次 favicon。失败不影响 site 已创建。
    let final_site = if let Some((bytes, mime)) = fetch_favicon(&site.url) {
        let conn = db.0.lock();
        let _ = db::sites::set_favicon(&conn, site.id, Some(&bytes), Some(&mime));
        db::sites::get(&conn, site.id)?
    } else {
        tracing::warn!(url = %site.url, "favicon fetch failed");
        site
    };
    events::emit_sites_changed(&app);
    Ok(final_site)
}

#[tauri::command]
pub fn sites_update(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
    name: String,
    url: String,
) -> AppResult<Site> {
    let s = {
        let conn = db.0.lock();
        db::sites::update(&conn, id, &name, &url)?
    };
    events::emit_sites_changed(&app);
    Ok(s)
}

#[tauri::command]
pub fn sites_delete(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<()> {
    {
        let conn = db.0.lock();
        db::sites::delete(&conn, id)?;
    }
    events::emit_sites_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn sites_reorder(
    app: AppHandle,
    db: State<'_, DbState>,
    ordered_ids: Vec<i64>,
) -> AppResult<()> {
    {
        let mut conn = db.0.lock();
        db::sites::reorder(&mut conn, &ordered_ids)?;
    }
    events::emit_sites_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn sites_refresh_favicon(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<Site> {
    let url = {
        let conn = db.0.lock();
        db::sites::get(&conn, id)?.url
    };
    let res = fetch_favicon(&url);
    let s = {
        let conn = db.0.lock();
        if let Some((bytes, mime)) = res {
            db::sites::set_favicon(&conn, id, Some(&bytes), Some(&mime))?;
        } else {
            db::sites::set_favicon(&conn, id, None, None)?;
        }
        db::sites::get(&conn, id)?
    };
    events::emit_sites_changed(&app);
    Ok(s)
}

#[tauri::command]
pub fn sites_open(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
) -> AppResult<()> {
    let url = {
        let conn = db.0.lock();
        db::sites::get(&conn, id)?.url
    };
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Internal(e.to_string()))
}

// ---- favicon fetcher ----

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 PromptHub/0.1")
        .timeout(Duration::from_secs(8))
        .build()
        .expect("reqwest client")
}

fn guess_mime(bytes: &[u8], url: &str) -> &'static str {
    if bytes.starts_with(b"\x89PNG") { return "image/png"; }
    if bytes.starts_with(&[0x00, 0x00, 0x01, 0x00]) { return "image/x-icon"; }
    if bytes.starts_with(b"GIF8") { return "image/gif"; }
    if bytes.starts_with(b"\xFF\xD8") { return "image/jpeg"; }
    if url.ends_with(".svg") { return "image/svg+xml"; }
    "image/png"
}

fn fetch_favicon(site_url: &str) -> Option<(Vec<u8>, String)> {
    let parsed = Url::parse(site_url).ok()?;
    let origin = format!(
        "{}://{}",
        parsed.scheme(),
        parsed.host_str()?
    );
    let cli = http_client();

    // 1) /favicon.ico
    let ico_url = format!("{origin}/favicon.ico");
    if let Ok(resp) = cli.get(&ico_url).send() {
        if resp.status().is_success() {
            if let Ok(bytes) = resp.bytes() {
                if !bytes.is_empty() {
                    let mime = guess_mime(&bytes, &ico_url).to_string();
                    return Some((bytes.to_vec(), mime));
                }
            }
        }
    }

    // 2) 解析主页 <link rel="icon">
    let resp = cli.get(&origin).send().ok()?;
    let html = resp.text().ok()?;
    let doc = Html::parse_document(&html);
    let sel = Selector::parse(r#"link[rel*="icon"]"#).ok()?;
    let mut href: Option<String> = None;
    for el in doc.select(&sel) {
        if let Some(h) = el.value().attr("href") {
            href = Some(h.to_string());
            break;
        }
    }
    let href = href?;
    let abs = parsed.join(&href).ok()?.to_string();
    let resp = cli.get(&abs).send().ok()?;
    if !resp.status().is_success() { return None; }
    let bytes = resp.bytes().ok()?;
    if bytes.is_empty() { return None; }
    let mime = guess_mime(&bytes, &abs).to_string();
    Some((bytes.to_vec(), mime))
}
