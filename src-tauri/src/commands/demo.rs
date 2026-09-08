use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::{demo, error::AppResult, state::AppState, window::PreviewState};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    Real,
    Demo,
}

#[tauri::command]
pub fn get_runtime_mode(state: State<'_, AppState>) -> AppResult<RuntimeMode> {
    Ok(if state.history.is_demo()? {
        RuntimeMode::Demo
    } else {
        RuntimeMode::Real
    })
}

#[tauri::command]
pub fn enter_demo_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    preview_state: State<'_, PreviewState>,
) -> AppResult<RuntimeMode> {
    let language = state.settings.get_stored()?.language;
    state.history.reset_demo(
        |environment| demo::cleanup_environment(&app, environment),
        || demo::create_environment(&app, language),
        |environment| {
            if preview_state.is_active() {
                environment
                    .external_preview
                    .close_native(&app, &preview_state)?;
            }
            Ok(())
        },
    )?;
    notify(&app, RuntimeMode::Demo);
    Ok(RuntimeMode::Demo)
}

#[tauri::command]
pub fn exit_demo_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    preview_state: State<'_, PreviewState>,
) -> AppResult<RuntimeMode> {
    let changed = state.history.exit_demo(|environment| {
        if preview_state.is_active() {
            environment
                .external_preview
                .close_native(&app, &preview_state)?;
        }
        demo::cleanup_environment(&app, environment)
    })?;
    if changed {
        notify(&app, RuntimeMode::Real);
    }
    Ok(RuntimeMode::Real)
}

fn notify(app: &AppHandle, mode: RuntimeMode) {
    let _ = app.emit("runtime_mode_changed", mode);
    let _ = app.emit("history_changed", serde_json::json!({ "latest_id": null }));
}
