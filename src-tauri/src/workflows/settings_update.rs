use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{
    error::{AppError, AppResult},
    settings::{validate_hotkey, HotkeyValidationError, Settings, SettingsService},
};

pub fn get(app: &AppHandle, service: &SettingsService) -> AppResult<Settings> {
    let mut settings = service.get_stored()?;
    settings.launch_at_login = app.autolaunch().is_enabled().unwrap_or(false);
    Ok(settings)
}

pub fn update(
    app: &AppHandle,
    service: &SettingsService,
    mut settings: Settings,
) -> AppResult<Settings> {
    validate(&settings)?;

    let previous = service.get_stored()?;
    let hotkey_changed = previous.hotkey != settings.hotkey;
    let registered_new_hotkey = hotkey_changed && prepare_hotkey(app, &settings.hotkey)?;

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
            cleanup_prepared_hotkey(app, &settings.hotkey, registered_new_hotkey);
            return Err(AppError::Platform(error.to_string()));
        }
    }

    let committed_autostart = autostart.is_enabled().unwrap_or_else(|error| {
        log::warn!("failed to read autostart state after settings update: {error}");
        settings.launch_at_login
    });
    if committed_autostart != settings.launch_at_login {
        cleanup_prepared_hotkey(app, &settings.hotkey, registered_new_hotkey);
        return Err(AppError::Platform(
            "autostart state did not match requested value".into(),
        ));
    }
    settings.launch_at_login = committed_autostart;

    let saved = service.update_preserving_check_time(settings.clone());
    if saved.is_err() && changed_autostart {
        let rollback = if previous_autostart {
            autostart.enable()
        } else {
            autostart.disable()
        };
        if let Err(error) = rollback {
            log::warn!("failed to restore autostart after settings write failure: {error}");
        }
        cleanup_prepared_hotkey(app, &settings.hotkey, registered_new_hotkey);
    } else if saved.is_err() {
        cleanup_prepared_hotkey(app, &settings.hotkey, registered_new_hotkey);
    } else if hotkey_changed
        && app
            .global_shortcut()
            .is_registered(previous.hotkey.as_str())
    {
        // Saved state is committed before the old shortcut is removed.
        if let Err(error) = app.global_shortcut().unregister(previous.hotkey.as_str()) {
            log::warn!("failed to unregister previous global shortcut: {error}");
        }
    }
    saved
}

fn validate(settings: &Settings) -> AppResult<()> {
    if !matches!(settings.retention_days, 7 | 30 | 90) {
        return Err(AppError::Validation(
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
        AppError::Hotkey(code)
    })
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
    register_hotkey(app, next).map_err(|error| AppError::HotkeyUnavailable(error.to_string()))?;
    Ok(true)
}

fn cleanup_prepared_hotkey(app: &AppHandle, hotkey: &str, registered: bool) {
    if registered {
        if let Err(error) = app.global_shortcut().unregister(hotkey) {
            log::warn!("failed to clean up prepared global shortcut: {error}");
        }
    }
}
