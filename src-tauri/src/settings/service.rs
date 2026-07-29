use std::sync::Arc;

use crate::{error::AppResult, storage::Database};

use super::{Settings, SETTINGS_KEY};

#[derive(Clone)]
pub struct SettingsService {
    database: Arc<Database>,
}

impl SettingsService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn get_stored(&self) -> AppResult<Settings> {
        Ok(self.database.get_setting(SETTINGS_KEY)?.unwrap_or_default())
    }

    pub fn set_internal(&self, settings: &Settings) -> AppResult<()> {
        self.database.set_setting(SETTINGS_KEY, settings)
    }

    pub fn update_preserving_check_time(&self, mut settings: Settings) -> AppResult<Settings> {
        // Preserve an updater timestamp written after the settings form was loaded.
        self.database
            .update_setting(SETTINGS_KEY, |existing: &mut Settings| {
                settings.last_update_check = existing.last_update_check.clone();
                *existing = settings.clone();
            })
    }

    pub fn record_update_check(&self) -> AppResult<String> {
        let checked_at = chrono::Utc::now().to_rfc3339();
        self.database
            .update_setting(SETTINGS_KEY, |settings: &mut Settings| {
                settings.last_update_check = Some(checked_at.clone());
            })?;
        Ok(checked_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_settings_and_update_check_share_one_persistence_boundary() {
        let service = SettingsService::new(Arc::new(Database::in_memory().unwrap()));
        let mut settings = Settings {
            check_updates: false,
            ..Settings::default()
        };
        service.set_internal(&settings).unwrap();

        let checked_at = service.record_update_check().unwrap();
        settings.last_update_check = Some(checked_at);

        assert_eq!(service.get_stored().unwrap(), settings);
    }
}
