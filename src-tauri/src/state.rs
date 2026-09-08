use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    assets::AssetService,
    error::{AppError, AppResult},
    history::HistoryService,
    onboarding::OnboardingService,
    paste::PasteController,
    preview::ExternalPreviewService,
    settings::SettingsService,
    storage::Database,
};

#[derive(Clone)]
pub struct HistoryEnvironment {
    pub history: HistoryService,
    pub assets: AssetService,
    pub external_preview: ExternalPreviewService,
    pub demo_directory: Option<PathBuf>,
}

impl HistoryEnvironment {
    pub(crate) fn new(database: Arc<Database>, cache_namespace: &'static str) -> Self {
        let history = HistoryService::new(database);
        Self {
            assets: AssetService::new(history.clone()),
            external_preview: ExternalPreviewService::with_cache_namespace(
                history.clone(),
                cache_namespace,
            ),
            history,
            demo_directory: None,
        }
    }

    pub(crate) fn with_demo_directory(mut self, directory: PathBuf) -> Self {
        self.demo_directory = Some(directory);
        self
    }
}

struct HistoryRuntimeState {
    real: HistoryEnvironment,
    demo: Option<HistoryEnvironment>,
}

#[derive(Clone)]
pub struct HistoryRuntime {
    state: Arc<Mutex<HistoryRuntimeState>>,
}

impl HistoryRuntime {
    fn new(database: Arc<Database>) -> Self {
        Self {
            state: Arc::new(Mutex::new(HistoryRuntimeState {
                real: HistoryEnvironment::new(database, "external-preview"),
                demo: None,
            })),
        }
    }

    pub fn with_current<T>(
        &self,
        operation: impl FnOnce(&HistoryEnvironment) -> AppResult<T>,
    ) -> AppResult<T> {
        let state = self.lock()?;
        operation(state.demo.as_ref().unwrap_or(&state.real))
    }

    pub fn is_demo(&self) -> AppResult<bool> {
        Ok(self.lock()?.demo.is_some())
    }

    pub fn replace_demo(
        &self,
        environment: HistoryEnvironment,
        mut cleanup: impl FnMut(&HistoryEnvironment) -> AppResult<()>,
    ) -> AppResult<()> {
        let mut state = self.lock()?;
        if let Some(current) = state.demo.as_ref() {
            cleanup(current)?;
        }
        state.demo = Some(environment);
        Ok(())
    }

    pub fn reset_demo(
        &self,
        mut cleanup: impl FnMut(&HistoryEnvironment) -> AppResult<()>,
        build: impl FnOnce() -> AppResult<HistoryEnvironment>,
        before_swap: impl FnOnce(&HistoryEnvironment) -> AppResult<()>,
    ) -> AppResult<()> {
        let replacement = build()?;
        let mut state = self.lock()?;
        if let Err(error) = before_swap(state.demo.as_ref().unwrap_or(&state.real)) {
            let _ = cleanup(&replacement);
            return Err(error);
        }
        if let Some(current) = state.demo.as_ref() {
            if let Err(error) = cleanup(current) {
                let _ = cleanup(&replacement);
                return Err(error);
            }
        }
        state.demo = Some(replacement);
        Ok(())
    }

    pub fn exit_demo(
        &self,
        cleanup: impl FnOnce(&HistoryEnvironment) -> AppResult<()>,
    ) -> AppResult<bool> {
        let mut state = self.lock()?;
        let Some(current) = state.demo.as_ref() else {
            return Ok(false);
        };
        cleanup(current)?;
        state.demo = None;
        Ok(true)
    }

    fn lock(&self) -> AppResult<std::sync::MutexGuard<'_, HistoryRuntimeState>> {
        self.state
            .lock()
            .map_err(|_| AppError::Storage("history runtime lock poisoned".into()))
    }
}

pub struct AppState {
    pub history: HistoryRuntime,
    pub paste: PasteController,
    pub settings: SettingsService,
    pub onboarding: OnboardingService,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let database = Arc::new(database);
        Self {
            history: HistoryRuntime::new(database.clone()),
            paste: PasteController::default(),
            settings: SettingsService::new(database.clone()),
            onboarding: OnboardingService::new(database),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{ClipMetadata, ContentType, Flavor, NewClip};
    use chrono::Utc;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    };

