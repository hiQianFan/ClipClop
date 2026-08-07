use std::path::Path;

use crate::error::AppResult;

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

#[cfg(not(target_os = "macos"))]
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
