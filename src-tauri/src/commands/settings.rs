use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{
    error::AppResult,
    settings::{validate_hotkey, HotkeyValidationError, Settings, SETTINGS_KEY},
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
    validate_hotkey(&settings.hotkey).map_err(|error| {
        let code = match error {
            HotkeyValidationError::InvalidFormat => "HOTKEY_INVALID_FORMAT",
            HotkeyValidationError::MissingModifier => "HOTKEY_MISSING_MODIFIER",
            HotkeyValidationError::UnsupportedKey => "HOTKEY_UNSUPPORTED_KEY",
            HotkeyValidationError::DuplicateModifier => "HOTKEY_DUPLICATE_MODIFIER",
            HotkeyValidationError::Reserved => "HOTKEY_RESERVED",
        };
        crate::error::AppError::Hotkey(code)
    })?;

    let previous: Settings = state
        .database
        .get_setting(SETTINGS_KEY)?
        .unwrap_or_default();
    let hotkey_changed = previous.hotkey != settings.hotkey;
    let registered_new_hotkey = hotkey_changed && prepare_hotkey(&app, &settings.hotkey)?;

    let autostart = app.autolaunch();
    let previous_autostart = autostart.is_enabled().unwrap_or_else(|error| {
        log::warn!("failed to read autostart state before settings update: {error}");
        false
    });
    let changed_autostart = previous_autostart != settings.launch_at_login;
    if changed_autostart {
        let result = if settings.launch_at_login {
            autostart.enable()
        } else {
            autostart.disable()
        };
        if let Err(error) = result {
            log::warn!("failed to update autostart state: {error}");
            cleanup_prepared_hotkey(&app, &settings.hotkey, registered_new_hotkey);
            return Err(crate::error::AppError::Platform(error.to_string()));
        }
    }

    let committed_autostart = autostart.is_enabled().unwrap_or_else(|error| {
        log::warn!("failed to read autostart state after settings update: {error}");
        settings.launch_at_login
    });
    if committed_autostart != settings.launch_at_login {
        cleanup_prepared_hotkey(&app, &settings.hotkey, registered_new_hotkey);
        return Err(crate::error::AppError::Platform(
            "autostart state did not match requested value".into(),
        ));
    }
    settings.launch_at_login = committed_autostart;

    // The updater owns last_update_check. Preserve its current value when a
    // stale settings form is saved after an automatic check completes.
    let saved = state
        .database
        .update_setting(SETTINGS_KEY, |existing: &mut Settings| {
            settings.last_update_check = existing.last_update_check.clone();
            *existing = settings.clone();
        });
    if saved.is_err() && changed_autostart {
        let rollback = if previous_autostart {
            autostart.enable()
        } else {
            autostart.disable()
        };
        if let Err(error) = rollback {
            log::warn!("failed to restore autostart after settings write failure: {error}");
        }
        cleanup_prepared_hotkey(&app, &settings.hotkey, registered_new_hotkey);
    } else if saved.is_err() {
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
            log::warn!("failed to unregister previous global shortcut: {error}");
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
        return Err(crate::error::AppError::HotkeyUnavailable(error.to_string()));
    }
    Ok(true)
}

fn cleanup_prepared_hotkey(app: &AppHandle, hotkey: &str, registered: bool) {
    if registered {
        if let Err(error) = app.global_shortcut().unregister(hotkey) {
            log::warn!("failed to clean up prepared global shortcut: {error}");
        }
    }
}

/// Opens the application log directory in the native file manager.
/// The log directory location is OS-resolved (macOS: ~/Library/Logs/<id>,
/// Windows: %LOCALAPPDATA%\<id>\logs). The actual file-open is handled in
/// Rust so the webview does not need a broad open-path capability.
#[tauri::command]
pub fn open_log_dir(app: AppHandle) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| crate::error::AppError::Platform(e.to_string()))?;
    // Ensure the directory exists before trying to open it (plugin may not
    // have written any entries yet on a fresh install).
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| crate::error::AppError::Platform(e.to_string()))?;
    log::info!("opening log directory: {}", log_dir.display());
    app.opener()
        .open_path(log_dir.to_string_lossy(), None::<&str>)
        .map_err(|e| crate::error::AppError::Platform(e.to_string()))
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
    use crate::settings::{LanguagePreference, Theme};

    #[test]
    fn defaults_are_minimal_and_local() {
        let settings = Settings::default();
        assert_eq!(settings.retention_days, 30);
        assert_eq!(settings.theme, Theme::System);
        assert_eq!(settings.language, LanguagePreference::System);
        assert!(settings.check_updates);
        assert!(settings.last_update_check.is_none());
    }
}
