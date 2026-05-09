// models/site.rs
// 注意：favicon_blob 不在序列化结构里；列表 IPC 返回 favicon_data_uri (base64)。
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub favicon_data_uri: Option<String>,
    pub favicon_fetched_at: Option<i64>,
    pub sort_order: i64,
    pub created_at: i64,
}
