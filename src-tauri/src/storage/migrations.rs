use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};

use super::database::timestamp;

pub(super) const SCHEMA: &str = include_str!("../../schema.sql");
pub(super) const SCHEMA_VERSION: u32 = 8;

pub(super) fn initialize(connection: &Connection) -> AppResult<()> {
    let version: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            connection.execute_batch(SCHEMA)?;
            connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        4 => {
            migrate_v4_to_v5(connection)?;
            migrate_v5_to_v6(connection)?;
            migrate_v6_to_v7(connection)?;
            migrate_v7_to_v8(connection)?;
        }
        5 => {
            migrate_v5_to_v6(connection)?;
            migrate_v6_to_v7(connection)?;
            migrate_v7_to_v8(connection)?;
        }
        6 => {
            migrate_v6_to_v7(connection)?;
            migrate_v7_to_v8(connection)?;
        }
        7 => migrate_v7_to_v8(connection)?,
        SCHEMA_VERSION => {}
        unsupported => {
            return Err(AppError::Storage(format!(
                "unsupported development database schema {unsupported}; delete the database and restart"
            )));
        }
    }
    Ok(())
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
    transaction.pragma_update(None, "user_version", 5)?;
    transaction.commit()?;
    Ok(())
}

fn migrate_v5_to_v6(connection: &Connection) -> AppResult<()> {
    let transaction = connection.unchecked_transaction()?;
    transaction.execute(
        "ALTER TABLE clips ADD COLUMN sort_at TEXT NOT NULL DEFAULT ''",
        [],
    )?;
    transaction.execute("UPDATE clips SET sort_at = last_used_at", [])?;
    transaction.execute("DROP INDEX idx_clips_order", [])?;
    transaction.execute(
        "CREATE INDEX idx_clips_order ON clips(sort_at DESC, id DESC)",
        [],
    )?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn migrate_v6_to_v7(connection: &Connection) -> AppResult<()> {
    let transaction = connection.unchecked_transaction()?;
    transaction.execute("DROP TABLE clips_fts", [])?;
    transaction.execute_batch(
        "CREATE VIRTUAL TABLE clips_fts USING fts5(
           clip_id UNINDEXED, plain_text, preview, source_name, tokenize = 'trigram'
         );
         INSERT INTO clips_fts (clip_id, plain_text, preview, source_name)
         SELECT id, plain_text, preview, source_name FROM clips;",
    )?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn migrate_v7_to_v8(connection: &Connection) -> AppResult<()> {
    let transaction = connection.unchecked_transaction()?;
    transaction.execute(
        "UPDATE clips SET content_type = 'text' WHERE content_type = 'code'",
        [],
    )?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_turns_legacy_code_into_text() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE clips(content_type TEXT NOT NULL);
                 INSERT INTO clips VALUES ('code');
                 PRAGMA user_version = 7;",
            )
            .unwrap();
        initialize(&connection).unwrap();
        let content_type: String = connection
            .query_row("SELECT content_type FROM clips", [], |row| row.get(0))
            .unwrap();
        assert_eq!(content_type, "text");
    }
}
