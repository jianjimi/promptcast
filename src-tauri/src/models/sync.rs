// models/sync.rs — 同步变更信封（与后端 sync_records 的 envelope 对齐）。
// data 是不透明 JSON：folder/tag/site/prompt 各自的可同步字段；prompt 的 data 内嵌
// folder_uuid + tag_uuids[]（跨设备引用用 uuid，apply 时解析成本地 id）。
use serde::{Deserialize, Serialize};

/// 一条同步变更。push 时 seq 不发（服务端分配）；pull 时 seq 是服务端游标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub entity: String, // "folder" | "tag" | "prompt" | "site"
    pub uuid: String,
    pub updated_at: i64,
    #[serde(default)]
    pub deleted_at: Option<i64>,
    pub data: serde_json::Value,
    #[serde(default)]
    pub seq: i64,
}

pub const ENTITY_FOLDER: &str = "folder";
pub const ENTITY_TAG: &str = "tag";
pub const ENTITY_PROMPT: &str = "prompt";
pub const ENTITY_SITE: &str = "site";

/// prompt 的 data 负载（内嵌跨设备引用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptData {
    pub title: String,
    pub content: String,
    pub is_favorite: bool,
    pub is_pinned: bool,
    pub created_at: i64,
    pub folder_uuid: Option<String>,
    pub tag_uuids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderData {
    pub name: String,
    pub sort_order: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagData {
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteData {
    pub name: String,
    pub url: String,
    pub sort_order: i64,
    pub created_at: i64,
}
