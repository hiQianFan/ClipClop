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
    let policy = settings.get_stored()?;
    let removed_before = crate::workflows::clip_actions::apply_retention(
        app,
        history,
        preview,
        policy.retention_days,
        policy.history_limit,
    )?;
    if let Some(id) = history.capture(snapshot)? {
        let cleanup = crate::workflows::clip_actions::apply_retention(
            app,
            history,
            preview,
            policy.retention_days,
            policy.history_limit,
        );
        let _ = app.emit("history_changed", json!({ "latest_id": id }));
        cleanup?;
    } else if removed_before > 0 {
        let _ = app.emit("history_changed", json!({ "latest_id": null }));
    }
    Ok(())
}
