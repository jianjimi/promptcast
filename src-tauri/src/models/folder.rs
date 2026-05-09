// models/folder.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
    pub created_at: i64,
}
