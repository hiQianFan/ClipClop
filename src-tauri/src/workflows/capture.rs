use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::{
    clipboard,
    error::AppResult,
    history::{HistoryService, NewClip},
    preview::ExternalPreviewService,
    settings::SettingsService,
};

pub fn start(
    app: AppHandle,
    history: HistoryService,
    preview: ExternalPreviewService,
    settings: SettingsService,
) -> AppResult<()> {
    clipboard::start_watcher(move |snapshot| {
        capture(&app, &history, &preview, &settings, &snapshot)
    })
}

fn capture(
    app: &AppHandle,
    history: &HistoryService,
    preview: &ExternalPreviewService,
    settings: &SettingsService,
    snapshot: &NewClip,
) -> AppResult<()> {
    let policy = settings.get_stored()?;
    let id = history.capture(snapshot)?;
    let cleanup = crate::workflows::clip_actions::apply_retention(
        app,
        history,
        preview,
        policy.retention_days,
        policy.history_limit,
    );
    let _ = app.emit("history_changed", json!({ "latest_id": id }));
    cleanup?;
    Ok(())
}
