use tauri::{AppHandle, Emitter};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::{
    error::{AppError, AppResult},
    history::HistoryService,
    preview::ExternalPreviewService,
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
    history: &HistoryService,
    preview: &ExternalPreviewService,
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
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let Err(error) = crate::tray::refresh_menu(app, &saved) {
        log::warn!("settings saved but tray refresh failed: {error}");
    }
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
    let removed = match crate::workflows::clip_actions::apply_retention(
        app,
        history,
        preview,
        saved.retention_days,
        saved.history_limit,
    ) {
        Ok(removed) => removed,
        Err(error) => {
            log::warn!(
                "settings saved but immediate history cleanup failed; capture will retry: {error}"
            );
            0
        }
    };
    if removed > 0 {
        let _ = app.emit("history_changed", serde_json::json!({ "latest_id": null }));
    }
    if let Err(error) = app.emit(
        "settings_changed",
        serde_json::json!({ "theme": saved.theme, "language": saved.language }),
    ) {
        log::warn!("settings saved but UI preference broadcast failed: {error}");
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

    // Refresh an existing Windows registry entry so upgrades pick up the current startup
    // arguments (notably `--autostart`, which keeps login launches in the background).
    #[cfg(target_os = "windows")]
    if expected && actual {
        return autostart.enable().map_err(|error| {
            AppError::Platform(format!("failed to refresh autostart registration: {error}"))
        });
    }

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
    if !matches!(settings.retention_days, None | Some(1 | 7 | 30 | 90 | 365)) {
        return Err(AppError::Validation(
            "retention_days must be null, 1, 7, 30, 90, or 365".into(),
        ));
    }
    if !matches!(settings.history_limit, None | Some(100 | 500 | 1000 | 5000)) {
        return Err(AppError::Validation(
            "history_limit must be null, 100, 500, 1000, or 5000".into(),
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
