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
    window_label: &str,
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
    if !paste.can_inject() {
        return Ok(PasteOutcome::CopiedPermissionRequired);
    }
    if !paste.is_current(&permit) {
        return Ok(PasteOutcome::CopiedFocusFailed);
    }
    #[cfg(target_os = "macos")]
    let hidden = {
        let paste = paste.clone();
        let session = permit.session;
        let label = window_label.to_string();
        window::on_main(app, move |app| {
            if !paste.session_is_current(session) {
                return Err(tauri::Error::FailedToReceiveMessage);
            }
            window::hide_panel(app, &label, HideReason::Paste)
        })
        .and_then(|result| result)
    };
    #[cfg(not(target_os = "macos"))]
    let hidden = window::hide_panel(app, window_label, HideReason::Paste);
    if let Err(error) = hidden {
        log::warn!("failed to hide panel before paste: {error}");
        return Ok(PasteOutcome::CopiedFocusFailed);
    }
    let outcome = paste.paste_to_target(app, permit);
    if outcome != PasteOutcome::Pasted {
        log::warn!("automatic paste degraded to clipboard-only: {outcome:?}");
    }
    Ok(outcome)
}
