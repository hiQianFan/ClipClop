use std::path::Path;

use crate::error::AppResult;

#[cfg(not(target_os = "macos"))]
use super::PreviewUnavailableReason;
use super::{PreviewCapability, PreviewProvider};

#[cfg(target_os = "macos")]
pub(super) fn capability() -> PreviewCapability {
    PreviewCapability::ready(PreviewProvider::MacosQuicklook)
}

#[cfg(target_os = "windows")]
pub(super) fn capability() -> PreviewCapability {
    match is_elevated() {
        Ok(true) => PreviewCapability::unavailable(PreviewUnavailableReason::Elevated),
        Err(()) => PreviewCapability::unavailable(PreviewUnavailableReason::DetectionFailed),
        Ok(false) => match peek_executable() {
            Some(_) => PreviewCapability::ready(PreviewProvider::PowertoysPeek),
            None => PreviewCapability::unavailable(PreviewUnavailableReason::NotInstalled),
        },
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn capability() -> PreviewCapability {
    PreviewCapability::unavailable(PreviewUnavailableReason::DetectionFailed)
}

#[cfg(target_os = "macos")]
pub(super) fn toggle_quicklook(
    app: &tauri::AppHandle,
    state: &crate::window::PreviewState,
    path: &Path,
) -> AppResult<bool> {
    use tauri_plugin_quicklook::{PreviewItem, QuicklookExt};

    crate::window::install_quicklook_key_handler();
    let url = url::Url::from_file_path(path)
        .map_err(|_| crate::error::AppError::Validation("preview path is invalid".into()))?;
    state.set_active(true);
    let result = (|| {
        app.quicklook()
            .set_items(vec![PreviewItem::new(url.to_string(), None)])
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
        app.quicklook()
            .queue_reload_if_dirty()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
        app.quicklook()
            .queue_toggle_visible()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))
    })();
    if result.is_err() {
        state.set_active(false);
    }
    result?;
    Ok(true)
}

#[cfg(target_os = "macos")]
pub(super) fn close_quicklook(
    app: &tauri::AppHandle,
    state: &crate::window::PreviewState,
) -> AppResult<()> {
    use tauri_plugin_quicklook::QuicklookExt;

    app.quicklook()
        .queue_toggle_visible()
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    state.set_active(false);
    Ok(())
}

#[cfg(target_os = "windows")]
pub(super) fn toggle_quicklook(
    _app: &tauri::AppHandle,
    _state: &crate::window::PreviewState,
    path: &Path,
) -> AppResult<bool> {
    let Some(executable) = peek_executable() else {
        return Ok(false);
    };
    if is_elevated().unwrap_or(true) || !path.is_file() {
        return Ok(false);
    }
    std::process::Command::new(executable)
        .arg(path)
        .spawn()
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    Ok(true)
}

#[cfg(target_os = "windows")]
fn peek_executable() -> Option<std::path::PathBuf> {
    ["LOCALAPPDATA", "ProgramFiles"]
        .into_iter()
        .find_map(|variable| {
            let candidate = std::env::var_os(variable)
                .map(std::path::PathBuf::from)?
                .join("PowerToys")
                .join("WinUI3Apps")
                .join("PowerToys.Peek.UI.exe");
            candidate.is_file().then_some(candidate)
        })
}

#[cfg(target_os = "windows")]
fn is_elevated() -> Result<bool, ()> {
    use std::{mem::size_of, ptr::null_mut};
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };

    unsafe {
        let mut token = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(());
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        ) != 0;
        CloseHandle(token);
        ok.then_some(elevation.TokenIsElevated != 0).ok_or(())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn toggle_quicklook(
    _app: &tauri::AppHandle,
    _state: &crate::window::PreviewState,
    _path: &Path,
) -> AppResult<bool> {
    Ok(false)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn close_quicklook(
    _app: &tauri::AppHandle,
    state: &crate::window::PreviewState,
) -> AppResult<()> {
    state.set_active(false);
    Ok(())
}
