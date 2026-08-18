use tauri::{AppHandle, State};

use crate::{
    error::AppResult,
    onboarding::{self, AutoPasteReadiness, OnboardingState},
    settings::LanguagePreference,
    state::AppState,
};

#[cfg(target_os = "macos")]
use crate::error::AppError;

#[tauri::command]
pub fn get_onboarding_state(state: State<'_, AppState>) -> AppResult<OnboardingState> {
    state.onboarding.get()
}

#[tauri::command]
pub fn save_onboarding_state(
    state: State<'_, AppState>,
    onboarding: OnboardingState,
) -> AppResult<OnboardingState> {
    state.onboarding.save(onboarding)
}

#[tauri::command]
pub fn get_auto_paste_readiness() -> AppResult<AutoPasteReadiness> {
    Ok(onboarding::auto_paste_readiness())
}

#[tauri::command]
pub fn request_auto_paste_access() -> AppResult<bool> {
    Ok(onboarding::request_auto_paste_access())
}

#[tauri::command]
pub fn open_auto_paste_settings(app: AppHandle) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener()
            .open_url(
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                None::<&str>,
            )
            .map_err(|error| AppError::Platform(error.to_string()))?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = app;
    Ok(())
}

#[tauri::command]
pub fn set_language_preference(
    app: AppHandle,
    state: State<'_, AppState>,
    language: LanguagePreference,
) -> AppResult<LanguagePreference> {
    state.settings.set_language(language)?;
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let settings = state.settings.get_stored()?;
    if let Err(error) = crate::tray::refresh_menu(&app, &settings) {
        log::warn!("language saved but tray refresh failed: {error}");
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = app;
    Ok(language)
}
