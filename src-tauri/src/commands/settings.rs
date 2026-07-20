use tauri::{AppHandle, State};
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
    // The updater owns last_update_check. Preserve its current value when a
    // stale settings form is saved after an automatic check completes.
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
        return Err(crate::error::AppError::HotkeyUnavailable(error.to_string()));
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
