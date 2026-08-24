use tauri::AppHandle;

use crate::{
    clipboard::SystemClipboard,
    error::AppResult,
    history::HistoryService,
    paste::{PasteController, PasteOutcome},
    window::{self, HideReason},
};

pub fn paste_clip(
    app: &AppHandle,
    history: &HistoryService,
    paste: &PasteController,
    settings: &crate::settings::SettingsService,
    id: &str,
    plain_text_only: bool,
) -> AppResult<PasteOutcome> {
    let Some(permit) = paste.try_begin() else {
        return Ok(PasteOutcome::AlreadyInProgress);
    };
    let settings = settings.get_stored()?;
    SystemClipboard::write(
        history.flavors(id)?,
        plain_text_only,
        settings.trim_whitespace,
    )?;
    if let Err(error) = history.mark_used(id, settings.move_used_to_top) {
        log::warn!("clipboard write succeeded but history usage update failed: {error}");
    }
    if let Err(error) = window::hide_panel(app, HideReason::Paste) {
        log::warn!("failed to hide panel before paste: {error}");
        return Ok(PasteOutcome::CopiedFocusFailed);
    }
    let outcome = paste.paste_to_target(permit);
    if outcome != PasteOutcome::Pasted {
        log::warn!("automatic paste degraded to clipboard-only: {outcome:?}");
    }
    Ok(outcome)
}
