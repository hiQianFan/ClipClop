use rusqlite::{params, OptionalExtension};

use crate::error::{AppError, AppResult};

use super::Database;

impl Database {
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
