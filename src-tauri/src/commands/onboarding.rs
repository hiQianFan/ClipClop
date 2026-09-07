use serde::Serialize;
use tauri::{AppHandle, State};
use tauri::{Emitter, Manager};

#[tauri::command]
pub async fn open_permission_guide(
    app: AppHandle,
    state: State<'_, AppState>,
    kind: String,
) -> AppResult<()> {
    if kind != "accessibility" && kind != "files" {
        return Err(crate::error::AppError::Platform(
            "invalid permission kind".into(),
        ));
    }
    #[cfg(target_os = "macos")]
    {
        let bundle = current_app_bundle();
        if let Some(window) = app.get_webview_window("permission-guide") {
            window
                .show()
                .map_err(|e| AppError::Platform(e.to_string()))?;
        } else {
            let theme = match state.settings.get_stored()?.theme {
                crate::settings::Theme::Light => Some(tauri::Theme::Light),
                crate::settings::Theme::Dark => Some(tauri::Theme::Dark),
                crate::settings::Theme::System => None,
            };
            tauri::WebviewWindowBuilder::new(
                &app,
                "permission-guide",
                tauri::WebviewUrl::App("permissions".into()),
            )
            .title("ClipClop")
            .theme(theme)
            .inner_size(430.0, 360.0)
            .resizable(false)
            .always_on_top(true)
            .build()
            .map_err(|e| AppError::Platform(e.to_string()))?;
        }
        log::info!(
            "permission guide opened: kind={kind} app_location={} draggable={}",
            app_location(bundle.as_deref()),
            bundle.is_some()
        );
        if kind == "accessibility" {
            open_auto_paste_settings(app)?;
        } else {
            super::open_file_preview_settings(app)?;
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, state);
    Ok(())
}

#[tauri::command]
pub fn start_current_app_drag(app: AppHandle) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        let bundle = current_app_bundle().ok_or_else(|| {
            AppError::Platform("current process is not running from an app bundle".into())
        })?;
        let window = app
            .get_webview_window("permission-guide")
            .ok_or_else(|| AppError::Platform("permission guide window is unavailable".into()))?;
        let path = bundle.to_string_lossy().into_owned();
        app.run_on_main_thread(move || unsafe {
            use objc::{class, msg_send, runtime::Object, sel, sel_impl};
            #[repr(C)]
            struct Point {
                x: f64,
                y: f64,
            }
            #[repr(C)]
            struct Size {
                width: f64,
                height: f64,
            }
            #[repr(C)]
            struct Rect {
                origin: Point,
                size: Size,
            }

            let Ok(native) = window.ns_window() else {
                return;
            };
            let native = native as *mut Object;
            let view: *mut Object = msg_send![native, contentView];
            let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let event: *mut Object = msg_send![application, currentEvent];
            if view.is_null() || event.is_null() {
                return;
            }
            let location: Point = msg_send![native, mouseLocationOutsideOfEventStream];
            let rect = Rect {
                origin: Point {
                    x: location.x - 32.0,
                    y: location.y - 32.0,
                },
                size: Size {
                    width: 64.0,
                    height: 64.0,
                },
            };
            let string: *mut Object = msg_send![class!(NSString), alloc];
            let string: *mut Object =
                msg_send![string, initWithBytes:path.as_ptr() length:path.len() encoding:4usize];
            let _: i8 = msg_send![view, dragFile:string fromRect:rect slideBack:1i8 event:event];
            let _: () = msg_send![string, release];
        })
        .map_err(|error| AppError::Platform(error.to_string()))?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = app;
    Ok(())
}

#[tauri::command]
pub fn close_permission_guide(app: AppHandle, restart: Option<bool>) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("permission-guide") {
        window
            .close()
            .map_err(|e| crate::error::AppError::Platform(e.to_string()))?;
    }
    if let Some(window) = app.get_webview_window("main") {
        window
            .show()
            .map_err(|e| crate::error::AppError::Platform(e.to_string()))?;
        if restart == Some(true) {
            window
                .emit("permission_restart_requested", ())
                .map_err(|e| crate::error::AppError::Platform(e.to_string()))?;
        }
        window
            .set_focus()
            .map_err(|e| crate::error::AppError::Platform(e.to_string()))?;
    }
    Ok(())
}

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
