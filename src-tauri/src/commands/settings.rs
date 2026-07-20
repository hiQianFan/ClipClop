use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::{error::AppResult, state::AppState};

const SETTINGS_KEY: &str = "app";

#[cfg(target_os = "macos")]
pub const DEFAULT_HOTKEY: &str = "Control+Command+C";

#[cfg(not(target_os = "macos"))]
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+C";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub retention_days: u32,
    pub launch_at_login: bool,
    pub hotkey: String,
    pub ignored_apps: Vec<String>,
    pub theme: Theme,
    pub check_updates: bool,
    pub last_update_check: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            retention_days: 30,
            launch_at_login: false,
            hotkey: DEFAULT_HOTKEY.into(),
            ignored_apps: Vec::new(),
            theme: Theme::System,
            check_updates: true,
            last_update_check: None,
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

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> AppResult<Settings> {
    let mut settings: Settings = state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default();
    // 快捷键录制尚未开放；不要向旧配置或设置页暴露一个不会实际生效的值。
    settings.hotkey = DEFAULT_HOTKEY.into();
    settings.launch_at_login = app.autolaunch().is_enabled().unwrap_or(false);
    Ok(settings)
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    mut settings: Settings,
) -> AppResult<Settings> {
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
    let autostart = app.autolaunch();
    let result = if settings.launch_at_login {
        autostart.enable()
    } else {
        autostart.disable()
    };
    result.map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    // last_update_check 由后台更新流程维护；保存表单时用库中现值覆盖前端传来的
    // 快照，避免用一份陈旧的时间戳把刚完成的检查记录冲掉。
    let existing: Settings = state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default();
    settings.last_update_check = existing.last_update_check;
    state.database.set_setting(SETTINGS_KEY, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn record_update_check(state: State<'_, AppState>) -> AppResult<String> {
    let checked_at = chrono::Utc::now().to_rfc3339();
    let mut settings: Settings = state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default();
    settings.last_update_check = Some(checked_at.clone());
    state.database.set_setting(SETTINGS_KEY, &settings)?;
    Ok(checked_at)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> AppResult<()> {
    app.exit(0);
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
        assert!(settings.ignored_apps.is_empty());
        assert_eq!(settings.theme, Theme::System);
        assert!(settings.check_updates);
        assert!(settings.last_update_check.is_none());
    }

    #[test]
    fn old_settings_gain_update_defaults() {
        let settings: Settings = serde_json::from_str(
            r#"{"retention_days":30,"launch_at_login":false,"hotkey":"test","ignored_apps":[],"theme":"system"}"#,
        )
        .expect("old settings should remain readable");
        assert!(settings.check_updates);
        assert!(settings.last_update_check.is_none());
    }
}
