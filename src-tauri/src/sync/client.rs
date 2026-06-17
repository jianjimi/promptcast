// sync/client.rs — 同步后端的 blocking HTTP 客户端（复用 sites.rs 的 reqwest 套路）。
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::models::sync::Change;

#[derive(Debug)]
pub enum SyncError {
    Network(String),
    Unauthorized,
    Status(u16, String),
    Parse(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::Network(s) => write!(f, "network: {s}"),
            SyncError::Unauthorized => write!(f, "unauthorized"),
            SyncError::Status(c, b) => write!(f, "http {c}: {b}"),
            SyncError::Parse(s) => write!(f, "parse: {s}"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub access: String,
    pub refresh: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokensResponse {
    pub access: String,
    pub refresh: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PullResponse {
    pub changes: Vec<Change>,
    pub next_cursor: i64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushItemResult {
    pub uuid: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushResponse {
    pub results: Vec<PushItemResult>,
}

#[derive(Serialize)]
struct Credentials<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct RefreshBody<'a> {
    refresh: &'a str,
}

#[derive(Serialize)]
struct PullBody {
    since_cursor: i64,
    limit: i64,
}

#[derive(Serialize)]
struct PushBody<'a> {
    changes: &'a [Change],
}

pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
}

impl Client {
    pub fn new(base: &str) -> Self {
        let http = reqwest::blocking::Client::builder()
            .user_agent("PromptCast/0.1")
            .timeout(Duration::from_secs(20))
            .build()
            .expect("reqwest client");
        Self {
            http,
            base: base.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    fn parse<T: for<'de> Deserialize<'de>>(
        resp: reqwest::blocking::Response,
    ) -> Result<T, SyncError> {
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SyncError::Unauthorized);
        }
        if !status.is_success() {
            return Err(SyncError::Status(
                status.as_u16(),
                resp.text().unwrap_or_default(),
            ));
        }
        resp.json::<T>()
            .map_err(|e| SyncError::Parse(e.to_string()))
    }

    pub fn register(&self, email: &str, password: &str) -> Result<AuthResponse, SyncError> {
        let resp = self
            .http
            .post(self.url("/auth/register"))
            .json(&Credentials { email, password })
            .send()
            .map_err(|e| SyncError::Network(e.to_string()))?;
        Self::parse(resp)
    }

    pub fn login(&self, email: &str, password: &str) -> Result<AuthResponse, SyncError> {
        let resp = self
            .http
            .post(self.url("/auth/login"))
            .json(&Credentials { email, password })
            .send()
            .map_err(|e| SyncError::Network(e.to_string()))?;
        Self::parse(resp)
    }

    pub fn refresh(&self, refresh_token: &str) -> Result<TokensResponse, SyncError> {
        let resp = self
            .http
            .post(self.url("/auth/refresh"))
            .json(&RefreshBody {
                refresh: refresh_token,
            })
            .send()
            .map_err(|e| SyncError::Network(e.to_string()))?;
        Self::parse(resp)
    }

    pub fn logout(&self, refresh_token: &str) -> Result<(), SyncError> {
        let resp = self
            .http
            .post(self.url("/auth/logout"))
            .json(&RefreshBody {
                refresh: refresh_token,
            })
            .send()
            .map_err(|e| SyncError::Network(e.to_string()))?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(SyncError::Status(
                status.as_u16(),
                resp.text().unwrap_or_default(),
            ))
        }
    }

    pub fn pull(
        &self,
        access: &str,
        since_cursor: i64,
        limit: i64,
    ) -> Result<PullResponse, SyncError> {
        let resp = self
            .http
            .post(self.url("/sync/pull"))
            .bearer_auth(access)
            .json(&PullBody {
                since_cursor,
                limit,
            })
            .send()
            .map_err(|e| SyncError::Network(e.to_string()))?;
        Self::parse(resp)
    }

    pub fn push(&self, access: &str, changes: &[Change]) -> Result<PushResponse, SyncError> {
        let resp = self
            .http
            .post(self.url("/sync/push"))
            .bearer_auth(access)
            .json(&PushBody { changes })
            .send()
            .map_err(|e| SyncError::Network(e.to_string()))?;
        Self::parse(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn prompt_change(uuid: &str, updated_at: i64, title: &str) -> Change {
        Change {
            entity: "prompt".into(),
            uuid: uuid.into(),
            updated_at,
            deleted_at: None,
            data: json!({
                "title": title, "content": "x", "is_favorite": false, "is_pinned": false,
                "created_at": 1, "folder_uuid": null, "tag_uuids": []
            }),
            seq: 0,
        }
    }

    // 实机集成测试：验证 Rust 客户端 ↔ 真实服务端的序列化/LWW 全链路。
    // 需要本地服务端在 http://localhost:3000（或设 SYNC_TEST_URL）。默认 #[ignore]，手动跑：
    //   cargo test --manifest-path src-tauri/Cargo.toml --lib sync::client::tests -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_register_push_pull_lww() {
        let base =
            std::env::var("SYNC_TEST_URL").unwrap_or_else(|_| "http://localhost:3000".into());
        let c = Client::new(&base);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let email = format!("rust{nanos}@test.io");
        let auth = c.register(&email, "password123").expect("register");
        assert!(!auth.access.is_empty(), "got access token");

        let uuid = "22222222-2222-4222-8222-222222222222";
        let push = c
            .push(&auth.access, &[prompt_change(uuid, 1000, "v1")])
            .expect("push");
        assert!(push.results[0].applied, "first push applied");

        let pull = c.pull(&auth.access, 0, 100).expect("pull");
        assert_eq!(pull.changes.len(), 1, "pull returns the pushed change");
        assert_eq!(pull.changes[0].uuid, uuid);

        // 更新覆盖。
        let p2 = c
            .push(&auth.access, &[prompt_change(uuid, 2000, "v2")])
            .expect("push v2");
        assert!(p2.results[0].applied);

        // LWW：过期 push 被拒。
        let stale = c
            .push(&auth.access, &[prompt_change(uuid, 1500, "old")])
            .expect("push stale");
        assert!(!stale.results[0].applied, "stale push rejected by LWW");

        // 从 cursor 0 再拉，应拿到 v2。
        let pull2 = c.pull(&auth.access, 0, 100).expect("pull2");
        let rec = pull2.changes.iter().find(|c| c.uuid == uuid).unwrap();
        assert_eq!(rec.data["title"], "v2");
    }
}
