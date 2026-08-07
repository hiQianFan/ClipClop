use tauri::{AppHandle, State};

use crate::{
    assets::PreviewResource,
    error::AppResult,
    history::ContentType,
    onboarding::OnboardingExample,
    settings::SettingsService,
    state::AppState,
    window::PreviewState,
    workflows::preview_clip::{self, PreviewOutcome},
};

// Reading an original file is gated twice: the in-app switch (this file) decides
// whether the app touches the file at all, and macOS Full Disk Access decides
// whether the OS allows the read. With the switch OFF the app never reads, so no
// system permission prompt can fire while browsing. The switch lock is held across
// the read so a concurrent disable cannot slip through the check/read window.
#[tauri::command]
pub async fn get_clip_asset(state: State<'_, AppState>, id: String) -> AppResult<PreviewResource> {
    let assets = state.assets.clone();
    let history = state.history.clone();
    let settings = state.settings.clone();
    run_blocking(move || {
        let _guard = settings.lock_mutation()?;
        let content_type = history.content_type(&id)?;
        let enabled =
            !requires_file_preview_permission(content_type) || file_preview_enabled(&settings)?;
        with_file_preview_access(content_type, enabled, || assets.asset(&id))
    })
    .await
}

#[tauri::command]
pub async fn get_clip_file_asset(
    state: State<'_, AppState>,
    id: String,
    index: usize,
) -> AppResult<PreviewResource> {
    let assets = state.assets.clone();
    let settings = state.settings.clone();
    run_blocking(move || {
        let _guard = settings.lock_mutation()?;
        with_file_preview_access(ContentType::File, file_preview_enabled(&settings)?, || {
            assets.file_asset(&id, index)
        })
    })
    .await
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
    let _guard = state.settings.lock_mutation()?;
    if !preview_state.is_active()
        && state.history.content_type(&id)? == ContentType::File
        && !file_preview_enabled(&state.settings)?
    {
        return Ok(PreviewOutcome::NotPreviewable);
    }
    preview_clip::preview(
        &app,
        &preview_state,
        &state.external_preview,
        &id,
        index.unwrap_or(0),
    )
}

fn with_file_preview_access<T>(
    content_type: ContentType,
    enabled: bool,
    operation: impl FnOnce() -> AppResult<T>,
) -> AppResult<T> {
    if requires_file_preview_permission(content_type) && !enabled {
        return Err(crate::error::AppError::Validation(
            "file preview is disabled".into(),
        ));
    }
    operation()
}

fn requires_file_preview_permission(content_type: ContentType) -> bool {
    content_type == ContentType::File
}

fn file_preview_enabled(settings: &SettingsService) -> AppResult<bool> {
    #[cfg(target_os = "macos")]
    return Ok(settings.get_stored()?.file_preview_enabled);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = settings;
        Ok(true)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_file_content_requires_the_explicit_preview_switch() {
        assert!(requires_file_preview_permission(ContentType::File));
        assert!(!requires_file_preview_permission(ContentType::Image));
        assert!(!requires_file_preview_permission(ContentType::Text));
    }

    #[test]
    fn disabled_file_preview_never_runs_the_file_operation() {
        let called = std::cell::Cell::new(false);
        let result = with_file_preview_access(ContentType::File, false, || {
            called.set(true);
            Ok(())
        });
        assert!(result.is_err());
        assert!(!called.get());
    }
}
