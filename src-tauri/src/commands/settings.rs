use tauri::{AppHandle, Manager, State};

use crate::{error::AppResult, settings::Settings, state::AppState, workflows::settings_update};

fn open_url(app: &AppHandle, url: &str) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;

    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))
}

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> AppResult<Settings> {
    settings_update::get(&app, &state.settings)
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    settings_update::update(
        &app,
        &state.settings,
        &state.history,
        &state.external_preview,
        settings,
    )
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
pub fn open_release_page(app: AppHandle) -> AppResult<()> {
    open_url(&app, "https://github.com/hiQianFan/ClipClop/releases")
}

#[tauri::command]
pub fn open_repository(app: AppHandle) -> AppResult<()> {
    open_url(&app, "https://github.com/hiQianFan/ClipClop")
}

#[tauri::command]
pub fn open_quicklook_install_page(app: AppHandle) -> AppResult<()> {
    open_url(&app, "https://github.com/QL-Win/QuickLook#-get-started")
}

#[tauri::command]
pub fn record_update_check(state: State<'_, AppState>) -> AppResult<String> {
    state.settings.record_update_check()
}

#[tauri::command]
pub fn skip_update_version(state: State<'_, AppState>, version: String) -> AppResult<()> {
    state.settings.skip_update_version(version)
}

#[tauri::command]
pub fn open_file_preview_settings(app: AppHandle) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_url(
                "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
                None::<&str>,
            )
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = app;
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> AppResult<()> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn perform_pager_haptic() -> AppResult<()> {
    #[cfg(target_os = "macos")]
    unsafe {
        use objc::{class, msg_send, runtime::Object, sel, sel_impl};
        let performer: *mut Object = msg_send![class!(NSHapticFeedbackManager), defaultPerformer];
        let _: () = msg_send![performer, performFeedbackPattern: 1_i64 performanceTime: 0_i64];
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{LanguagePreference, Theme};

    #[test]
    fn defaults_are_minimal_and_local() {
        let settings = Settings::default();
        assert_eq!(settings.retention_days, Some(30));
        assert_eq!(settings.history_limit, Some(500));
        assert!(settings.move_used_to_top);
        assert!(!settings.restore_browse_position);
        assert!(!settings.preserve_search_conditions);
        assert!(!settings.file_preview_enabled);
        assert_eq!(settings.theme, Theme::System);
        assert_eq!(settings.language, LanguagePreference::System);
        assert!(settings.check_updates);
        assert!(settings.last_update_check.is_none());
    }
}
