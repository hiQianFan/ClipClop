use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{
    error::{AppError, AppResult},
    settings::{validate_hotkey, HotkeyValidationError, Settings, SettingsService},
};

pub fn get(app: &AppHandle, service: &SettingsService) -> AppResult<Settings> {
    let mut settings = service.get_stored()?;
    settings.launch_at_login = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| AppError::Platform(format!("failed to read autostart state: {error}")))?;
    Ok(settings)
}

pub fn update(
    app: &AppHandle,
    service: &SettingsService,
    mut settings: Settings,
) -> AppResult<Settings> {
    let _guard = service.lock_mutation()?;
    validate(&settings)?;

    let previous = service.get_stored()?;
    let autostart = app.autolaunch();
    let previous_autostart = autostart
        .is_enabled()
        .map_err(|error| AppError::Platform(format!("failed to read autostart state: {error}")))?;
    let hotkey_changed = previous.hotkey != settings.hotkey;
    let registered_new_hotkey = hotkey_changed && prepare_hotkey(app, &settings.hotkey)?;
    let changed_autostart = previous_autostart != settings.launch_at_login;
    if changed_autostart {
        let result = if settings.launch_at_login {
            autostart.enable()
        } else {
            autostart.disable()
        };
        if let Err(error) = result {
            log::warn!("failed to update autostart state: {error}");
            let original = AppError::Platform(error.to_string());
            return Err(with_compensation(
                original,
                compensate(
                    app,
                    &settings.hotkey,
                    registered_new_hotkey,
                    Some(previous_autostart),
                ),
            ));
        }
    }

    let committed_autostart = match autostart.is_enabled() {
        Ok(value) => value,
        Err(error) => {
            let original = AppError::Platform(format!("failed to verify autostart state: {error}"));
            return Err(with_compensation(
                original,
                compensate(
                    app,
                    &settings.hotkey,
                    registered_new_hotkey,
                    changed_autostart.then_some(previous_autostart),
                ),
            ));
        }
    };
    if committed_autostart != settings.launch_at_login {
        let original = AppError::Platform("autostart state did not match requested value".into());
        return Err(with_compensation(
            original,
            compensate(
                app,
                &settings.hotkey,
                registered_new_hotkey,
                changed_autostart.then_some(previous_autostart),
            ),
        ));
    }
    settings.launch_at_login = committed_autostart;

    let saved = service
        .update_preserving_check_time(settings.clone())
        .map_err(|error| {
            with_compensation(
                error,
                compensate(
                    app,
                    &settings.hotkey,
                    registered_new_hotkey,
                    changed_autostart.then_some(previous_autostart),
                ),
            )
        })?;
    if hotkey_changed
        && app
            .global_shortcut()
            .is_registered(previous.hotkey.as_str())
    {
        // Saved state is committed before the old shortcut is removed.
        if let Err(error) = app.global_shortcut().unregister(previous.hotkey.as_str()) {
            let mut failures = compensate(
                app,
                &settings.hotkey,
                registered_new_hotkey,
                changed_autostart.then_some(previous_autostart),
            );
            if let Err(restore_error) = service.update_preserving_check_time(previous) {
                failures.push(format!("settings rollback: {restore_error}"));
            }
            return Err(with_compensation(
                AppError::Platform(format!(
                    "failed to unregister previous global shortcut: {error}"
                )),
                failures,
            ));
        }
    }
    Ok(saved)
}

pub fn reconcile_autostart(app: &AppHandle, service: &SettingsService) -> AppResult<()> {
    let _guard = service.lock_mutation()?;
    let expected = service.get_stored()?.launch_at_login;
    let autostart = app.autolaunch();
    let actual = autostart
        .is_enabled()
        .map_err(|error| AppError::Platform(format!("failed to read autostart state: {error}")))?;
    if actual == expected {
        return Ok(());
    }
    if expected {
        autostart.enable()
    } else {
        autostart.disable()
    }
    .map_err(|error| AppError::Platform(format!("failed to reconcile autostart: {error}")))
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

fn compensate(
    app: &AppHandle,
    hotkey: &str,
    registered: bool,
    restore_autostart: Option<bool>,
) -> Vec<String> {
    let mut failures = Vec::new();
    if let Some(enabled) = restore_autostart {
        let result = if enabled {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };
        if let Err(error) = result {
            failures.push(format!("autostart rollback: {error}"));
        }
    }
    if registered {
        if let Err(error) = app.global_shortcut().unregister(hotkey) {
            failures.push(format!("shortcut rollback: {error}"));
        }
    }
    failures
}

fn with_compensation(original: AppError, failures: Vec<String>) -> AppError {
    if failures.is_empty() {
        return original;
    }
    let diagnostic = format!("{original}; compensation failed: {}", failures.join("; "));
    log::error!("{diagnostic}");
    AppError::Platform(diagnostic)
}
