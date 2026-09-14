use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

use crate::error::{AppError, AppResult};

use super::database::timestamp;

pub(super) const SCHEMA: &str = include_str!("../../schema.sql");
pub(super) const SCHEMA_VERSION: u32 = 10;
const COMPATIBILITY_METADATA_VERSION: u32 = 10;
// Raise this only when older readers/writers (including cleanup) are no longer safe.
const MIN_COMPATIBLE_VERSION: u32 = 10;
const METADATA: &str = "CREATE TABLE db_meta (
    key TEXT PRIMARY KEY NOT NULL,
    value INTEGER NOT NULL CHECK(typeof(value) = 'integer' AND value > 0)
);";

fn inspect(connection: &Connection) -> AppResult<u32> {
    let version: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version == 0 {
        if connection
            .prepare("SELECT 1 FROM sqlite_schema WHERE name NOT GLOB 'sqlite_*'")?
            .exists([])?
        {
            return Err(AppError::DatabaseInvalid(
                "unversioned non-empty database".into(),
            ));
        }
    } else if version >= COMPATIBILITY_METADATA_VERSION {
        let has_metadata = connection
            .prepare("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'db_meta'")?
            .exists([])?;
        if !has_metadata {
            return Err(AppError::DatabaseInvalid(format!(
                "schema {version} has no compatibility metadata"
            )));
        }
        let minimum: Option<u32> = connection
            .query_row(
                "SELECT value FROM db_meta WHERE key = 'min_compatible_version'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let minimum = minimum
            .filter(|minimum| (COMPATIBILITY_METADATA_VERSION..=version).contains(minimum))
            .ok_or_else(|| AppError::DatabaseInvalid("invalid compatibility metadata".into()))?;
        if minimum > SCHEMA_VERSION {
            return Err(AppError::DatabaseTooNew {
                version,
                required: minimum,
            });
        }
        if version >= SCHEMA_VERSION {
            validate_structure(connection)?;
        }
    } else if !(4..SCHEMA_VERSION).contains(&version) {
        return Err(AppError::DatabaseInvalid(format!(
            "unsupported legacy schema {version}"
        )));
    }
    Ok(version)
}

pub(super) fn initialize(connection: &mut Connection, path: Option<&Path>) -> AppResult<()> {
    let version = inspect(connection)?;
    if version >= SCHEMA_VERSION {
        if version > SCHEMA_VERSION {
            log::info!("using compatible future database schema {version}");
        }
        return Ok(());
    }
    if version > 0 {
        super::backup::verify(connection)?;
        if let Some(path) = path {
            super::backup::create(connection, path, version)?;
        }
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if inspect(&transaction)? != version {
        return Err(AppError::DatabaseInvalid(
            "database changed before migration; restart ClipClop".into(),
        ));
    }
    log::info!("migrating database schema {version} to {SCHEMA_VERSION}");
    let start = if version == 0 {
        transaction.execute_batch(SCHEMA)?;
        9 // schema.sql contains the data tables as of schema 9.
    } else {
        version
    };
    for step in start..SCHEMA_VERSION {
        match step {
            4 => migrate_v4_to_v5(&transaction)?,
            5 => migrate_v5_to_v6(&transaction)?,
            6 => migrate_v6_to_v7(&transaction)?,
            7 => migrate_v7_to_v8(&transaction)?,
            8 => {
                transaction.execute("ALTER TABLE clips ADD COLUMN favorited_at TEXT", [])?;
            }
            9 => {
                transaction.execute_batch(METADATA)?;
                transaction.execute(
                    "INSERT INTO db_meta(key, value) VALUES ('min_compatible_version', ?1)",
                    [MIN_COMPATIBLE_VERSION],
                )?;
            }
            _ => unreachable!(),
        }
    }
    transaction.execute(
        "UPDATE db_meta SET value = MAX(value, ?1) WHERE key = 'min_compatible_version'",
        [MIN_COMPATIBLE_VERSION],
    )?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    validate_structure(&transaction)?;
    super::backup::verify(&transaction)?;
    transaction.commit()?;
    log::info!("database migration committed: schema {SCHEMA_VERSION}");
    if version > 0 {
        if let Some(path) = path {
            if let Err(error) = super::backup::prune(path) {
                log::warn!("could not rotate database backups: {error}");
            }
        }
    }
    Ok(())
}

fn validate_structure(connection: &Connection) -> AppResult<()> {
    // Check the columns this build uses, not exact CREATE SQL: migrated databases
    // legitimately retain older CHECK expressions and column order.
    for sql in [
        "SELECT id, content_type, plain_text, preview, source_id, source_name, created_at, last_used_at, sort_at, favorited_at, content_hash, byte_size, metadata_json FROM clips LIMIT 0",
        "SELECT clip_id, format, inline_data, blob_path, byte_size FROM clip_flavors LIMIT 0",
        "SELECT clip_id, plain_text, preview, source_name FROM clips_fts WHERE clips_fts MATCH 'compatibility' LIMIT 0",
        "SELECT key, value_json FROM settings LIMIT 0",
    ] {
        connection.prepare(sql).map_err(|_| AppError::DatabaseInvalid("required tables or columns are missing".into()))?;
    }
    Ok(())
}

fn migrate_v4_to_v5(connection: &Connection) -> AppResult<()> {
    connection.execute(
        "ALTER TABLE clips ADD COLUMN last_used_at TEXT NOT NULL DEFAULT ''",
        [],
    )?;
    let rows = {
        let mut statement = connection.prepare("SELECT id, created_at FROM clips")?;
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
        connection.execute(
            "UPDATE clips SET created_at = ?1, last_used_at = ?1 WHERE id = ?2",
            params![normalized, id],
        )?;
    }
    connection.execute("DROP INDEX idx_clips_order", [])?;
    connection.execute(
        "CREATE INDEX idx_clips_order ON clips(last_used_at DESC, id DESC)",
        [],
    )?;
    connection.pragma_update(None, "user_version", 5)?;
    Ok(())
}

fn migrate_v5_to_v6(connection: &Connection) -> AppResult<()> {
    connection.execute(
        "ALTER TABLE clips ADD COLUMN sort_at TEXT NOT NULL DEFAULT ''",
        [],
    )?;
    connection.execute("UPDATE clips SET sort_at = last_used_at", [])?;
    connection.execute("DROP INDEX idx_clips_order", [])?;
    connection.execute(
        "CREATE INDEX idx_clips_order ON clips(sort_at DESC, id DESC)",
        [],
    )?;
    connection.pragma_update(None, "user_version", 6)?;
    Ok(())
}

fn migrate_v6_to_v7(connection: &Connection) -> AppResult<()> {
    connection.execute("DROP TABLE clips_fts", [])?;
    connection.execute_batch(
        "CREATE VIRTUAL TABLE clips_fts USING fts5(
           clip_id UNINDEXED, plain_text, preview, source_name, tokenize = 'trigram'
         );
         INSERT INTO clips_fts (clip_id, plain_text, preview, source_name)
         SELECT id, plain_text, preview, source_name FROM clips;",
    )?;
    connection.pragma_update(None, "user_version", 7)?;
    Ok(())
}

fn migrate_v7_to_v8(connection: &Connection) -> AppResult<()> {
    connection.execute(
        "UPDATE clips SET content_type = 'text' WHERE content_type = 'code'",
        [],
    )?;
    connection.pragma_update(None, "user_version", 8)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_initialization_leaves_an_empty_database_that_can_retry() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection.pragma_update(None, "max_page_count", 2).unwrap();
        assert!(initialize(&mut connection, None).is_err());
        assert_eq!(inspect(&connection).unwrap(), 0);
        connection
            .pragma_update(None, "max_page_count", 1000)
            .unwrap();
        initialize(&mut connection, None).unwrap();
        assert_eq!(inspect(&connection).unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn upgrades_released_schemas_preserving_history_payloads_and_settings() {
        for schema in [
            include_str!("fixtures/v4.sql"),
            include_str!("fixtures/v5.sql"),
            include_str!("fixtures/v6.sql"),
            include_str!("fixtures/v8.sql"),
            include_str!("fixtures/v9.sql"),
        ] {
            let mut connection = Connection::open_in_memory().unwrap();
            connection.execute_batch(schema).unwrap();
            let version = inspect(&connection).unwrap();
            let time = "2026-01-01T00:00:00.000000Z";
            let mut columns = "id, content_type, preview, created_at, content_hash".to_string();
            let mut values = "'old', 'text', 'sample', ?1, 'hash'".to_string();
            for (minimum, name) in [(5, "last_used_at"), (6, "sort_at")] {
                if version >= minimum {
                    columns.push_str(&format!(", {name}"));
                    values.push_str(", ?1");
                }
            }
            connection
                .execute(
                    &format!("INSERT INTO clips ({columns}) VALUES ({values})"),
                    [time],
                )
                .unwrap();
            connection.execute_batch(
                "INSERT INTO clip_flavors(clip_id,format,inline_data,byte_size) VALUES('old','text/plain',X'010203',3);
                 INSERT INTO clips_fts(clip_id,preview) VALUES('old','sample');
                 INSERT INTO settings VALUES('app', '{\"future_setting\":true}');"
            ).unwrap();
            if version == 9 {
                connection
                    .execute("UPDATE clips SET favorited_at=?1", [time])
                    .unwrap();
            }
            initialize(&mut connection, None).unwrap();
            assert_eq!(inspect(&connection).unwrap(), SCHEMA_VERSION);
            let row: (String, String, String, Option<String>) = connection
                .query_row(
                    "SELECT created_at,last_used_at,sort_at,favorited_at FROM clips WHERE id='old'",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .unwrap();
            assert_eq!((&*row.0, &*row.1, &*row.2), (time, time, time));
            assert_eq!(row.3, (version == 9).then(|| time.to_string()));
            assert_eq!(
                connection
                    .query_row("SELECT hex(inline_data) FROM clip_flavors", [], |row| {
                        row.get::<_, String>(0)
                    })
                    .unwrap(),
                "010203"
            );
            assert_eq!(
                connection
                    .query_row("SELECT value_json FROM settings", [], |row| row
                        .get::<_, String>(0))
                    .unwrap(),
                "{\"future_setting\":true}"
            );
            assert_eq!(
                connection
                    .query_row(
                        "SELECT clip_id FROM clips_fts WHERE clips_fts MATCH 'sample'",
                        [],
                        |row| row.get::<_, String>(0)
                    )
                    .unwrap(),
                "old"
            );
            initialize(&mut connection, None).unwrap();
        }
    }

    #[test]
    fn migration_turns_legacy_code_into_text() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(include_str!("fixtures/v6.sql"))
            .unwrap();
        connection.execute_batch("INSERT INTO clips(id,content_type,preview,created_at,last_used_at,sort_at,content_hash) VALUES('old','code','sample','2026-01-01','2026-01-01','2026-01-01','hash');").unwrap();
        migrate_v6_to_v7(&connection).unwrap();
        initialize(&mut connection, None).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT content_type FROM clips", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            "text"
        );
    }

    #[test]
    fn failed_late_migration_rolls_back_earlier_steps_and_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(include_str!("fixtures/v4.sql"))
            .unwrap();
        // Force the final metadata creation to fail, after every legacy migration ran.
        connection
            .execute_batch("CREATE TABLE db_meta(key TEXT);")
            .unwrap();
        let before: Vec<String> = connection
            .prepare("SELECT sql FROM sqlite_schema ORDER BY name")
            .unwrap()
            .query_map([], |row| {
                Ok(row.get::<_, Option<String>>(0)?.unwrap_or_default())
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(initialize(&mut connection, None).is_err());
        assert_eq!(inspect(&connection).unwrap(), 4);
        let after: Vec<String> = connection
            .prepare("SELECT sql FROM sqlite_schema ORDER BY name")
            .unwrap()
            .query_map([], |row| {
                Ok(row.get::<_, Option<String>>(0)?.unwrap_or_default())
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn rejects_unversioned_nonempty_and_incompatible_or_malformed_future_databases() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE TABLE sqliteExtra(value TEXT);")
            .unwrap();
        assert!(matches!(
            initialize(&mut connection, None),
            Err(AppError::DatabaseInvalid(_))
        ));
        assert!(!connection
            .prepare("SELECT 1 FROM sqlite_schema WHERE name='clips'")
            .unwrap()
            .exists([])
            .unwrap());
        for metadata in [None, Some(0), Some(9), Some(11), Some(13)] {
            let mut connection = Connection::open_in_memory().unwrap();
            initialize(&mut connection, None).unwrap();
            connection.pragma_update(None, "user_version", 12).unwrap();
            connection
                .execute_batch("DROP TABLE db_meta; CREATE TABLE db_meta(key TEXT, value INTEGER);")
                .unwrap();
            if let Some(minimum) = metadata {
                connection
                    .execute(
                        "INSERT INTO db_meta VALUES('min_compatible_version',?1)",
                        [minimum],
                    )
                    .unwrap();
            }
            let error = initialize(&mut connection, None).unwrap_err();
            if metadata == Some(11) {
                assert!(matches!(
                    error,
                    AppError::DatabaseTooNew {
                        version: 12,
                        required: 11
                    }
                ));
            }
            assert_eq!(
                connection
                    .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
                    .unwrap(),
                12
            );
        }
    }
}
