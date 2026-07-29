use tauri::{AppHandle, State};

use crate::{error::AppResult, preview::PreviewResource, state::AppState, window::PreviewState};

#[tauri::command]
pub fn get_clip_asset(state: State<'_, AppState>, id: String) -> AppResult<PreviewResource> {
    state.preview.asset(&id)
}

#[tauri::command]
pub fn get_clip_file_asset(
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<PreviewResource> {
    state.preview.file_asset(&id, index)
}

#[tauri::command]
pub fn get_clip_thumbnail(state: State<'_, AppState>, id: String) -> AppResult<PreviewResource> {
    state.preview.thumbnail(&id)
}

#[tauri::command]
pub fn open_clip(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.preview.open_clip(&app, &id)
}

#[tauri::command]
pub fn open_clip_file(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<()> {
    state.preview.open_clip_file(&app, &id, index)
}

#[tauri::command]
pub fn toggle_clip_preview(
    app: AppHandle,
    state: State<'_, AppState>,
    preview_state: State<'_, PreviewState>,
    id: String,
    index: usize,
) -> AppResult<bool> {
    state.preview.toggle(&app, &preview_state, &id, index)
}

#[tauri::command]
pub fn get_source_app_icon(state: State<'_, AppState>, app_id: String) -> PreviewResource {
    state.preview.source_app_icon(&app_id)
}
