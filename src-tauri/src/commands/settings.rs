use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{
    error::AppResult,
    settings::{validate_hotkey, Settings, SETTINGS_KEY},
    state::AppState,
};

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> AppResult<Settings> {
    let mut settings: Settings = state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default();
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
    validate_hotkey(&settings.hotkey)
        .map_err(|message| crate::error::AppError::Validation(message.into()))?;

    let previous: Settings = state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default();
    let hotkey_changed = previous.hotkey != settings.hotkey;
    let registered_new_hotkey = hotkey_changed && prepare_hotkey(&app, &settings.hotkey)?;

    let autostart = app.autolaunch();
    let previous_autostart = autostart.is_enabled().unwrap_or(false);
    let result = if settings.launch_at_login {
        autostart.enable()
    } else {
        autostart.disable()
    };
    if let Err(error) = result {
        cleanup_prepared_hotkey(&app, &settings.hotkey, registered_new_hotkey);
        return Err(crate::error::AppError::Platform(error.to_string()));
    }
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
        cleanup_prepared_hotkey(&app, &settings.hotkey, registered_new_hotkey);
    } else if hotkey_changed
        && app
            .global_shortcut()
            .is_registered(previous.hotkey.as_str())
    {
        // All user-visible state is committed before the old shortcut is
        // removed. If OS cleanup fails, both shortcuts may work until restart,
        // but the saved and primary shortcut remains the new one.
        if let Err(error) = app.global_shortcut().unregister(previous.hotkey.as_str()) {
            eprintln!("failed to unregister previous global shortcut: {error}");
        }
    }
    saved
}

fn register_hotkey(
    app: &AppHandle,
    hotkey: &str,
) -> Result<(), tauri_plugin_global_shortcut::Error> {
    app.global_shortcut()
        .on_shortcut(hotkey, |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                crate::window::toggle_panel(app);
            }
        })
}

fn prepare_hotkey(app: &AppHandle, next: &str) -> AppResult<bool> {
    if app.global_shortcut().is_registered(next) {
        return Ok(false);
    }
    if let Err(error) = register_hotkey(app, next) {
        return Err(crate::error::AppError::Platform(format!(
            "无法注册该快捷键，可能已被其他应用占用：{error}"
        )));
    }
    Ok(true)
}

fn cleanup_prepared_hotkey(app: &AppHandle, hotkey: &str, registered: bool) {
    if registered {
        if let Err(error) = app.global_shortcut().unregister(hotkey) {
            eprintln!("failed to clean up prepared global shortcut: {error}");
        }
    }
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
