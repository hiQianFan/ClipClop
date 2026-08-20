use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{async_runtime::JoinHandle, AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_updater::{Update, UpdaterExt};

const DOWNLOAD_EVENT: &str = "clipclop://update-download";
const RETRY_ATTEMPTS: u32 = 3;

#[derive(Default)]
pub struct UpdaterDownloadState(Mutex<DownloadState>);

#[derive(Default)]
struct DownloadState {
    generation: u64,
    task: Option<JoinHandle<()>>,
    request_id: Option<String>,
    downloaded: Option<DownloadedUpdate>,
}

struct DownloadedUpdate {
    version: String,
    update: Update,
    bytes: Vec<u8>,
}

fn invalidate(state: &mut DownloadState) -> u64 {
    state.generation += 1;
    state.generation
}

fn is_current(state: &DownloadState, generation: u64) -> bool {
    state.generation == generation
}

fn should_register_task(state: &DownloadState, generation: u64, request_id: &str) -> bool {
    is_current(state, generation) && state.request_id.as_deref() == Some(request_id)
}

fn is_transient_network_error(error: &str) -> bool {
    let error = error.to_lowercase();
    if error.contains("signature") || error.contains("verif") || error.contains("update_changed") {
        return false;
    }
    [
        "error sending request",
        "error decoding response body",
        "timed out",
        "timeout",
        "connection",
        "connreset",
        "reset by peer",
        "network",
        "dns",
        "eof",
        "os error",
        "tls",
        "handshake",
    ]
    .iter()
    .any(|signature| error.contains(signature))
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadEvent {
    request_id: String,
    kind: &'static str,
    percent: Option<u8>,
    error: Option<String>,
}

fn emit<R: Runtime>(
    app: &AppHandle<R>,
    request_id: &str,
    kind: &'static str,
    percent: Option<u8>,
    error: Option<String>,
) {
    let _ = app.emit(
        DOWNLOAD_EVENT,
        DownloadEvent {
            request_id: request_id.to_owned(),
            kind,
            percent,
            error,
        },
    );
}

#[tauri::command]
pub async fn start_update_download<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, UpdaterDownloadState>,
    expected_version: String,
    request_id: String,
) -> Result<(), String> {
    let generation = {
        let mut state = state.0.lock().map_err(|_| "updater state lock poisoned")?;
        if let Some(task) = state.task.take() {
            task.abort();
        }
        if let Some(old_request_id) = state.request_id.take() {
            emit(
                &app,
                &old_request_id,
                "error",
                None,
                Some("UPDATE_CANCELLED".to_owned()),
            );
        }
        state.downloaded = None;
        state.request_id = Some(request_id.clone());
        invalidate(&mut state)
    };

    let app_for_task = app.clone();
    let task_request_id = request_id.clone();
    let task = tauri::async_runtime::spawn(async move {
        let state_for_task = app_for_task.state::<UpdaterDownloadState>();
        let result = async {
            let mut completed = None;
            for attempt in 1..=RETRY_ATTEMPTS {
                emit(&app_for_task, &task_request_id, "progress", None, None);
                let attempt_result = async {
                    let update = app_for_task
                        .updater_builder()
                        .timeout(Duration::from_secs(120))
                        .build()
                        .map_err(|error| error.to_string())?
                        .check()
                        .await
                        .map_err(|error| error.to_string())?
                        .ok_or_else(|| "UPDATE_CHANGED".to_owned())?;
                    if update.version != expected_version {
                        return Err("UPDATE_CHANGED".to_owned());
                    }

                    let mut downloaded = 0_u64;
                    let bytes = update
                        .download(
                            |chunk, total| {
                                downloaded += chunk as u64;
                                let percent = total.filter(|total| *total > 0).map(|total| {
                                    ((downloaded.saturating_mul(100) / total).min(99)) as u8
                                });
                                emit(&app_for_task, &task_request_id, "progress", percent, None);
                            },
                            || {},
                        )
                        .await
                        .map_err(|error| error.to_string())?;
                    Ok::<_, String>((update, bytes))
                }
                .await;

                match attempt_result {
                    Ok(value) => {
                        completed = Some(value);
                        break;
                    }
                    Err(error)
                        if attempt < RETRY_ATTEMPTS && is_transient_network_error(&error) =>
                    {
                        tauri::async_runtime::spawn_blocking(move || {
                            std::thread::sleep(Duration::from_millis(1500 * attempt as u64));
                        })
                        .await
                        .map_err(|error| error.to_string())?;
                    }
                    Err(error) => return Err(error),
                }
            }
            let (update, bytes) = completed.ok_or_else(|| "Update download failed".to_owned())?;

            let mut state = state_for_task
                .0
                .lock()
                .map_err(|_| "updater state lock poisoned")?;
            if !is_current(&state, generation) {
                return Ok(());
            }
            state.downloaded = Some(DownloadedUpdate {
                version: expected_version,
                update,
                bytes,
            });
            emit(&app_for_task, &task_request_id, "finished", Some(100), None);
            state.task = None;
            state.request_id = None;
            Ok(())
        }
        .await;

        if let Err(error) = result {
            if let Ok(mut state) = state_for_task.0.lock() {
                if is_current(&state, generation) {
                    state.task = None;
                    state.request_id = None;
                    state.downloaded = None;
                    drop(state);
                    emit(&app_for_task, &task_request_id, "error", None, Some(error));
                }
            }
        }
    });

    let mut state = state.0.lock().map_err(|_| "updater state lock poisoned")?;
    if should_register_task(&state, generation, &request_id) {
        state.task = Some(task);
    } else {
        task.abort();
    }
    Ok(())
}

#[tauri::command]
pub fn cancel_update_download<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, UpdaterDownloadState>,
) -> Result<(), String> {
    let mut state = state.0.lock().map_err(|_| "updater state lock poisoned")?;
    if state.request_id.is_none() {
        return Ok(());
    }
    invalidate(&mut state);
    if let Some(task) = state.task.take() {
        task.abort();
    }
    if let Some(request_id) = state.request_id.take() {
        emit(
            &app,
            &request_id,
            "error",
            None,
            Some("UPDATE_CANCELLED".to_owned()),
        );
    }
    state.downloaded = None;
    Ok(())
}

