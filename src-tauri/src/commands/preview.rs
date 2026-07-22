use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    clipboard::SystemClipboard,
    clips::{ClipDetail, ContentType},
    error::{AppError, AppResult},
    state::AppState,
    window::PreviewState,
};

#[derive(Serialize)]
pub struct ClipAssetDto {
    pub data_url: Option<String>,
    pub byte_size: Option<u64>,
}

#[tauri::command]
pub fn get_clip_asset(state: State<'_, AppState>, id: String) -> AppResult<ClipAssetDto> {
    let detail = state.clips.get(&id)?;
    let flavors = state.clips.flavors(&id)?;
    Ok(ClipAssetDto {
        data_url: SystemClipboard::preview_asset(&flavors, file_path(&detail))?,
        byte_size: None,
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
    let file_path = file_path_at(&detail, index);
    Ok(ClipAssetDto {
        data_url: SystemClipboard::preview_asset(&flavors, file_path)?,
        byte_size: file_path
            .map(normalized_path)
            .and_then(|path| std::fs::metadata(path).ok())
            .filter(|metadata| metadata.is_file())
            .map(|metadata| metadata.len()),
    })
}

#[tauri::command]
pub fn get_clip_thumbnail(state: State<'_, AppState>, id: String) -> AppResult<ClipAssetDto> {
    state.clips.get(&id)?;
    let flavors = state.clips.flavors(&id)?;
    Ok(ClipAssetDto {
        data_url: SystemClipboard::thumbnail_asset(&flavors)?,
        byte_size: None,
    })
}

#[tauri::command]
pub fn open_clip(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    let detail = state.clips.get(&id)?;
    match detail.summary.content_type {
        ContentType::File => {
            let path = file_path(&detail)
                .map(normalized_path)
                .ok_or(AppError::NotFound)?;
            if !path.is_file() {
                return Err(AppError::NotFound);
            }
            open_path(&app, path)
        }
        ContentType::Link => {
            let url = detail.plain_text.unwrap_or(detail.summary.preview);
            let parsed = url::Url::parse(&url)
                .map_err(|_| AppError::Validation("link is not a valid URL".into()))?;
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err(AppError::Validation("link must use http or https".into()));
            }
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|error| AppError::Platform(error.to_string()))
        }
        ContentType::Image => {
            let flavors = state.clips.flavors(&id)?;
            let png = flavors
                .iter()
                .find(|flavor| flavor.format == "image/png")
                .ok_or(AppError::NotFound)?;
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
        return Err(AppError::Validation("clip is not a file record".into()));
    }
    let path = file_path_at(&detail, index)
        .map(normalized_path)
        .ok_or(AppError::NotFound)?;
    if !path.is_file() {
        return Err(AppError::NotFound);
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

    crate::window::install_quicklook_key_handler();
    let path = clip_preview_path(&app, &state, &id, index)?;
    let url = url::Url::from_file_path(&path)
        .map_err(|_| AppError::Validation("preview path is invalid".into()))?;

    preview.set_active(true);
    let result = (|| {
        app.quicklook()
            .set_items(vec![PreviewItem::new(url.to_string(), None)])
            .map_err(|error| AppError::Platform(error.to_string()))?;
        app.quicklook()
            .queue_reload_if_dirty()
            .map_err(|error| AppError::Platform(error.to_string()))?;
        app.quicklook()
            .queue_toggle_visible()
            .map_err(|error| AppError::Platform(error.to_string()))
    })();
    if result.is_err() {
        preview.set_active(false);
    }
    result?;
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

#[cfg(target_os = "macos")]
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
                .ok_or(AppError::NotFound)?;
            if !path.is_file() {
                return Err(AppError::NotFound);
            }
            Ok(path)
        }
        ContentType::Image => {
            let flavors = state.clips.flavors(id)?;
            let png = flavors
                .iter()
                .find(|flavor| flavor.format == "image/png")
                .ok_or(AppError::NotFound)?;
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
        .map_err(|error| AppError::Platform(error.to_string()))
}

fn preview_path(app: &AppHandle, id: &str, extension: &str) -> AppResult<PathBuf> {
    let dir = cached_preview_dir(app)?;
    std::fs::create_dir_all(&dir).map_err(|error| AppError::Platform(error.to_string()))?;
    Ok(dir.join(format!("{id}.{extension}")))
}

fn cached_preview_dir(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(app
        .path()
        .app_cache_dir()
        .map_err(|error| AppError::Platform(error.to_string()))?
        .join("external-preview"))
}

pub(super) fn delete_cached_previews(app: &AppHandle, id: &str) -> AppResult<()> {
    delete_cached_previews_in(&cached_preview_dir(app)?, id)
}

fn delete_cached_previews_in(dir: &Path, id: &str) -> AppResult<()> {
    for extension in ["png", "txt"] {
        match std::fs::remove_file(dir.join(format!("{id}.{extension}"))) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppError::Platform(error.to_string())),
        }
    }
    Ok(())
}

pub(super) fn clear_cached_previews(app: &AppHandle) -> AppResult<()> {
    match std::fs::remove_dir_all(cached_preview_dir(app)?) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::Platform(error.to_string())),
    }
}

fn write_preview(path: &Path, bytes: &[u8]) -> AppResult<()> {
    std::fs::write(path, bytes).map_err(|error| AppError::Platform(error.to_string()))
}

fn file_path(detail: &ClipDetail) -> Option<&str> {
    file_path_at(detail, 0)
}

fn file_path_at(detail: &ClipDetail, index: usize) -> Option<&str> {
    detail.summary.metadata.files.get(index).map(String::as_str)
}

fn normalized_path(path: &str) -> PathBuf {
    url::Url::parse(path)
        .ok()
        .filter(|url| url.scheme() == "file")
        .and_then(|url| url.to_file_path().ok())
        .unwrap_or_else(|| PathBuf::from(path))
}

#[tauri::command]
pub fn get_source_app_icon(app_id: String) -> ClipAssetDto {
    ClipAssetDto {
        data_url: SystemClipboard::source_app_icon(&app_id),
        byte_size: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deleting_one_clip_keeps_other_cached_previews() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("selected.png"), b"png").unwrap();
        std::fs::write(temp.path().join("selected.txt"), b"text").unwrap();
        std::fs::write(temp.path().join("other.png"), b"other").unwrap();

        delete_cached_previews_in(temp.path(), "selected").unwrap();

        assert!(!temp.path().join("selected.png").exists());
        assert!(!temp.path().join("selected.txt").exists());
        assert!(temp.path().join("other.png").exists());
    }
}
