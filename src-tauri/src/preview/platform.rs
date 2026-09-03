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
    let Some(executable) = quicklook_executable() else {
        if quicklook_store_install_detected() {
            return PreviewCapability::unavailable(PreviewUnavailableReason::UnsupportedInstall);
        }
        return PreviewCapability::unavailable(PreviewUnavailableReason::NotInstalled);
    };
    let version_label = file_version(&executable)
        .map(|version| format!("{}.{}.{}.{}", version.0, version.1, version.2, version.3));
    let attach_version = |mut capability: PreviewCapability| {
        if let Some(version) = &version_label {
            capability = capability.with_version(version.clone());
        }
        capability
    };
    let capability = match is_elevated() {
        Ok(true) => attach_version(PreviewCapability::unavailable(
            PreviewUnavailableReason::Elevated,
        )),
        Err(()) => attach_version(PreviewCapability::unavailable(
            PreviewUnavailableReason::DetectionFailed,
        )),
        Ok(false) => attach_version(PreviewCapability::ready(PreviewProvider::Quicklook)),
    };
    log::debug!(
        "Windows preview capability: provider={:?} reason={:?} version={:?}",
        capability.provider,
        capability.reason,
        capability.version
    );
    capability
}

#[cfg(target_os = "windows")]
fn file_version(path: &Path) -> Option<(u16, u16, u16, u16)> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, ptr::null_mut};
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    };

    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let size = unsafe { GetFileVersionInfoSizeW(wide.as_ptr(), null_mut()) };
    if size == 0 {
        return None;
    }
    let mut data = vec![0u8; size as usize];
    if unsafe { GetFileVersionInfoW(wide.as_ptr(), 0, size, data.as_mut_ptr().cast()) } == 0 {
        return None;
    }
    let mut fixed = null_mut::<c_void>();
    let mut fixed_len = 0;
    let root = ['\\' as u16, 0];
    if unsafe {
        VerQueryValueW(
            data.as_ptr().cast(),
            root.as_ptr(),
            &mut fixed,
            &mut fixed_len,
        )
    } == 0
        || fixed_len < std::mem::size_of::<VS_FIXEDFILEINFO>() as u32
    {
        return None;
    }
    let fixed = unsafe { &*fixed.cast::<VS_FIXEDFILEINFO>() };
    Some((
        (fixed.dwFileVersionMS >> 16) as u16,
        fixed.dwFileVersionMS as u16,
        (fixed.dwFileVersionLS >> 16) as u16,
        fixed.dwFileVersionLS as u16,
    ))
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
    app: &tauri::AppHandle,
    state: &crate::window::PreviewState,
    path: &Path,
) -> AppResult<bool> {
    let Some(executable) = quicklook_executable() else {
        return Ok(false);
    };
    if is_elevated().unwrap_or(true) || !path.is_file() {
        return Ok(false);
    }
    std::process::Command::new(executable)
        .arg(path)
        .spawn()
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    crate::window::yield_topmost_for_preview(app);
    if let Ok(mut active_path) = active_quicklook_path().lock() {
        *active_path = Some(path.to_path_buf());
    }
    state.set_active(true);
    log::info!("sent QuickLook toggle request");
    Ok(true)
}

#[cfg(target_os = "windows")]
fn active_quicklook_path() -> &'static std::sync::Mutex<Option<std::path::PathBuf>> {
    static PATH: std::sync::OnceLock<std::sync::Mutex<Option<std::path::PathBuf>>> =
        std::sync::OnceLock::new();
    PATH.get_or_init(|| std::sync::Mutex::new(None))
}

#[cfg(target_os = "windows")]
fn quicklook_executable() -> Option<std::path::PathBuf> {
    ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"]
        .into_iter()
        .find_map(|variable| {
            let root = std::env::var_os(variable).map(std::path::PathBuf::from)?;
            [
                root.join("Programs")
                    .join("QuickLook")
                    .join("QuickLook.exe"),
                root.join("QuickLook").join("QuickLook.exe"),
            ]
            .into_iter()
            .find(|candidate| candidate.is_file())
        })
}

#[cfg(target_os = "windows")]
fn quicklook_store_install_detected() -> bool {
    let Some(packages) = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .map(|path| path.join("Packages"))
    else {
        return false;
    };
    std::fs::read_dir(packages).ok().is_some_and(|entries| {
        entries.flatten().any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains("quicklook")
        })
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

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn close_quicklook(
    _app: &tauri::AppHandle,
    state: &crate::window::PreviewState,
) -> AppResult<()> {
    state.set_active(false);
    Ok(())
}

#[cfg(target_os = "windows")]
pub(super) fn close_quicklook(
    app: &tauri::AppHandle,
    state: &crate::window::PreviewState,
) -> AppResult<()> {
    let path = active_quicklook_path()
        .lock()
        .ok()
        .and_then(|mut active_path| active_path.take());
    if let (Some(executable), Some(path)) = (quicklook_executable(), path) {
        std::process::Command::new(executable)
            .arg(path)
            .spawn()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
        log::info!("sent QuickLook close toggle request");
    }
    state.set_active(false);
    crate::window::restore_topmost_after_preview_transition(app);
    Ok(())
}
