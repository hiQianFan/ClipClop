use tauri::{AppHandle, Manager, State};

use crate::{error::AppResult, settings::Settings, state::AppState, workflows::settings_update};

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
    settings_update::update(&app, &state.settings, settings)
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
    state.settings.record_update_check()
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
