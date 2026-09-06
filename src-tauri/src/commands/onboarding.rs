use serde::Serialize;
use tauri::{AppHandle, State};

#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};

use crate::{
    error::AppResult, onboarding::OnboardingState, paste::InjectionPermission,
    settings::LanguagePreference, state::AppState,
};

#[cfg(target_os = "macos")]
use crate::error::AppError;

#[derive(Serialize)]
pub struct AutoPastePermissionStatus {
    status: InjectionPermission,
    app_location: &'static str,
    app_path: Option<String>,
}

#[cfg(target_os = "macos")]
fn current_app_bundle() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    app_bundle_for_executable(&executable)
}

#[cfg(target_os = "macos")]
fn app_bundle_for_executable(executable: &Path) -> Option<PathBuf> {
    let macos = executable.parent()?;
    let contents = macos.parent()?;
    let bundle = contents.parent()?;
    (macos.file_name()? == "MacOS"
        && contents.file_name()? == "Contents"
        && bundle
            .extension()
            .is_some_and(|extension| extension == "app"))
    .then(|| bundle.to_path_buf())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn resolves_only_a_standard_macos_app_bundle() {
        assert_eq!(
            app_bundle_for_executable(Path::new(
                "/Applications/ClipClop.app/Contents/MacOS/clipclop"
            )),
            Some(PathBuf::from("/Applications/ClipClop.app"))
        );
        assert_eq!(
            app_bundle_for_executable(Path::new("/work/target/debug/clipclop")),
            None
        );
        assert_eq!(
            app_location(Some(Path::new("/ApplicationsElse/ClipClop.app"))),
            "other_bundle"
        );
        assert_eq!(
            app_location(Some(Path::new("/Users/test/Applications/ClipClop.app"))),
            "other_bundle"
        );
    }
}

#[cfg(target_os = "macos")]
fn app_location(bundle: Option<&Path>) -> &'static str {
    match bundle {
        Some(path) if path.starts_with("/Applications") => "applications",
        Some(_) => "other_bundle",
        None => "development",
    }
}

#[tauri::command]
pub fn get_auto_paste_permission_status(app: AppHandle) -> AppResult<AutoPastePermissionStatus> {
    #[cfg(target_os = "macos")]
    {
        let bundle = current_app_bundle();
        let location = app_location(bundle.as_deref());
        let status = crate::paste::injection_permission();
        log::info!(
            "automatic paste permission checked: status={status:?} version={} bundle_id={} app_location={location}",
            app.package_info().version,
            app.config().identifier,
        );
        Ok(AutoPastePermissionStatus {
            status,
            app_location: location,
            app_path: bundle.map(|path| path.to_string_lossy().into_owned()),
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        Ok(AutoPastePermissionStatus {
            status: InjectionPermission::Unsupported,
            app_location: "unsupported",
            app_path: None,
        })
    }
}

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
pub fn open_auto_paste_settings(app: AppHandle) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_opener::OpenerExt;
        log::info!(
            "automatic paste permission requested: app_location={}",
            app_location(current_app_bundle().as_deref())
        );
        crate::paste::request_accessibility_permission();
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
pub fn reveal_current_app(_app: AppHandle) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let bundle = current_app_bundle().ok_or_else(|| {
            AppError::Platform("current process is not running from an app bundle".into())
        })?;
        log::info!(
            "revealing current app bundle: app_location={}",
            app_location(Some(&bundle))
        );
        std::process::Command::new("/usr/bin/open")
            .arg("-R")
            .arg(bundle)
            .status()
            .map_err(|error| AppError::Platform(error.to_string()))?
            .success()
            .then_some(())
            .ok_or_else(|| AppError::Platform("Finder could not reveal the current app".into()))?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = _app;
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
