use std::path::Path;
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::history::{
    ClipDetail, ClipSummary, ContentType, Flavor, FlavorInfo, HistoryPage, HistoryQuery, NewClip,
    SourceApp,
};

const SCHEMA: &str = include_str!("../../schema.sql");
// Development schema revisions are not migrated. Any mismatch requires a reset.
const SCHEMA_VERSION: u32 = 5;

pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| AppError::Storage(error.to_string()))?;
        }
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    pub fn in_memory() -> AppResult<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(connection: Connection) -> AppResult<Self> {
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        let version: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        match version {
            0 => {
                connection.execute_batch(SCHEMA)?;
                connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            }
            4 => migrate_v4_to_v5(&connection)?,
            SCHEMA_VERSION => {}
            unsupported => {
                return Err(AppError::Storage(format!(
                    "unsupported development database schema {unsupported}; delete the database and restart"
                )));
            }
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn connection(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| AppError::Storage("database lock was poisoned".into()))
    }

    pub fn insert_clip(&self, clip: &NewClip) -> AppResult<String> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let id = Uuid::now_v7().to_string();
        let byte_size: usize = clip.flavors.iter().map(|flavor| flavor.payload.len()).sum();
        let source_id = clip.source_app.as_ref().map(|source| source.id.as_str());
        let source_name = clip.source_app.as_ref().map(|source| source.name.as_str());
        transaction.execute(
            "INSERT INTO clips (id, content_type, plain_text, preview, source_id, source_name, created_at, last_used_at, content_hash, byte_size, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8, ?9, ?10)",
            params![
                id,
                clip.content_type.to_string(),
                clip.plain_text,
                clip.preview,
                source_id,
                source_name,
                timestamp(clip.created_at),
                clip.content_hash,
                byte_size as i64,
                serde_json::to_string(&clip.metadata)?,
            ],
        )?;
        for flavor in &clip.flavors {
            transaction.execute(
                "INSERT INTO clip_flavors (clip_id, format, inline_data, byte_size) VALUES (?1, ?2, ?3, ?4)",
                params![id, flavor.format, flavor.payload, flavor.payload.len() as i64],
            )?;
        }
        transaction.execute(
            "INSERT INTO clips_fts (clip_id, plain_text, preview, source_name) VALUES (?1, ?2, ?3, ?4)",
            params![id, clip.plain_text, clip.preview, source_name],
        )?;
        transaction.commit()?;
        Ok(id)
    }

    pub fn exists_recent_hash(&self, hash: &str, since: DateTime<Utc>) -> AppResult<bool> {
        Ok(self
            .connection()?
            .query_row(
                "SELECT 1 FROM clips WHERE content_hash = ?1 AND created_at >= ?2 LIMIT 1",
                params![hash, timestamp(since)],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    pub fn query_history(&self, request: &HistoryQuery) -> AppResult<HistoryPage> {
        if request.page == 0 || request.page > 1_000_000 || !(1..=100).contains(&request.page_size)
        {
            return Err(AppError::Validation(
                "page must be positive and page_size must be between 1 and 100".into(),
            ));
        }
        if request.query.chars().count() > 256 {
            return Err(AppError::Validation(
                "search query exceeds 256 characters".into(),
            ));
        }

        let mut conditions = Vec::new();
        let mut values = Vec::<Value>::new();
        let query = request.query.trim();
        if !query.is_empty() {
            conditions.push("c.id IN (SELECT clip_id FROM clips_fts WHERE clips_fts MATCH ?)");
            values.push(Value::Text(fts_query(query)));
        }
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };
        let connection = self.connection()?;
        let total: u64 = connection.query_row(
            &format!("SELECT COUNT(*) FROM clips c{where_clause}"),
            params_from_iter(values.clone()),
            |row| row.get(0),
        )?;
        let offset = (request.page - 1)
            .checked_mul(request.page_size)
            .ok_or_else(|| AppError::Validation("page offset is too large".into()))?;
        let mut page_values = values;
        page_values.push(Value::Integer(request.page_size.into()));
        page_values.push(Value::Integer(offset.into()));
        let sql = format!(
            "SELECT c.id, c.content_type, c.preview, c.source_id, c.source_name, c.created_at, c.byte_size, c.metadata_json
             FROM clips c{where_clause}
             ORDER BY c.last_used_at DESC, c.id DESC LIMIT ? OFFSET ?"
        );
        let mut statement = connection.prepare(&sql)?;
        let items = statement
            .query_map(params_from_iter(page_values), summary_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(HistoryPage {
            items,
            page: request.page,
            page_size: request.page_size,
            total,
            total_pages: total.div_ceil(request.page_size as u64) as u32,
        })
    }

    pub fn get_clip(&self, id: &str) -> AppResult<ClipDetail> {
        let connection = self.connection()?;
        let (summary, plain_text) = connection
            .query_row(
                "SELECT id, content_type, preview, source_id, source_name, created_at, byte_size, metadata_json, plain_text
                 FROM clips WHERE id = ?1",
                [id],
                |row| Ok((summary_from_row(row)?, row.get(8)?)),
            )
            .optional()?
            .ok_or(AppError::NotFound)?;
        let mut statement = connection.prepare(
            "SELECT format, byte_size FROM clip_flavors WHERE clip_id = ?1 ORDER BY format",
        )?;
        let flavors = statement
            .query_map([id], |row| {
                Ok(FlavorInfo {
                    format: row.get(0)?,
                    byte_size: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ClipDetail {
            summary,
            plain_text,
            flavors,
        })
    }

    pub fn get_flavors(&self, id: &str) -> AppResult<Vec<Flavor>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT format, inline_data FROM clip_flavors WHERE clip_id = ?1 ORDER BY format",
        )?;
        let flavors = statement
            .query_map([id], |row| {
                Ok(Flavor {
                    format: row.get(0)?,
                    payload: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(flavors)
    }

    pub fn touch_clip(&self, id: &str) -> AppResult<bool> {
        Ok(self.connection()?.execute(
            "UPDATE clips SET last_used_at = ?1 WHERE id = ?2",
            params![timestamp(Utc::now()), id],
        )? > 0)
    }

    pub fn cleanup_candidate_ids(
        &self,
        cutoff: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> AppResult<Vec<String>> {
        let connection = self.connection()?;
        let mut ids = std::collections::BTreeSet::new();
        if let Some(cutoff) = cutoff {
            let mut statement =
                connection.prepare("SELECT id FROM clips WHERE last_used_at < ?1")?;
            for id in statement.query_map([timestamp(cutoff)], |row| row.get::<_, String>(0))? {
                ids.insert(id?);
            }
        }
        if let Some(limit) = limit {
            let mut statement = connection.prepare(
                "SELECT id FROM clips ORDER BY last_used_at DESC, id DESC LIMIT -1 OFFSET ?1",
            )?;
            for id in statement.query_map([limit], |row| row.get::<_, String>(0))? {
                ids.insert(id?);
            }
        }
        Ok(ids.into_iter().collect())
    }

    pub fn delete_clip_ids(&self, ids: &[String]) -> AppResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let placeholders = std::iter::repeat_n("?", ids.len())
            .collect::<Vec<_>>()
            .join(",");
        transaction.execute(
            &format!("DELETE FROM clips_fts WHERE clip_id IN ({placeholders})"),
            params_from_iter(ids),
        )?;
        let changed = transaction.execute(
            &format!("DELETE FROM clips WHERE id IN ({placeholders})"),
            params_from_iter(ids),
        )?;
        transaction.commit()?;
        Ok(changed as u64)
    }

    pub fn delete_clip(&self, id: &str) -> AppResult<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute("DELETE FROM clips WHERE id = ?1", [id])?;
        if changed == 0 {
            return Err(AppError::NotFound);
        }
        transaction.execute("DELETE FROM clips_fts WHERE clip_id = ?1", [id])?;
        transaction.commit()?;
        Ok(())
    }

    pub fn clear(&self) -> AppResult<u64> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute("DELETE FROM clips_fts", [])?;
        let changed = transaction.execute("DELETE FROM clips", [])?;
        transaction.commit()?;
        Ok(changed as u64)
    }

    pub fn delete_older_than(&self, cutoff: DateTime<Utc>) -> AppResult<u64> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "DELETE FROM clips_fts WHERE clip_id IN (SELECT id FROM clips WHERE last_used_at < ?1)",
            [timestamp(cutoff)],
        )?;
        let changed = transaction.execute(
            "DELETE FROM clips WHERE last_used_at < ?1",
            [timestamp(cutoff)],
        )?;
        transaction.commit()?;
        Ok(changed as u64)
    }

    pub fn ids_older_than(&self, cutoff: DateTime<Utc>) -> AppResult<Vec<String>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT id FROM clips WHERE last_used_at < ?1")?;
        let rows = statement.query_map([timestamp(cutoff)], |row| row.get(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_setting<T: serde::de::DeserializeOwned>(&self, key: &str) -> AppResult<Option<T>> {
        let json: Option<String> = self
            .connection()?
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(AppError::from))
            .transpose()
    }

    pub fn set_setting<T: serde::Serialize>(&self, key: &str, value: &T) -> AppResult<()> {
        self.connection()?.execute(
            "INSERT INTO settings(key, value_json) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            params![key, serde_json::to_string(value)?],
        )?;
        Ok(())
    }

    pub fn update_setting<T>(&self, key: &str, update: impl FnOnce(&mut T)) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize + Default,
    {
        let connection = self.connection()?;
        let json: Option<String> = connection
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()?;
        let mut value = json
            .map(|value| serde_json::from_str(&value).map_err(AppError::from))
            .transpose()?
            .unwrap_or_default();
        update(&mut value);
        connection.execute(
            "INSERT INTO settings(key, value_json) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            params![key, serde_json::to_string(&value)?],
        )?;
        Ok(value)
    }
}

fn migrate_v4_to_v5(connection: &Connection) -> AppResult<()> {
    let transaction = connection.unchecked_transaction()?;
    transaction.execute(
        "ALTER TABLE clips ADD COLUMN last_used_at TEXT NOT NULL DEFAULT ''",
        [],
    )?;
    let rows = {
        let mut statement = transaction.prepare("SELECT id, created_at FROM clips")?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    for (id, created_at) in rows {
        let parsed = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|error| AppError::Storage(error.to_string()))?
            .with_timezone(&Utc);
        let normalized = timestamp(parsed);
        transaction.execute(
            "UPDATE clips SET created_at = ?1, last_used_at = ?1 WHERE id = ?2",
            params![normalized, id],
        )?;
    }
    transaction.execute("DROP INDEX idx_clips_order", [])?;
    transaction.execute(
        "CREATE INDEX idx_clips_order ON clips(last_used_at DESC, id DESC)",
        [],
    )?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Micros, true)
}

fn summary_from_row(row: &Row<'_>) -> rusqlite::Result<ClipSummary> {
    let source_id: Option<String> = row.get(3)?;
    let source_name: Option<String> = row.get(4)?;
    let content_type_text: String = row.get(1)?;
    let created_at_text: String = row.get(5)?;
    let metadata_text: String = row.get(7)?;
    Ok(ClipSummary {
        id: row.get(0)?,
        content_type: ContentType::from_str(&content_type_text).map_err(conversion_error)?,
        preview: row.get(2)?,
        source_app: source_id.zip(source_name).and_then(|(id, name)| {
            let source = SourceApp { id, name };
            source.is_meaningful().then_some(source)
        }),
        created_at: DateTime::parse_from_rfc3339(&created_at_text)
            .map_err(conversion_error)?
            .with_timezone(&Utc),
        byte_size: row.get(6)?,
        metadata: serde_json::from_str(&metadata_text).map_err(conversion_error)?,
    })
}

fn conversion_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        history::{Flavor, SourceApp},
        settings::{LanguagePreference, Settings},
    };
    use chrono::Duration;

    fn sample(text: &str, created_at: DateTime<Utc>) -> NewClip {
        NewClip {
            content_type: ContentType::Text,
            plain_text: Some(text.into()),
            preview: text.into(),
            source_app: Some(SourceApp {
                id: "com.example.editor".into(),
                name: "Editor".into(),
            }),
            flavors: vec![Flavor {
                format: "text/plain".into(),
                payload: text.as_bytes().to_vec(),
            }],
            metadata: crate::history::ClipMetadata {
                char_count: Some(text.chars().count() as u64),
                ..Default::default()
            },
            content_hash: format!("hash-{text}"),
            created_at,
        }
    }

    #[test]
    fn creates_inserts_searches_and_reads_details() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        let first_id = database.insert_clip(&sample("alpha command", now)).unwrap();
        database
            .insert_clip(&sample("beta paragraph", now - Duration::seconds(1)))
            .unwrap();

        let page = database
            .query_history(&HistoryQuery {
                query: "alpha".into(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].id, first_id);

        let detail = database.get_clip(&first_id).unwrap();
        assert_eq!(detail.plain_text.as_deref(), Some("alpha command"));
        assert_eq!(detail.flavors[0].format, "text/plain");
    }

    #[test]
    fn orders_newest_first_and_clears_history() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        database
            .insert_clip(&sample("older", now - Duration::minutes(1)))
            .unwrap();
        let newer = database.insert_clip(&sample("newer", now)).unwrap();

        let page = database.query_history(&HistoryQuery::default()).unwrap();
        assert_eq!(page.items[0].id, newer);
        assert_eq!(database.clear().unwrap(), 2);
        assert_eq!(
            database
                .query_history(&HistoryQuery::default())
                .unwrap()
                .total,
            0
        );
    }

    #[test]
    fn touching_an_old_text_moves_it_to_the_front() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        let older = database
            .insert_clip(&sample("older", now - Duration::minutes(1)))
            .unwrap();
        database.insert_clip(&sample("newer", now)).unwrap();

        assert!(database.touch_clip(&older).unwrap());
        assert_eq!(
            database
                .query_history(&HistoryQuery::default())
                .unwrap()
                .items[0]
                .id,
            older
        );
        assert_eq!(
            timestamp(database.get_clip(&older).unwrap().summary.created_at),
            timestamp(now - Duration::minutes(1))
        );
    }

    #[test]
    fn cleanup_candidates_combine_time_and_count_without_duplicates() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        let oldest = database
            .insert_clip(&sample("oldest", now - Duration::days(10)))
            .unwrap();
        let middle = database
            .insert_clip(&sample("middle", now - Duration::days(2)))
            .unwrap();
        database.insert_clip(&sample("newest", now)).unwrap();

        let ids = database
            .cleanup_candidate_ids(Some(now - Duration::days(7)), Some(1))
            .unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&oldest));
        assert!(ids.contains(&middle));
        assert_eq!(database.delete_clip_ids(&ids).unwrap(), 2);
        assert_eq!(
            database
                .query_history(&HistoryQuery::default())
                .unwrap()
                .total,
            1
        );
        assert!(database
            .cleanup_candidate_ids(None, None)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn detects_recent_duplicates_and_persists_settings() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        database.insert_clip(&sample("same", now)).unwrap();
        assert!(database
            .exists_recent_hash("hash-same", now - Duration::seconds(30))
            .unwrap());
        assert!(!database
            .exists_recent_hash("hash-same", now + Duration::seconds(1))
            .unwrap());

        database.set_setting("retention_days", &30_u32).unwrap();
        assert_eq!(
            database.get_setting::<u32>("retention_days").unwrap(),
            Some(30)
        );

        let settings = Settings {
            language: LanguagePreference::ChineseSimplified,
            ..Settings::default()
        };
        database.set_setting("app", &settings).unwrap();
        assert_eq!(
            database.get_setting::<Settings>("app").unwrap(),
            Some(settings)
        );
    }

    #[test]
    fn schema_is_versioned_and_setting_updates_are_atomic() {
        let database = Database::in_memory().unwrap();
        let version: u32 = database
            .connection()
            .unwrap()
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        database.set_setting("counter", &1_u32).unwrap();
        assert_eq!(
            database
                .update_setting("counter", |value: &mut u32| *value += 1)
                .unwrap(),
            2
        );
    }

    #[test]
    fn migrates_v4_last_used_time_without_changing_capture_time() {
        let connection = Connection::open_in_memory().unwrap();
        let old_schema = SCHEMA
            .lines()
            .filter(|line| line.trim() != "last_used_at TEXT NOT NULL,")
            .collect::<Vec<_>>()
            .join("\n")
            .replace(
                "ON clips(last_used_at DESC, id DESC)",
                "ON clips(created_at DESC, id DESC)",
            );
        connection.execute_batch(&old_schema).unwrap();
        connection.pragma_update(None, "user_version", 4).unwrap();
        connection.execute(
            "INSERT INTO clips (id, content_type, preview, created_at, content_hash, byte_size, metadata_json)
             VALUES ('old', 'text', 'old', '2026-01-01T00:00:00Z', 'hash', 0, '{}')",
            [],
        ).unwrap();

        let database = Database::from_connection(connection).unwrap();
        let stored: (String, String) = database
            .connection()
            .unwrap()
            .query_row(
                "SELECT created_at, last_used_at FROM clips WHERE id = 'old'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(stored.0, stored.1);
    }

    #[test]
    fn rejects_non_current_development_schema() {
        let connection = Connection::open_in_memory().unwrap();
        connection.pragma_update(None, "user_version", 2).unwrap();
        let error = Database::from_connection(connection).err().unwrap();
        assert!(error.to_string().contains("delete the database"));
    }
}
