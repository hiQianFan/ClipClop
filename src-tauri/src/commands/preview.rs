use tauri::{AppHandle, State};

use crate::{
    error::AppResult,
    onboarding::OnboardingExample,
    preview::PreviewResource,
    state::AppState,
    window::PreviewState,
    workflows::preview_clip::{self, PreviewOutcome},
};

#[tauri::command]
pub async fn get_clip_asset(state: State<'_, AppState>, id: String) -> AppResult<PreviewResource> {
    let preview = state.preview.clone();
    run_blocking(move || preview.asset(&id)).await
}

#[tauri::command]
pub async fn get_clip_file_asset(
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<PreviewResource> {
    let preview = state.preview.clone();
    run_blocking(move || preview.file_asset(&id, index)).await
}

#[tauri::command]
pub async fn get_clip_thumbnail(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PreviewResource> {
    let preview = state.preview.clone();
    run_blocking(move || preview.thumbnail(&id)).await
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
        &state.preview,
        &id,
        index.unwrap_or(0),
    )
}

#[tauri::command]
pub async fn get_source_app_icon(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PreviewResource> {
    let preview = state.preview.clone();
    run_blocking(move || preview.source_app_icon(&id)).await
}

/// Toggle Quick Look over a fixed onboarding example without touching history.
#[tauri::command]
pub fn preview_onboarding_example(
    app: AppHandle,
    state: State<'_, AppState>,
    preview_state: State<'_, PreviewState>,
    example: OnboardingExample,
) -> AppResult<PreviewOutcome> {
    if preview_state.is_active() {
        state.preview.close_native(&app, &preview_state)?;
        return Ok(PreviewOutcome::NativeClosed);
    }
    if state
        .preview
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
