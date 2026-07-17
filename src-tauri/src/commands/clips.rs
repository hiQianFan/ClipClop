use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    clipboard::SystemClipboard,
    clips::{ClipDetail, ClipPage, ContentType, ListClipsRequest},
    error::AppResult,
    paste::PasteOutcome,
    state::AppState,
    PreviewState,
};

#[tauri::command]
pub fn list_clips(state: State<'_, AppState>, request: ListClipsRequest) -> AppResult<ClipPage> {
    state.clips.list(&request)
}

#[tauri::command]
pub fn get_clip(state: State<'_, AppState>, id: String) -> AppResult<ClipDetail> {
    state.clips.get(&id)
}

#[tauri::command]
pub fn delete_clip(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.clips.delete(&id)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> AppResult<u64> {
    state.clips.clear()
}

#[tauri::command]
pub fn copy_clip(state: State<'_, AppState>, id: String) -> AppResult<()> {
    SystemClipboard::write(state.clips.flavors(&id)?)
}

#[tauri::command]
pub fn paste_clip(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PasteOutcome> {
    SystemClipboard::write(state.clips.flavors(&id)?)?;
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    }
    let outcome = state.paste.paste_to_target();
    if outcome != PasteOutcome::Pasted {
        eprintln!("automatic paste degraded to clipboard-only: {outcome:?}");
    }
    Ok(outcome)
}

#[derive(Serialize)]
pub struct ClipAssetDto {
    pub data_url: Option<String>,
}

#[tauri::command]
pub fn get_clip_asset(state: State<'_, AppState>, id: String) -> AppResult<ClipAssetDto> {
    let detail = state.clips.get(&id)?;
    let flavors = state.clips.flavors(&id)?;
    Ok(ClipAssetDto {
        data_url: SystemClipboard::preview_asset(&flavors, file_path(&detail))?,
    })
}

#[tauri::command]
pub fn get_clip_file_asset(
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<ClipAssetDto> {
    let detail = state.clips.get(&id)?;
    let flavors = state.clips.flavors(&id)?;
    Ok(ClipAssetDto {
        data_url: SystemClipboard::preview_asset(&flavors, file_path_at(&detail, index))?,
    })
}

#[tauri::command]
pub fn get_clip_thumbnail(state: State<'_, AppState>, id: String) -> AppResult<ClipAssetDto> {
    let detail = state.clips.get(&id)?;
    let flavors = state.clips.flavors(&id)?;
    Ok(ClipAssetDto {
        data_url: SystemClipboard::thumbnail_asset(&flavors, file_path(&detail))?,
    })
}

#[tauri::command]
pub fn get_clip_file_thumbnail(
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<ClipAssetDto> {
    let detail = state.clips.get(&id)?;
    let flavors = state.clips.flavors(&id)?;
    Ok(ClipAssetDto {
        data_url: SystemClipboard::thumbnail_asset(&flavors, file_path_at(&detail, index))?,
    })
}

#[tauri::command]
pub fn open_clip(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    let detail = state.clips.get(&id)?;
    match detail.summary.content_type {
        ContentType::File => {
            let path = file_path(&detail)
                .map(normalized_path)
                .ok_or(crate::error::AppError::NotFound)?;
            if !Path::new(path).is_file() {
                return Err(crate::error::AppError::NotFound);
            }
            open_path(&app, path)
        }
        ContentType::Link => {
            let url = detail.plain_text.unwrap_or(detail.summary.preview);
            let parsed = url::Url::parse(&url).map_err(|_| {
                crate::error::AppError::Validation("link is not a valid URL".into())
            })?;
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err(crate::error::AppError::Validation(
                    "link must use http or https".into(),
                ));
            }
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|error| crate::error::AppError::Platform(error.to_string()))
        }
        ContentType::Image => {
            let flavors = state.clips.flavors(&id)?;
            let png = flavors
                .iter()
                .find(|flavor| flavor.format == "image/png")
                .ok_or(crate::error::AppError::NotFound)?;
            let path = preview_path(&app, &id, "png")?;
            write_preview(&path, &png.payload)?;
            open_path(&app, &path)
        }
        ContentType::Text | ContentType::Color | ContentType::Code => {
            let path = preview_path(&app, &id, "txt")?;
            write_preview(
                &path,
                detail
                    .plain_text
                    .unwrap_or(detail.summary.preview)
                    .as_bytes(),
            )?;
            open_path(&app, &path)
        }
    }
}

#[tauri::command]
pub fn open_clip_file(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<()> {
    let detail = state.clips.get(&id)?;
    if detail.summary.content_type != ContentType::File {
        return Err(crate::error::AppError::Validation(
            "clip is not a file record".into(),
        ));
    }
    let path = file_path_at(&detail, index)
        .map(normalized_path)
        .ok_or(crate::error::AppError::NotFound)?;
    if !Path::new(path).is_file() {
        return Err(crate::error::AppError::NotFound);
    }
    open_path(&app, path)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn toggle_clip_preview(
    app: AppHandle,
    state: State<'_, AppState>,
    preview: State<'_, PreviewState>,
    id: String,
    index: usize,
) -> AppResult<bool> {
    use tauri_plugin_quicklook::{PreviewItem, QuicklookExt};

    crate::install_quicklook_key_handler();
    let path = clip_preview_path(&app, &state, &id, index)?;
    let url = url::Url::from_file_path(&path)
        .map_err(|_| crate::error::AppError::Validation("preview path is invalid".into()))?;

    preview.set_active(true);
    app.quicklook()
        .set_items(vec![PreviewItem::new(url.to_string(), None)])
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    app.quicklook()
        .queue_reload_if_dirty()
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    app.quicklook()
        .queue_toggle_visible()
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    Ok(true)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub fn toggle_clip_preview(
    _app: AppHandle,
    _state: State<'_, AppState>,
    _preview: State<'_, PreviewState>,
    _id: String,
    _index: usize,
) -> AppResult<bool> {
    Ok(false)
}

fn clip_preview_path(
    app: &AppHandle,
    state: &State<'_, AppState>,
    id: &str,
    index: usize,
) -> AppResult<PathBuf> {
    let detail = state.clips.get(id)?;
    match detail.summary.content_type {
        ContentType::File => {
            let path = file_path_at(&detail, index)
                .map(normalized_path)
                .ok_or(crate::error::AppError::NotFound)?;
            let path = PathBuf::from(path);
            if !path.is_file() {
                return Err(crate::error::AppError::NotFound);
            }
            Ok(path)
        }
        ContentType::Image => {
            let flavors = state.clips.flavors(id)?;
            let png = flavors
                .iter()
                .find(|flavor| flavor.format == "image/png")
                .ok_or(crate::error::AppError::NotFound)?;
            let path = preview_path(app, id, "png")?;
            write_preview(&path, &png.payload)?;
            Ok(path)
        }
        ContentType::Text | ContentType::Color | ContentType::Code | ContentType::Link => {
            let path = preview_path(app, id, "txt")?;
            write_preview(
                &path,
                detail
                    .plain_text
                    .unwrap_or(detail.summary.preview)
                    .as_bytes(),
            )?;
            Ok(path)
        }
    }
}

fn open_path(app: &AppHandle, path: impl AsRef<Path>) -> AppResult<()> {
    app.opener()
        .open_path(path.as_ref().to_string_lossy(), None::<&str>)
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))
}

fn preview_path(app: &AppHandle, id: &str, extension: &str) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?
        .join("external-preview");
    std::fs::create_dir_all(&dir)
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    Ok(dir.join(format!("{id}.{extension}")))
}

fn write_preview(path: &Path, bytes: &[u8]) -> AppResult<()> {
    std::fs::write(path, bytes).map_err(|error| crate::error::AppError::Platform(error.to_string()))
}

fn file_path(detail: &ClipDetail) -> Option<&str> {
    file_path_at(detail, 0)
}

fn file_path_at(detail: &ClipDetail, index: usize) -> Option<&str> {
    detail
        .summary
        .metadata
        .get("files")
        .and_then(|files| files.as_array())
        .and_then(|files| files.get(index))
        .and_then(|path| path.as_str())
}

fn normalized_path(path: &str) -> &str {
    path.strip_prefix("file://").unwrap_or(path)
}

#[tauri::command]
pub fn get_source_app_icon(app_id: String) -> ClipAssetDto {
    ClipAssetDto {
        data_url: SystemClipboard::source_app_icon(&app_id),
    }
}

#[tauri::command]
pub fn hide_panel(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
    }
    Ok(())
}