#[tauri::command]
pub fn discard_downloaded_update(state: State<'_, UpdaterDownloadState>) -> Result<(), String> {
    state
        .0
        .lock()
        .map_err(|_| "updater state lock poisoned")?
        .downloaded = None;
    Ok(())
}

#[tauri::command]
pub fn install_downloaded_update(
    state: State<'_, UpdaterDownloadState>,
    expected_version: String,
) -> Result<(), String> {
    let (update, bytes) = {
        let state = state.0.lock().map_err(|_| "updater state lock poisoned")?;
        let downloaded = state
            .downloaded
            .as_ref()
            .ok_or_else(|| "UPDATE_NOT_DOWNLOADED".to_owned())?;
        if downloaded.version != expected_version {
            return Err("UPDATE_CHANGED".to_owned());
        }
        (downloaded.update.clone(), downloaded.bytes.clone())
    };
    update.install(&bytes).map_err(|error| error.to_string())?;
    let mut state = state.0.lock().map_err(|_| "updater state lock poisoned")?;
    if state
        .downloaded
        .as_ref()
        .is_some_and(|downloaded| downloaded.version == expected_version)
    {
        state.downloaded = None;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        invalidate, is_current, is_transient_network_error, should_register_task, DownloadState,
    };

    #[test]
    fn generation_invalidates_cancelled_download_completion() {
        let mut state = DownloadState::default();
        let cancelled = invalidate(&mut state);
        assert!(is_current(&state, cancelled));
        invalidate(&mut state);
        assert!(!is_current(&state, cancelled));
    }

    #[test]
    fn retries_only_transient_network_failures() {
        assert!(is_transient_network_error("error decoding response body"));
        assert!(is_transient_network_error("connection reset by peer"));
        assert!(!is_transient_network_error("signature verification failed"));
        assert!(!is_transient_network_error("UPDATE_CHANGED"));
        assert!(!is_transient_network_error("permission denied"));
    }

    #[test]
    fn cancelled_spawn_gap_never_registers_the_task() {
        let mut state = DownloadState {
            request_id: Some("request".into()),
            ..Default::default()
        };
        let generation = invalidate(&mut state);
        assert!(should_register_task(&state, generation, "request"));
        state.request_id = None;
        invalidate(&mut state);
        assert!(!should_register_task(&state, generation, "request"));
    }
}
