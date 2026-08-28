use std::path::Path;
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::history::{
    ClipDetail, ClipSummary, ContentType, Flavor, FlavorInfo, HistoryFacets, HistoryPage,
    HistoryQuery, HistorySourceOption, NewClip, SourceApp,
};

#[cfg(test)]
use super::migrations::{SCHEMA, SCHEMA_VERSION};

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
        super::migrations::initialize(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub(super) fn connection(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| AppError::Storage("database lock was poisoned".into()))
    }

    pub fn capture_clip(&self, clip: &NewClip) -> AppResult<String> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let captured_at = timestamp(clip.created_at);
        let existing_id = transaction
            .query_row(
                "SELECT id FROM clips WHERE content_hash = ?1 ORDER BY created_at DESC LIMIT 1",
                [&clip.content_hash],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        if let Some(id) = existing_id {
            let source_id = clip.source_app.as_ref().map(|source| source.id.as_str());
            let source_name = clip.source_app.as_ref().map(|source| source.name.as_str());
            transaction.execute(
                "UPDATE clips SET last_used_at = ?1, sort_at = ?1, source_id = ?2, source_name = ?3 WHERE id = ?4",
                params![captured_at, source_id, source_name, id],
            )?;
            transaction.execute(
                "UPDATE clips_fts SET source_name = ?1 WHERE clip_id = ?2",
                params![source_name, id],
            )?;
            transaction.commit()?;
            return Ok(id);
        }
        let id = Uuid::now_v7().to_string();
        let byte_size: usize = clip.flavors.iter().map(|flavor| flavor.payload.len()).sum();
        let source_id = clip.source_app.as_ref().map(|source| source.id.as_str());
        let source_name = clip.source_app.as_ref().map(|source| source.name.as_str());
        transaction.execute(
            "INSERT INTO clips (id, content_type, plain_text, preview, source_id, source_name, created_at, last_used_at, sort_at, content_hash, byte_size, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?7, ?8, ?9, ?10)",
            params![
                id,
                clip.content_type.to_string(),
                clip.plain_text,
                clip.preview,
                source_id,
                source_name,
                captured_at,
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

        let (conditions, values) = query_conditions(request, true, true);
        let where_clause = where_clause(&conditions);
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
            "SELECT c.id, c.content_type, c.preview, c.source_id, c.source_name, c.created_at, c.byte_size, c.metadata_json, c.last_used_at
             FROM clips c{where_clause}
             ORDER BY c.sort_at DESC, c.id DESC LIMIT ? OFFSET ?"
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

    pub fn history_facets(
        &self,
        request: &HistoryQuery,
        source_query: &str,
    ) -> AppResult<HistoryFacets> {
        let connection = self.connection()?;
        let (type_conditions, type_values) = query_conditions(request, false, true);
        let type_where = where_clause(&type_conditions);
        let mut type_statement = connection.prepare(&format!(
            "SELECT content_type, COUNT(*) FROM clips c{type_where} GROUP BY content_type"
        ))?;
        let type_counts = type_statement
            .query_map(params_from_iter(type_values), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
            })?
            .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
        let type_total = type_counts.values().sum();

        let mut statement = connection.prepare(
            "SELECT source_id, source_name FROM clips
             WHERE source_id IS NOT NULL AND source_name IS NOT NULL
               AND (source_name LIKE ?1 ESCAPE '\\' OR source_id LIKE ?1 ESCAPE '\\')
             GROUP BY source_id, source_name ORDER BY MAX(last_used_at) DESC LIMIT 20",
        )?;
        let pattern = format!("%{}%", like_pattern(source_query.trim()));
        let source_apps = statement
            .query_map([pattern], |row| {
                Ok(SourceApp {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let (availability_conditions, availability_values) = query_conditions(request, true, false);
        // ponytail: at most 20 tiny COUNT queries; replace with one conditional aggregate if profiling shows it matters.
        let mut sources = Vec::with_capacity(source_apps.len());
        for source in source_apps {
            let mut conditions = availability_conditions.clone();
            conditions.push("c.source_id = ?");
            let mut values = availability_values.clone();
            values.push(Value::Text(source.id.clone()));
            let count: u64 = connection.query_row(
                &format!("SELECT COUNT(*) FROM clips c{}", where_clause(&conditions)),
                params_from_iter(values),
                |row| row.get(0),
            )?;
            sources.push(HistorySourceOption {
                source,
                available: count > 0,
            });
        }
        Ok(HistoryFacets {
            type_total,
            type_counts,
            sources,
        })
    }

    pub fn get_clip(&self, id: &str) -> AppResult<ClipDetail> {
        let connection = self.connection()?;
        let (summary, plain_text) = connection
            .query_row(
                "SELECT id, content_type, preview, source_id, source_name, created_at, byte_size, metadata_json, last_used_at, plain_text
                 FROM clips WHERE id = ?1",
                [id],
                |row| Ok((summary_from_row(row)?, row.get(9)?)),
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

    pub fn touch_clip(&self, id: &str, promote: bool) -> AppResult<bool> {
        let used_at = timestamp(Utc::now());
        Ok(self.connection()?.execute(
            "UPDATE clips SET last_used_at = ?1, sort_at = CASE WHEN ?3 THEN ?1 ELSE sort_at END WHERE id = ?2",
            params![used_at, id, promote],
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
                "SELECT id FROM clips ORDER BY sort_at DESC, id DESC LIMIT -1 OFFSET ?1",
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
}

fn query_conditions(
    request: &HistoryQuery,
    include_content_type: bool,
    include_source: bool,
) -> (Vec<&'static str>, Vec<Value>) {
    let mut conditions = Vec::new();
    let mut values = Vec::new();
    for term in request.query.split_whitespace() {
        if term.chars().count() >= 3 {
            conditions.push("c.id IN (SELECT clip_id FROM clips_fts WHERE clips_fts MATCH ?)");
            values.push(Value::Text(format!("\"{}\"", term.replace('"', "\"\""))));
        } else {
            conditions.push("c.id IN (SELECT clip_id FROM clips_fts WHERE plain_text LIKE ? ESCAPE '\\' OR preview LIKE ? ESCAPE '\\' OR source_name LIKE ? ESCAPE '\\')");
            let pattern = format!("%{}%", like_pattern(term));
            values.extend([
                Value::Text(pattern.clone()),
                Value::Text(pattern.clone()),
                Value::Text(pattern),
            ]);
        }
    }
    if include_content_type {
        if let Some(content_type) = request.content_type {
            conditions.push("c.content_type = ?");
            values.push(Value::Text(content_type.to_string()));
        }
    }
    if include_source {
        if let Some(source_id) = request
            .source_id
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            conditions.push("c.source_id = ?");
            values.push(Value::Text(source_id.into()));
        }
    }
    if let Some(since) = request.since {
        conditions.push("c.last_used_at >= ?");
        values.push(Value::Text(timestamp(since)));
    }
    (conditions, values)
}

fn where_clause(conditions: &[&str]) -> String {
    if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    }
}

fn like_pattern(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub(super) fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Micros, true)
}

fn summary_from_row(row: &Row<'_>) -> rusqlite::Result<ClipSummary> {
    let source_id: Option<String> = row.get(3)?;
    let source_name: Option<String> = row.get(4)?;
    let content_type_text: String = row.get(1)?;
    let created_at_text: String = row.get(5)?;
    let last_used_at_text: String = row.get(8)?;
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
        last_used_at: DateTime::parse_from_rfc3339(&last_used_at_text)
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
        let first_id = database
            .capture_clip(&sample("alpha command", now))
            .unwrap();
        database
            .capture_clip(&sample("beta paragraph", now - Duration::seconds(1)))
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
    fn searches_substrings_and_combines_structured_filters() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        let mut wanted = sample("这是一个搜索功能测试", now);
        wanted.content_type = ContentType::Image;
        wanted.source_app = Some(SourceApp {
            id: "editor".into(),
            name: "Editor".into(),
        });
        database.capture_clip(&wanted).unwrap();
        let mut other = sample("搜索功能", now - Duration::days(10));
        other.source_app = Some(SourceApp {
            id: "browser".into(),
            name: "Browser".into(),
        });
        database.capture_clip(&other).unwrap();

        for query in ["搜索", "索功能"] {
            let page = database
                .query_history(&HistoryQuery {
                    query: query.into(),
                    ..Default::default()
                })
                .unwrap();
            assert!(!page.items.is_empty());
        }
        let page = database
            .query_history(&HistoryQuery {
                content_type: Some(ContentType::Image),
                source_id: Some("editor".into()),
                since: Some(now - Duration::days(1)),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(page.total, 1);
        let facets = database
            .history_facets(
                &HistoryQuery {
                    content_type: Some(ContentType::Image),
                    source_id: Some("editor".into()),
                    ..Default::default()
                },
                "",
            )
            .unwrap();
        assert_eq!(facets.type_total, 1);
        assert_eq!(facets.type_counts["image"], 1);
        assert!(facets
            .sources
            .iter()
            .find(|source| source.source.id == "browser")
            .is_some_and(|source| !source.available));
        assert_eq!(
            database
                .history_facets(&HistoryQuery::default(), "Brows")
                .unwrap()
                .sources[0]
                .source
                .id,
            "browser"
        );
    }

    #[test]
    fn orders_newest_first_and_clears_history() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        database
            .capture_clip(&sample("older", now - Duration::minutes(1)))
            .unwrap();
        let newer = database.capture_clip(&sample("newer", now)).unwrap();

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
            .capture_clip(&sample("older", now - Duration::minutes(1)))
            .unwrap();
        database.capture_clip(&sample("newer", now)).unwrap();

        assert!(database.touch_clip(&older, true).unwrap());
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
        assert!(database.get_clip(&older).unwrap().summary.last_used_at > now);
    }

    #[test]
    fn touching_without_promotion_updates_use_time_but_preserves_order() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        let older = database
            .capture_clip(&sample("older", now - Duration::minutes(1)))
            .unwrap();
        let newer = database.capture_clip(&sample("newer", now)).unwrap();

        assert!(database.touch_clip(&older, false).unwrap());
        let page = database.query_history(&HistoryQuery::default()).unwrap();
        assert_eq!(page.items[0].id, newer);
        assert!(database.get_clip(&older).unwrap().summary.last_used_at > now);
    }

    #[test]
    fn cleanup_candidates_combine_time_and_count_without_duplicates() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        let oldest = database
            .capture_clip(&sample("oldest", now - Duration::days(10)))
            .unwrap();
        let middle = database
            .capture_clip(&sample("middle", now - Duration::days(2)))
            .unwrap();
        database.capture_clip(&sample("newest", now)).unwrap();

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
    fn exact_capture_promotes_the_canonical_row_and_persists_settings() {
        let database = Database::in_memory().unwrap();
        let now = Utc::now();
        let original = database.capture_clip(&sample("same", now)).unwrap();
        let mut repeated = sample("same", now + Duration::minutes(5));
        repeated.source_app = Some(SourceApp {
            id: "com.example.new".into(),
            name: "New Source".into(),
        });
        assert_eq!(database.capture_clip(&repeated).unwrap(), original);
        let page = database.query_history(&HistoryQuery::default()).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(timestamp(page.items[0].created_at), timestamp(now));
        assert_eq!(
            timestamp(page.items[0].last_used_at),
            timestamp(repeated.created_at)
        );
        assert_eq!(
            page.items[0].source_app.as_ref().unwrap().name,
            "New Source"
        );
        let distinct = database
            .capture_clip(&sample("same ", repeated.created_at + Duration::seconds(1)))
            .unwrap();
        assert_ne!(distinct, original);
        assert_eq!(
            database
                .query_history(&HistoryQuery::default())
                .unwrap()
                .total,
            2
        );

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
            .filter(|line| {
                !matches!(
                    line.trim(),
                    "last_used_at TEXT NOT NULL," | "sort_at TEXT NOT NULL,"
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
            .replace(
                "ON clips(sort_at DESC, id DESC)",
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
        let stored: (String, String, String) = database
            .connection()
            .unwrap()
            .query_row(
                "SELECT created_at, last_used_at, sort_at FROM clips WHERE id = 'old'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(stored.0, stored.1);
        assert_eq!(stored.1, stored.2);
    }

    #[test]
    fn migrates_v5_order_time_to_v6_sort_time() {
        let connection = Connection::open_in_memory().unwrap();
        let old_schema = SCHEMA
            .lines()
            .filter(|line| line.trim() != "sort_at TEXT NOT NULL,")
            .collect::<Vec<_>>()
            .join("\n")
            .replace(
                "ON clips(sort_at DESC, id DESC)",
                "ON clips(last_used_at DESC, id DESC)",
            );
        connection.execute_batch(&old_schema).unwrap();
        connection.pragma_update(None, "user_version", 5).unwrap();
        connection.execute(
            "INSERT INTO clips (id, content_type, preview, created_at, last_used_at, content_hash, byte_size, metadata_json)
             VALUES ('old', 'text', 'old', '2026-01-01T00:00:00Z', '2026-02-01T00:00:00Z', 'hash', 0, '{}')",
            [],
        ).unwrap();

        let database = Database::from_connection(connection).unwrap();
        let stored: (String, String) = database
            .connection()
            .unwrap()
            .query_row(
                "SELECT last_used_at, sort_at FROM clips WHERE id = 'old'",
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
