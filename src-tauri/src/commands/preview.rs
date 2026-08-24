use tauri::{AppHandle, State};

use crate::{
    assets::PreviewResource,
    error::AppResult,
    onboarding::OnboardingExample,
    state::AppState,
    window::PreviewState,
    workflows::preview_clip::{self, PreviewOutcome},
};

#[tauri::command]
pub async fn get_clip_asset(state: State<'_, AppState>, id: String) -> AppResult<PreviewResource> {
    let assets = state.assets.clone();
    run_blocking(move || assets.asset(&id)).await
}

#[tauri::command]
pub async fn get_clip_file_asset(
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<PreviewResource> {
    let assets = state.assets.clone();
    run_blocking(move || assets.file_asset(&id, index)).await
}

#[tauri::command]
pub async fn get_clip_thumbnail(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PreviewResource> {
    let assets = state.assets.clone();
    run_blocking(move || assets.thumbnail(&id)).await
}

#[tauri::command]
pub fn preview_clip(
    app: AppHandle,
    state: State<'_, AppState>,
    preview_state: State<'_, PreviewState>,
    id: String,
    index: Option<usize>,
) -> AppResult<PreviewOutcome> {
    preview_clip::preview(
        &app,
        &preview_state,
        &state.external_preview,
        &id,
        index.unwrap_or(0),
    )
}

#[tauri::command]
pub fn open_clip_link(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    origin_only: Option<bool>,
) -> AppResult<()> {
    state
        .external_preview
        .open_link(&app, &id, origin_only.unwrap_or(false))
}

#[tauri::command]
pub async fn get_source_app_icon(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PreviewResource> {
    let assets = state.assets.clone();
    run_blocking(move || assets.source_app_icon(&id)).await
}

/// Set Quick Look to the requested state over a fixed onboarding example.
#[tauri::command]
pub fn preview_onboarding_example(
    app: AppHandle,
    state: State<'_, AppState>,
    preview_state: State<'_, PreviewState>,
    example: OnboardingExample,
    open: bool,
) -> AppResult<PreviewOutcome> {
    if !open {
        if preview_state.is_active() {
            state.external_preview.close_native(&app, &preview_state)?;
        }
        return Ok(PreviewOutcome::NativeClosed);
    }
    if preview_state.is_active() {
        state.external_preview.close_native(&app, &preview_state)?;
    }
    if state
        .external_preview
        .toggle_onboarding_example(&app, &preview_state, example)?
    {
        return Ok(PreviewOutcome::NativeOpened);
    }
    Ok(PreviewOutcome::NotPreviewable)
}

async fn run_blocking<T: Send + 'static>(
    operation: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))?
}
