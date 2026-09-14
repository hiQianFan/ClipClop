use serde::Serialize;
use tauri::Manager;

use crate::error::AppError;

#[derive(Clone, Serialize)]
pub struct StartupFailure {
    kind: &'static str,
    app_version: String,
    database_version: Option<u32>,
    required_version: Option<u32>,
}

impl StartupFailure {
    fn new(error: &AppError, app_version: String) -> Self {
        let (kind, database_version, required_version) = match error {
            AppError::DatabaseTooNew { version, required } => {
                ("too_new", Some(*version), Some(*required))
            }
            AppError::DatabaseInUse => ("in_use", None, None),
            _ => ("storage", None, None),
        };
        Self {
            kind,
            app_version,
            database_version,
            required_version,
        }
    }
}

#[tauri::command]
pub fn get_startup_failure(app: tauri::AppHandle) -> Option<StartupFailure> {
    app.try_state::<StartupFailure>()
        .map(|failure| failure.inner().clone())
}

pub fn show_failure(app: &mut tauri::App, error: AppError) {
    log::error!("database startup failed: {error}");
    app.manage(StartupFailure::new(
        &error,
        app.package_info().version.to_string(),
    ));
    // The root layout renders a recovery page without mounting history/settings.
    // Returning Ok from setup is essential: Tauri aborts on a setup error on macOS.
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular);
    if let Some(window) = app.get_webview_window("main") {
        for result in [
            window.set_decorations(true),
            window.show(),
            window.set_focus(),
        ] {
            if let Err(error) = result {
                log::error!("failed to show startup error window: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_failure_exposes_versions_but_not_database_diagnostics() {
        let failure = StartupFailure::new(
            &AppError::DatabaseTooNew {
                version: 12,
                required: 11,
            },
            "0.11.1".into(),
        );
        let value = serde_json::to_value(failure).unwrap();
        assert_eq!(value["kind"], "too_new");
        assert_eq!(value["database_version"], 12);
        assert_eq!(value["required_version"], 11);
        let failure = StartupFailure::new(
            &AppError::Storage("private diagnostic".into()),
            "0.11.1".into(),
        );
        assert!(!serde_json::to_string(&failure)
            .unwrap()
            .contains("private diagnostic"));
    }
}
