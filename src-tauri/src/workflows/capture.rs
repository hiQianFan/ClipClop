use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::{
    clipboard,
    error::AppResult,
    history::{HistoryService, NewClip},
    preview::PreviewService,
    settings::SettingsService,
};

pub fn start(
    app: AppHandle,
    history: HistoryService,
    preview: PreviewService,
    settings: SettingsService,
) -> AppResult<()> {
    clipboard::start_watcher(move |snapshot| {
        capture(&app, &history, &preview, &settings, &snapshot)
    })
}

fn capture(
    app: &AppHandle,
    history: &HistoryService,
    preview: &PreviewService,
    settings: &SettingsService,
    snapshot: &NewClip,
) -> AppResult<()> {
    let retention_days = settings.get_stored()?.retention_days;
    let cutoff = HistoryService::retention_cutoff(retention_days);
    let expired = history.expired_ids_before(cutoff)?;
    if !expired.is_empty() {
        let _guard = preview.lock_lifecycle()?;
        for id in expired {
            preview.delete_cached(app, &id)?;
        }
        history.prune_before(cutoff)?;
    }
    if let Some(id) = history.capture(snapshot)? {
        let _ = app.emit("history_changed", json!({ "latest_id": id }));
    }
    Ok(())
}
