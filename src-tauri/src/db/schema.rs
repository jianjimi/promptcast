// db/schema.rs — 当前 schema 版本的建表 SQL（migrations 引用）。
pub const V1: &str = r#"
CREATE TABLE IF NOT EXISTS folders (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT NOT NULL UNIQUE,
  sort_order  INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
  id     INTEGER PRIMARY KEY AUTOINCREMENT,
  name   TEXT NOT NULL UNIQUE,
  color  TEXT
);

CREATE TABLE IF NOT EXISTS prompts (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  title        TEXT NOT NULL,
  content      TEXT NOT NULL,
  folder_id    INTEGER REFERENCES folders(id) ON DELETE SET NULL,
  is_favorite  INTEGER NOT NULL DEFAULT 0,
  is_pinned    INTEGER NOT NULL DEFAULT 0,
  use_count    INTEGER NOT NULL DEFAULT 0,
  last_used_at INTEGER,
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS prompt_tags (
  prompt_id INTEGER NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
  tag_id    INTEGER NOT NULL REFERENCES tags(id)    ON DELETE CASCADE,
  PRIMARY KEY (prompt_id, tag_id)
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sites (
  id                 INTEGER PRIMARY KEY AUTOINCREMENT,
  name               TEXT NOT NULL,
  url                TEXT NOT NULL,
  favicon_blob       BLOB,
  favicon_mime       TEXT,
  favicon_fetched_at INTEGER,
  sort_order         INTEGER NOT NULL DEFAULT 0,
  created_at         INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_prompts_folder    ON prompts(folder_id);
CREATE INDEX IF NOT EXISTS idx_prompts_lastused  ON prompts(last_used_at);
CREATE INDEX IF NOT EXISTS idx_prompts_pinned    ON prompts(is_pinned, is_favorite);
"#;

// V2 — 剪贴板历史（仅文本）。后台监听 changeCount，自动入库。
pub const V2: &str = r#"
CREATE TABLE IF NOT EXISTS clipboard_history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  content     TEXT NOT NULL,
  char_count  INTEGER NOT NULL,
  created_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_clip_created ON clipboard_history(created_at DESC);
"#;

// V3 — 同步就绪：给可同步实体（folders/tags/prompts/sites）加全局 uuid、软删墓碑
// deleted_at、待推送标志 dirty，并补齐缺失的 updated_at / created_at。纯加列（安全）。
// 加 DEFAULT 的列会自动给既有行填默认值（dirty=1 ⇒ 迁移后整库排队首次 push）。
// uuid 的回填在 migrations.rs 里用 Rust 生成（execute_batch 不能生成 uuid）。
// 详见 plan：离线优先多设备同步 Phase 0。
pub const V3: &str = r#"
ALTER TABLE folders ADD COLUMN uuid       TEXT;
ALTER TABLE folders ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE folders ADD COLUMN deleted_at INTEGER;
ALTER TABLE folders ADD COLUMN dirty      INTEGER NOT NULL DEFAULT 1;

ALTER TABLE tags ADD COLUMN uuid       TEXT;
ALTER TABLE tags ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tags ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tags ADD COLUMN deleted_at INTEGER;
ALTER TABLE tags ADD COLUMN dirty      INTEGER NOT NULL DEFAULT 1;

ALTER TABLE prompts ADD COLUMN uuid       TEXT;
ALTER TABLE prompts ADD COLUMN deleted_at INTEGER;
ALTER TABLE prompts ADD COLUMN dirty      INTEGER NOT NULL DEFAULT 1;

ALTER TABLE sites ADD COLUMN uuid       TEXT;
ALTER TABLE sites ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sites ADD COLUMN deleted_at INTEGER;
ALTER TABLE sites ADD COLUMN dirty      INTEGER NOT NULL DEFAULT 1;

CREATE UNIQUE INDEX IF NOT EXISTS idx_folders_uuid ON folders(uuid) WHERE uuid IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_tags_uuid    ON tags(uuid)    WHERE uuid IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_prompts_uuid ON prompts(uuid) WHERE uuid IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_sites_uuid   ON sites(uuid)   WHERE uuid IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_folders_dirty ON folders(dirty) WHERE dirty = 1;
CREATE INDEX IF NOT EXISTS idx_tags_dirty    ON tags(dirty)    WHERE dirty = 1;
CREATE INDEX IF NOT EXISTS idx_prompts_dirty ON prompts(dirty) WHERE dirty = 1;
CREATE INDEX IF NOT EXISTS idx_sites_dirty   ON sites(dirty)   WHERE dirty = 1;

CREATE TABLE IF NOT EXISTS sync_state (
  id               INTEGER PRIMARY KEY CHECK (id = 1),
  device_id        TEXT NOT NULL,
  last_pull_cursor INTEGER NOT NULL DEFAULT 0,
  last_sync_at     INTEGER,
  user_id          TEXT,
  sync_enabled     INTEGER NOT NULL DEFAULT 0
);
"#;
