use tauri::{AppHandle, State, WebviewWindow};

use crate::{
    error::AppResult,
    history::{ClipDetail, HistoryFacets, HistoryPage, HistoryQuery},
    paste::PasteOutcome,
    state::AppState,
    window::{self, HideReason, QuickSelectionState},
    workflows::{clip_actions, paste_clip as paste_workflow},
};

#[tauri::command]
pub fn query_history(state: State<'_, AppState>, request: HistoryQuery) -> AppResult<HistoryPage> {
    state.history.query(&request)
}

#[tauri::command]
pub fn get_history_facets(
    state: State<'_, AppState>,
    request: HistoryQuery,
    source_query: Option<String>,
) -> AppResult<HistoryFacets> {
    state
        .history
        .facets(&request, source_query.as_deref().unwrap_or(""))
}

#[tauri::command]
pub fn get_clip(state: State<'_, AppState>, id: String) -> AppResult<ClipDetail> {
    state.history.get(&id)
}

#[tauri::command]
pub fn delete_clip(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    clip_actions::delete_clip(&app, &state.history, &state.external_preview, &id)
}

#[tauri::command]
pub fn clear_history(app: AppHandle, state: State<'_, AppState>) -> AppResult<u64> {
    clip_actions::clear_history(&app, &state.history, &state.external_preview)
}

#[tauri::command]
pub fn copy_clip(
    state: State<'_, AppState>,
    id: String,
    plain_text: Option<bool>,
) -> AppResult<bool> {
    clip_actions::copy_clip(
        &state.history,
        &state.settings,
        &id,
        plain_text.unwrap_or(false),
    )
}

#[tauri::command]
pub async fn paste_clip(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, AppState>,
    id: String,
    plain_text: Option<bool>,
) -> AppResult<PasteOutcome> {
    let history = state.history.clone();
    let paste = state.paste.clone();
    let settings = state.settings.clone();
    let window_label = window.label().to_string();
    tauri::async_runtime::spawn_blocking(move || {
        paste_workflow::paste_clip(
            &app,
            &history,
            &paste,
            &settings,
            &window_label,
            &id,
            plain_text.unwrap_or(false),
        )
    })
    .await
    .map_err(|error| crate::error::AppError::Platform(error.to_string()))?
}

#[tauri::command]
pub fn hide_panel(app: AppHandle, panel: WebviewWindow) -> AppResult<()> {
    window::hide_panel(&app, panel.label(), HideReason::Escape)
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))
}

#[tauri::command]
pub fn show_full_panel(app: AppHandle, selected_id: Option<String>, settings: Option<bool>) {
    window::open_full_panel(&app, selected_id, settings.unwrap_or(false));
}

#[tauri::command]
pub fn set_quick_selection(state: State<'_, QuickSelectionState>, id: Option<String>) {
    state.set(id);
}
