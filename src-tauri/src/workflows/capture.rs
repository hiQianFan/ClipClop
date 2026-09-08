use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::{
    clipboard, error::AppResult, history::NewClip, settings::SettingsService, state::HistoryRuntime,
};

pub fn start(app: AppHandle, history: HistoryRuntime, settings: SettingsService) -> AppResult<()> {
    clipboard::start_watcher(move |snapshot| capture(&app, &history, &settings, &snapshot))
}

fn capture(
    app: &AppHandle,
    history: &HistoryRuntime,
    settings: &SettingsService,
    snapshot: &NewClip,
) -> AppResult<()> {
    let policy = settings.get_stored()?;
    let id = history.with_current(|environment| {
        let id = environment.history.capture(snapshot)?;
        crate::workflows::clip_actions::apply_retention(
            app,
            &environment.history,
            &environment.external_preview,
            policy.retention_days,
            policy.history_limit,
        )?;
        Ok(id)
    })?;
    let _ = app.emit("history_changed", json!({ "latest_id": id }));
    Ok(())
}
