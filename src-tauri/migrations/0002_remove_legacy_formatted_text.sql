-- Development-only data cleanup: formatted_text is no longer a supported content type.
-- Do not convert it to plain text; discard the obsolete local record instead.
DELETE FROM clips WHERE content_type = 'formatted_text';
DELETE FROM clips_fts WHERE clip_id NOT IN (SELECT id FROM clips);
INSERT OR REPLACE INTO schema_meta(key, value) VALUES ('version', '2');
