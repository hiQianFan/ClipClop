use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{error::AppResult, state::AppState};

const SETTINGS_KEY: &str = "app";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub retention_days: u32,
    pub capture_paused: bool,
    pub launch_at_login: bool,
    pub hotkey: String,
    pub ignored_apps: Vec<String>,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            retention_days: 30,
            capture_paused: false,
            launch_at_login: false,
            hotkey: default_hotkey(),
            ignored_apps: Vec::new(),
            theme: Theme::System,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

fn default_hotkey() -> String {
    if cfg!(target_os = "macos") {
        "CommandOrControl+Shift+C".into()
    } else {
        "Ctrl+Shift+C".into()
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    Ok(state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default())
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<Settings> {
    if !matches!(settings.retention_days, 7 | 30 | 90) {
        return Err(crate::error::AppError::Validation(
            "retention_days must be 7, 30, or 90".into(),
        ));
    }
    if settings.hotkey.trim().is_empty() || settings.hotkey.chars().count() > 80 {
        return Err(crate::error::AppError::Validation(
            "hotkey must contain between 1 and 80 characters".into(),
        ));
    }
    state.database.set_setting(SETTINGS_KEY, &settings)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_minimal_and_local() {
        let settings = Settings::default();
        assert_eq!(settings.retention_days, 30);
        assert!(!settings.capture_paused);
        assert!(settings.ignored_apps.is_empty());
        assert_eq!(settings.theme, Theme::System);
    }
}
