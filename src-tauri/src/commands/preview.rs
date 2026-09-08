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
pub fn get_preview_capability() -> crate::preview::PreviewCapability {
    crate::preview::capability()
}

#[tauri::command]
pub async fn get_clip_asset(state: State<'_, AppState>, id: String) -> AppResult<PreviewResource> {
    let history = state.history.clone();
    run_blocking(move || history.with_current(|environment| environment.assets.asset(&id))).await
}

#[tauri::command]
pub async fn get_clip_file_asset(
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<PreviewResource> {
    let history = state.history.clone();
    run_blocking(move || {
        history.with_current(|environment| environment.assets.file_asset(&id, index))
    })
    .await
}

#[tauri::command]
pub async fn get_clip_thumbnail(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PreviewResource> {
    let history = state.history.clone();
    run_blocking(move || history.with_current(|environment| environment.assets.thumbnail(&id)))
        .await
}

#[tauri::command]
pub fn preview_clip(
    app: AppHandle,
    state: State<'_, AppState>,
    preview_state: State<'_, PreviewState>,
    id: String,
    index: Option<usize>,
) -> AppResult<PreviewOutcome> {
    log::info!("preview requested: index={}", index.unwrap_or(0));
    state.history.with_current(|environment| {
        preview_clip::preview(
            &app,
            &preview_state,
            &environment.external_preview,
            &id,
            index.unwrap_or(0),
        )
    })
}

#[tauri::command]
pub fn open_clip_link(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    origin_only: Option<bool>,
) -> AppResult<()> {
    state.history.with_current(|environment| {
        environment
            .external_preview
            .open_link(&app, &id, origin_only.unwrap_or(false))
    })
}

#[tauri::command]
pub async fn get_source_app_icon(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PreviewResource> {
    let history = state.history.clone();
    run_blocking(move || {
        history.with_current(|environment| environment.assets.source_app_icon(&id))
    })
    .await
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
            state.history.with_current(|environment| {
                environment
                    .external_preview
                    .close_native(&app, &preview_state)
            })?;
        }
        return Ok(PreviewOutcome::NativeClosed);
    }
    if preview_state.is_active() {
        state.history.with_current(|environment| {
            environment
                .external_preview
                .close_native(&app, &preview_state)
        })?;
    }
    if state.history.with_current(|environment| {
        environment
            .external_preview
            .toggle_onboarding_example(&app, &preview_state, example)
    })? {
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
