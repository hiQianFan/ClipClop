use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};

use super::database::timestamp;

pub(super) const SCHEMA: &str = include_str!("../../schema.sql");
pub(super) const SCHEMA_VERSION: u32 = 6;

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
        }
        5 => migrate_v5_to_v6(connection)?,
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
