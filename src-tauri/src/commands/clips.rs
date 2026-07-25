use tauri::{AppHandle, State};

use crate::{
    clipboard::SystemClipboard,
    clips::{ClipDetail, ClipPage, ListClipsRequest},
    error::AppResult,
    paste::PasteOutcome,
    state::AppState,
    window::{self, HideReason},
};

use super::preview::{clear_cached_previews, delete_cached_previews};

const MAX_PREVIEW_CHARS: usize = 100_000;

#[tauri::command]
pub fn list_clips(state: State<'_, AppState>, request: ListClipsRequest) -> AppResult<ClipPage> {
    state.clips.list(&request)
}

#[tauri::command]
pub fn get_clip(state: State<'_, AppState>, id: String) -> AppResult<ClipDetail> {
    let mut detail = state.clips.get(&id)?;
    if let Some(text) = &mut detail.plain_text {
        truncate_preview(text);
    }
    Ok(detail)
}

fn truncate_preview(text: &mut String) {
    if let Some((byte_index, _)) = text.char_indices().nth(MAX_PREVIEW_CHARS) {
        text.truncate(byte_index);
        text.push('…');
    }
}

#[tauri::command]
pub fn delete_clip(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.clips.delete(&id)?;
    if let Err(error) = delete_cached_previews(&app, &id) {
        log::warn!("failed to delete cached preview for {id}: {error}");
    }
    Ok(())
}

#[tauri::command]
pub fn clear_history(app: AppHandle, state: State<'_, AppState>) -> AppResult<u64> {
    let cleared = state.clips.clear()?;
    if let Err(error) = clear_cached_previews(&app) {
        log::warn!("failed to clear cached previews: {error}");
    }
    Ok(cleared)
}

#[tauri::command]
pub fn copy_clip(
    state: State<'_, AppState>,
    id: String,
    plain_text: Option<bool>,
) -> AppResult<()> {
    SystemClipboard::write(state.clips.flavors(&id)?, plain_text.unwrap_or(false))
}

#[tauri::command]
pub fn paste_clip(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    plain_text: Option<bool>,
) -> AppResult<PasteOutcome> {
    let Some(permit) = state.paste.try_begin() else {
        return Ok(PasteOutcome::AlreadyInProgress);
    };
    SystemClipboard::write(state.clips.flavors(&id)?, plain_text.unwrap_or(false))?;
    if let Err(error) = window::hide_panel(&app, HideReason::Paste) {
        log::warn!("failed to hide panel before paste: {error}");
        return Ok(PasteOutcome::CopiedFocusFailed);
    }
    let outcome = state.paste.paste_to_target(permit);
    if outcome != PasteOutcome::Pasted {
        log::warn!("automatic paste degraded to clipboard-only: {outcome:?}");
    }
    Ok(outcome)
}

#[tauri::command]
pub fn hide_panel(app: AppHandle) -> AppResult<()> {
    window::hide_panel(&app, HideReason::Escape)
        .map_err(|error| crate::error::AppError::Platform(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_limit_preserves_utf8_boundaries() {
        let mut text = "界".repeat(MAX_PREVIEW_CHARS + 1);
        truncate_preview(&mut text);
        assert_eq!(text.chars().count(), MAX_PREVIEW_CHARS + 1);
        assert!(text.ends_with('…'));
    }
}
