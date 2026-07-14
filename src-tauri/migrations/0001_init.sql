PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS clips (
  id TEXT PRIMARY KEY,
  content_type TEXT NOT NULL CHECK (content_type IN ('text', 'link', 'color', 'code', 'image', 'file')),
  plain_text TEXT,
  preview TEXT NOT NULL,
  source_id TEXT,
  source_name TEXT,
  created_at TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  byte_size INTEGER NOT NULL DEFAULT 0,
  metadata_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_clips_order
  ON clips(created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_clips_hash_created
  ON clips(content_hash, created_at DESC);

CREATE TABLE IF NOT EXISTS clip_flavors (
  clip_id TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
  format TEXT NOT NULL,
  inline_data BLOB,
  blob_path TEXT,
  byte_size INTEGER NOT NULL,
  PRIMARY KEY (clip_id, format),
  CHECK ((inline_data IS NULL) <> (blob_path IS NULL))
);

CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
  clip_id UNINDEXED,
  plain_text,
  preview,
  source_name,
  tokenize = 'unicode61'
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ignored_apps (
  app_id TEXT PRIMARY KEY,
  app_name TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS schema_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
INSERT OR REPLACE INTO schema_meta(key, value) VALUES ('version', '1');
