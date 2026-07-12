use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

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

#[tauri::command]
pub fn open_settings(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("settings") {
        window
            .show()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
        window
            .set_focus()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings".into()))
        .title("ClipClop 设置")
        .inner_size(480.0, 420.0)
        .min_inner_size(440.0, 380.0)
        .resizable(false)
        .build()
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn ignore_source(state: State<'_, AppState>, app_id: String) -> AppResult<Settings> {
    if app_id.trim().is_empty() || app_id.chars().count() > 1024 {
        return Err(crate::error::AppError::Validation(
            "app_id must contain between 1 and 1024 characters".into(),
        ));
    }
    let mut settings: Settings = state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default();
    if !settings.ignored_apps.contains(&app_id) {
        settings.ignored_apps.push(app_id);
        state.database.set_setting(SETTINGS_KEY, &settings)?;
    }
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
