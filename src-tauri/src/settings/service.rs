use std::sync::{Arc, Mutex, MutexGuard};

use crate::{
    error::{AppError, AppResult},
    storage::Database,
};

use super::{LanguagePreference, Settings, SETTINGS_KEY};

#[derive(Clone)]
pub struct SettingsService {
    database: Arc<Database>,
    mutation: Arc<Mutex<()>>,
}

impl SettingsService {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            mutation: Arc::new(Mutex::new(())),
        }
    }

    pub fn get_stored(&self) -> AppResult<Settings> {
        Ok(self.database.get_setting(SETTINGS_KEY)?.unwrap_or_default())
    }

    pub fn set_internal(&self, settings: &Settings) -> AppResult<()> {
        let _guard = self.lock_mutation()?;
        self.database.set_setting(SETTINGS_KEY, settings)
    }

    pub fn update_preserving_check_time(&self, mut settings: Settings) -> AppResult<Settings> {
        // Preserve an updater timestamp written after the settings form was loaded.
        self.database
            .update_setting(SETTINGS_KEY, |existing: &mut Settings| {
                settings.last_update_check = existing.last_update_check.clone();
                settings.skipped_update_version = existing.skipped_update_version.clone();
                *existing = settings.clone();
            })
    }

    pub fn record_update_check(&self) -> AppResult<String> {
        let _guard = self.lock_mutation()?;
        let checked_at = chrono::Utc::now().to_rfc3339();
        self.database
            .update_setting(SETTINGS_KEY, |settings: &mut Settings| {
                settings.last_update_check = Some(checked_at.clone());
            })?;
        Ok(checked_at)
    }

    pub fn skip_update_version(&self, version: String) -> AppResult<()> {
        if version.is_empty() || version.len() > 80 || version.trim() != version {
            return Err(AppError::Validation("invalid update version".into()));
        }
        let _guard = self.lock_mutation()?;
        self.database
            .update_setting(SETTINGS_KEY, |settings: &mut Settings| {
                settings.skipped_update_version = Some(version.clone());
            })?;
        Ok(())
    }

    pub fn set_language(&self, language: LanguagePreference) -> AppResult<LanguagePreference> {
        let _guard = self.lock_mutation()?;
        self.database
            .update_setting(SETTINGS_KEY, |settings: &mut Settings| {
                settings.language = language;
            })?;
        Ok(language)
    }

    pub fn lock_mutation(&self) -> AppResult<MutexGuard<'_, ()>> {
        self.mutation
            .lock()
            .map_err(|_| AppError::Platform("settings mutation lock poisoned".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, thread, time::Duration};

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

    #[test]
    fn update_check_waits_for_an_active_settings_mutation() {
        let service = SettingsService::new(Arc::new(Database::in_memory().unwrap()));
        let guard = service.lock_mutation().unwrap();
        let worker = service.clone();
        let (sent, received) = mpsc::channel();
        thread::spawn(move || {
            sent.send(worker.record_update_check()).unwrap();
        });

        assert!(received.recv_timeout(Duration::from_millis(20)).is_err());
        drop(guard);
        assert!(received
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .is_ok());
    }

    #[test]
    fn language_update_preserves_every_other_setting() {
        let service = SettingsService::new(Arc::new(Database::in_memory().unwrap()));
        let before = Settings {
            retention_days: Some(90),
            last_update_check: Some("now".into()),
            ..Settings::default()
        };
        service.set_internal(&before).unwrap();
        service.set_language(LanguagePreference::English).unwrap();
        assert_eq!(
            service.get_stored().unwrap(),
            Settings {
                language: LanguagePreference::English,
                ..before
            }
        );
    }

}
