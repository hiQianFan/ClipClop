use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    error::AppResult,
    settings::{Settings, DEFAULT_HOTKEY, SETTINGS_KEY},
    state::AppState,
};

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> AppResult<Settings> {
    let mut settings: Settings = state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default();
    // 快捷键录制尚未开放；设置页只展示实际生效的固定值。
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
    let previous_autostart = autostart.is_enabled().unwrap_or(false);
    let result = if settings.launch_at_login {
        autostart.enable()
    } else {
        autostart.disable()
    };
    result.map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    // last_update_check 由后台更新流程维护；保存表单时用库中现值覆盖前端传来的
    // 快照，避免用一份陈旧的时间戳把刚完成的检查记录冲掉。
    let saved = state
        .database
        .update_setting(SETTINGS_KEY, |existing: &mut Settings| {
            settings.last_update_check = existing.last_update_check.clone();
            *existing = settings.clone();
        });
    if saved.is_err() {
        let rollback = if previous_autostart {
            autostart.enable()
        } else {
            autostart.disable()
        };
        if let Err(error) = rollback {
            eprintln!("failed to restore autostart after settings write failure: {error}");
        }
    }
    saved
}

#[tauri::command]
pub fn record_update_check(state: State<'_, AppState>) -> AppResult<String> {
    let checked_at = chrono::Utc::now().to_rfc3339();
    state
        .database
        .update_setting(SETTINGS_KEY, |settings: &mut Settings| {
            settings.last_update_check = Some(checked_at.clone());
        })?;
    Ok(checked_at)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> AppResult<()> {
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Theme;

    #[test]
    fn defaults_are_minimal_and_local() {
        let settings = Settings::default();
        assert_eq!(settings.retention_days, 30);
        assert_eq!(settings.theme, Theme::System);
        assert!(settings.check_updates);
        assert!(settings.last_update_check.is_none());
    }
}
