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
