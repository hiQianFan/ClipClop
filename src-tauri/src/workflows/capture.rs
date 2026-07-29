use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::{
    clipboard,
    error::AppResult,
    history::{HistoryService, NewClip},
    settings::SettingsService,
};

pub fn start(app: AppHandle, history: HistoryService, settings: SettingsService) -> AppResult<()> {
    clipboard::start_watcher(move |snapshot| capture(&app, &history, &settings, &snapshot))
}

fn capture(
    app: &AppHandle,
    history: &HistoryService,
    settings: &SettingsService,
    snapshot: &NewClip,
) -> AppResult<()> {
    history.prune(settings.get_stored()?.retention_days)?;
    if let Some(id) = history.capture(snapshot)? {
        let _ = app.emit("history_changed", json!({ "latest_id": id }));
    }
    Ok(())
}