    fn sample(text: &str) -> NewClip {
        NewClip {
            content_type: ContentType::Text,
            plain_text: Some(text.into()),
            preview: text.into(),
            source_app: None,
            flavors: vec![Flavor {
                format: "text/plain".into(),
                payload: text.as_bytes().to_vec(),
            }],
            metadata: ClipMetadata::default(),
            content_hash: text.into(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn demo_environment_is_isolated_and_resettable() {
        let runtime = HistoryRuntime::new(Arc::new(Database::in_memory().unwrap()));
        runtime
            .with_current(|environment| environment.history.capture(&sample("real")))
            .unwrap();
        let demo = HistoryEnvironment::new(
            Arc::new(Database::in_memory().unwrap()),
            "demo-external-preview",
        );
        demo.history.capture(&sample("demo")).unwrap();
        runtime.replace_demo(demo, |_| Ok(())).unwrap();
        assert_eq!(
            runtime
                .with_current(|environment| environment.history.query(&Default::default()))
                .unwrap()
                .items[0]
                .preview,
            "demo"
        );
        runtime.exit_demo(|_| Ok(())).unwrap();
        assert_eq!(
            runtime
                .with_current(|environment| environment.history.query(&Default::default()))
                .unwrap()
                .items[0]
                .preview,
            "real"
        );
    }

    #[test]
    fn failed_cleanup_keeps_demo_available_for_retry() {
        let runtime = HistoryRuntime::new(Arc::new(Database::in_memory().unwrap()));
        runtime
            .replace_demo(
                HistoryEnvironment::new(Arc::new(Database::in_memory().unwrap()), "demo-test"),
                |_| Ok(()),
            )
            .unwrap();
        assert!(runtime
            .exit_demo(|_| Err(AppError::Platform("busy".into())))
            .is_err());
        assert!(runtime.is_demo().unwrap());
        assert!(runtime.exit_demo(|_| Ok(())).unwrap());
        assert!(!runtime.is_demo().unwrap());
    }

    #[test]
    fn exit_waits_for_an_active_operation() {
        let runtime = HistoryRuntime::new(Arc::new(Database::in_memory().unwrap()));
        runtime
            .replace_demo(
                HistoryEnvironment::new(Arc::new(Database::in_memory().unwrap()), "demo-test"),
                |_| Ok(()),
            )
            .unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let active = runtime.clone();
        let operation = std::thread::spawn(move || {
            active
                .with_current(|_| {
                    entered_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    Ok(())
                })
                .unwrap()
        });
        entered_rx.recv().unwrap();
        let cleaned = Arc::new(AtomicBool::new(false));
        let exit_runtime = runtime.clone();
        let exit_cleaned = cleaned.clone();
        let exit = std::thread::spawn(move || {
            exit_runtime
                .exit_demo(|_| {
                    exit_cleaned.store(true, Ordering::SeqCst);
                    Ok(())
                })
                .unwrap()
        });
        std::thread::yield_now();
        assert!(!cleaned.load(Ordering::SeqCst));
        release_tx.send(()).unwrap();
        operation.join().unwrap();
        assert!(exit.join().unwrap());
        assert!(cleaned.load(Ordering::SeqCst));
    }

    #[test]
    fn failed_rebuild_preserves_the_current_demo() {
        let runtime = HistoryRuntime::new(Arc::new(Database::in_memory().unwrap()));
        let demo = HistoryEnvironment::new(Arc::new(Database::in_memory().unwrap()), "demo-test");
        demo.history.capture(&sample("current")).unwrap();
        runtime.replace_demo(demo, |_| Ok(())).unwrap();
        assert!(runtime
            .reset_demo(
                |_| Ok(()),
                || Err(AppError::Platform("build failed".into())),
                |_| Ok(())
            )
            .is_err());
        assert_eq!(
            runtime
                .with_current(|environment| environment.history.query(&Default::default()))
                .unwrap()
                .items[0]
                .preview,
            "current"
        );
    }
}
