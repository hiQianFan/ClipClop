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
    state
        .history
        .with_current(|environment| environment.history.query(&request))
}

#[tauri::command]
pub fn get_history_facets(
    state: State<'_, AppState>,
    request: HistoryQuery,
    source_query: Option<String>,
) -> AppResult<HistoryFacets> {
    state.history.with_current(|environment| {
        environment
            .history
            .facets(&request, source_query.as_deref().unwrap_or(""))
    })
}

#[tauri::command]
pub fn get_clip_page(state: State<'_, AppState>, id: String, page_size: u32) -> AppResult<u32> {
    state
        .history
        .with_current(|environment| environment.history.clip_page(&id, page_size))
}

#[tauri::command]
pub fn get_clip(state: State<'_, AppState>, id: String) -> AppResult<ClipDetail> {
    state
        .history
        .with_current(|environment| environment.history.get(&id))
}

#[tauri::command]
pub fn set_clip_favorite(state: State<'_, AppState>, id: String, favorite: bool) -> AppResult<()> {
    state
        .history
        .with_current(|environment| environment.history.set_favorite(&id, favorite))
}

#[tauri::command]
pub fn delete_clip(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.history.with_current(|environment| {
        clip_actions::delete_clip(
            &app,
            &environment.history,
            &environment.external_preview,
            &id,
        )
    })
}

#[tauri::command]
pub fn clear_history(app: AppHandle, state: State<'_, AppState>) -> AppResult<u64> {
    state.history.with_current(|environment| {
        clip_actions::clear_history(&app, &environment.history, &environment.external_preview)
    })
}

#[tauri::command]
pub fn copy_clip(
    state: State<'_, AppState>,
    id: String,
    plain_text: Option<bool>,
) -> AppResult<bool> {
    state.history.with_current(|environment| {
        clip_actions::copy_clip(
            &environment.history,
            &state.settings,
            &id,
            plain_text.unwrap_or(false),
        )
    })
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
        history.with_current(|environment| {
            paste_workflow::paste_clip(
                &app,
                &environment.history,
                &paste,
                &settings,
                &window_label,
                &id,
                plain_text.unwrap_or(false),
            )
        })
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
pub fn show_full_panel(
    app: AppHandle,
    selected_id: Option<String>,
    settings: Option<bool>,
    permission_guide: Option<bool>,
) {
    window::open_full_panel(
        &app,
        selected_id,
        settings.unwrap_or(false),
        permission_guide.unwrap_or(false),
    );
}

#[tauri::command]
pub fn set_quick_selection(state: State<'_, QuickSelectionState>, id: Option<String>) {
    state.set(id);
}
