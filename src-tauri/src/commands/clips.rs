use serde::Serialize;
use tauri::State;

use crate::{
    clipboard::SystemClipboard,
    clips::{ClipDetail, ClipPage, CopyMode, ListClipsRequest},
    error::AppResult,
    state::AppState,
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
pub fn copy_clip(state: State<'_, AppState>, id: String, mode: CopyMode) -> AppResult<()> {
    SystemClipboard::write(state.clips.flavors(&id)?, mode)
}

#[derive(Serialize)]
pub struct ClipAssetDto {
    pub data_url: Option<String>,
}

#[tauri::command]
pub fn get_clip_asset(state: State<'_, AppState>, id: String) -> AppResult<ClipAssetDto> {
    let detail = state.clips.get(&id)?;
    let file_path = detail
        .summary
        .metadata
        .get("files")
        .and_then(|files| files.as_array())
        .and_then(|files| files.first())
        .and_then(|path| path.as_str());
    Ok(ClipAssetDto {
        data_url: SystemClipboard::preview_asset(&state.clips.flavors(&id)?, file_path)?,
    })
}
