// commands/sites.rs — 网址 CRUD + favicon 抓取 + 用浏览器打开。
//
// favicon 策略：
//   1) 先尝试 `<host>/favicon.ico`
//   2) 失败则 GET 主页，用 scraper 找 <link rel="icon"> 取最大尺寸
//   3) 都失败则保留空白
use std::time::Duration;

use crate::events;
use scraper::{Html, Selector};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use url::Url;

use crate::db::{self, DbState};
use crate::error::{AppError, AppResult};
use crate::models::site::Site;

const MAX_FAVICON: u64 = 512 * 1024; // favicon 最大 512KB
const MAX_HTML: u64 = 1024 * 1024; // 主页 HTML 最多读 1MB

/// 规范化用户输入的网址：无 scheme 则补 https://；只放行 http/https。
fn normalize_url(raw: &str) -> AppResult<String> {
    let raw = raw.trim();
    let candidate = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };
    let parsed =
        Url::parse(&candidate).map_err(|e| AppError::InvalidInput(format!("无效网址: {e}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::InvalidInput("仅支持 http/https 链接".into()));
    }
    Ok(candidate)
}

/// 限长读取响应体，避免恶意/超大资源把内存撑爆。超 Content-Length 直接拒。
fn read_capped(resp: reqwest::blocking::Response, max: u64) -> Option<Vec<u8>> {
    use std::io::Read;
    if let Some(len) = resp.content_length() {
        if len > max {
            return None;
        }
    }
    let mut buf = Vec::new();
    resp.take(max).read_to_end(&mut buf).ok()?;
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

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
    let url = normalize_url(&url)?;
    let site = {
        let conn = db.0.lock();
        db::sites::create(&conn, &name, &url)?
    };
    tracing::info!(id = site.id, url = %site.url, "site created");
    events::emit_sites_changed(&app);

    // favicon 后台抓取，不阻塞 UI（慢/无响应的站点不会卡住「添加」）。
    let app2 = app.clone();
    let site_id = site.id;
    let site_url = site.url.clone();
    std::thread::spawn(move || {
        if let Some((bytes, mime)) = fetch_favicon(&site_url) {
            if let Some(state) = app2.try_state::<DbState>() {
                let conn = state.0.lock();
                let _ = db::sites::set_favicon(&conn, site_id, Some(&bytes), Some(&mime));
            }
            events::emit_sites_changed(&app2);
        } else {
            tracing::warn!(url = %site_url, "favicon fetch failed");
        }
    });
    Ok(site)
}

#[tauri::command]
pub fn sites_update(
    app: AppHandle,
    db: State<'_, DbState>,
    id: i64,
    name: String,
    url: String,
) -> AppResult<Site> {
    let url = normalize_url(&url)?;
    let s = {
        let conn = db.0.lock();
        db::sites::update(&conn, id, &name, &url)?
    };
    events::emit_sites_changed(&app);
    Ok(s)
}

#[tauri::command]
pub fn sites_delete(app: AppHandle, db: State<'_, DbState>, id: i64) -> AppResult<()> {
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
pub fn sites_refresh_favicon(app: AppHandle, db: State<'_, DbState>, id: i64) -> AppResult<Site> {
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

/// 给所有还没图标且未删除的网站后台抓 favicon。
/// 同步只传 url/元数据、不传图标字节（见 plan Phase 3）；拉到新站点后由各设备自取。
pub fn refetch_missing_favicons(app: AppHandle) {
    std::thread::spawn(move || {
        let Some(db) = app.try_state::<DbState>() else {
            return;
        };
        let targets: Vec<(i64, String)> = {
            let conn = db.0.lock();
            let mut stmt = match conn.prepare(
                "SELECT id, url FROM sites WHERE favicon_blob IS NULL AND deleted_at IS NULL",
            ) {
                Ok(s) => s,
                Err(_) => return,
            };
            let mapped = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)));
            match mapped {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(_) => return,
            }
        };
        let mut any = false;
        for (id, url) in targets {
            if let Some((bytes, mime)) = fetch_favicon(&url) {
                let conn = db.0.lock();
                if db::sites::set_favicon(&conn, id, Some(&bytes), Some(&mime)).is_ok() {
                    any = true;
                }
            }
        }
        if any {
            events::emit_sites_changed(&app);
        }
    });
}

#[tauri::command]
pub fn sites_open(app: AppHandle, db: State<'_, DbState>, id: i64) -> AppResult<()> {
    let url = {
        let conn = db.0.lock();
        db::sites::get(&conn, id)?.url
    };
    // 防御：只用系统浏览器打开 http/https，挡住 file:// 等被导入数据植入的危险 scheme。
    let parsed = Url::parse(&url).map_err(|e| AppError::InvalidInput(format!("无效网址: {e}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::InvalidInput(format!(
            "仅支持 http/https，拒绝打开: {}",
            parsed.scheme()
        )));
    }
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
    if bytes.starts_with(b"\x89PNG") {
        return "image/png";
    }
    if bytes.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
        return "image/x-icon";
    }
    if bytes.starts_with(b"GIF8") {
        return "image/gif";
    }
    if bytes.starts_with(b"\xFF\xD8") {
        return "image/jpeg";
    }
    if url.ends_with(".svg") {
        return "image/svg+xml";
    }
    "image/png"
}

fn fetch_favicon(site_url: &str) -> Option<(Vec<u8>, String)> {
    let parsed = Url::parse(site_url).ok()?;
    let origin = format!("{}://{}", parsed.scheme(), parsed.host_str()?);
    let cli = http_client();

    // 1) /favicon.ico
    let ico_url = format!("{origin}/favicon.ico");
    if let Ok(resp) = cli.get(&ico_url).send() {
        if resp.status().is_success() {
            if let Some(bytes) = read_capped(resp, MAX_FAVICON) {
                let mime = guess_mime(&bytes, &ico_url).to_string();
                return Some((bytes, mime));
            }
        }
    }

    // 2) 解析主页 <link rel="icon">
    let resp = cli.get(&origin).send().ok()?;
    let html_bytes = read_capped(resp, MAX_HTML)?;
    let html = String::from_utf8_lossy(&html_bytes);
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
    if !resp.status().is_success() {
        return None;
    }
    let bytes = read_capped(resp, MAX_FAVICON)?;
    let mime = guess_mime(&bytes, &abs).to_string();
    Some((bytes, mime))
}
