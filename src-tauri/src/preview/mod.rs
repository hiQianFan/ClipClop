mod platform;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::{
    error::{AppError, AppResult},
    history::{normalized_file_path, ContentType, HistoryService},
    onboarding::OnboardingExample,
    window::PreviewState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewProvider {
    MacosQuicklook,
    Quicklook,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewUnavailableReason {
    NotInstalled,
    UnsupportedInstall,
    Elevated,
    DetectionFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PreviewCapability {
    pub provider: PreviewProvider,
    pub reason: Option<PreviewUnavailableReason>,
    pub version: Option<String>,
}

impl PreviewCapability {
    fn ready(provider: PreviewProvider) -> Self {
        Self {
            provider,
            reason: None,
            version: None,
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn unavailable(reason: PreviewUnavailableReason) -> Self {
        Self {
            provider: PreviewProvider::Unavailable,
            reason: Some(reason),
            version: None,
        }
    }

    #[cfg(target_os = "windows")]
    fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
}

pub fn capability() -> PreviewCapability {
    platform::capability()
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
const ONBOARDING_LOGO: &[u8] = include_bytes!("../../../static/app-icon.png");
#[cfg(any(target_os = "macos", target_os = "windows", test))]
const ONBOARDING_TEXT: &[u8] = b"ClipClop";
#[cfg(any(target_os = "macos", target_os = "windows", test))]
const ONBOARDING_LINK: &[u8] = b"https://github.com/hiQianFan/ClipClop";

#[derive(Clone)]
pub struct ExternalPreviewService {
    history: HistoryService,
    lifecycle: Arc<Mutex<()>>,
}

impl ExternalPreviewService {
    pub fn new(history: HistoryService) -> Self {
        Self {
            history,
            lifecycle: Arc::new(Mutex::new(())),
        }
    }

    pub fn open_clip(&self, app: &AppHandle, id: &str) -> AppResult<()> {
        let detail = self.history.get_full(id)?;
        match detail.summary.content_type {
            ContentType::File => open_file(app, &detail, 0),
            ContentType::Image => {
                let flavors = self.history.flavors(id)?;
                let png = flavors
                    .iter()
                    .find(|flavor| flavor.format == "image/png")
                    .ok_or(AppError::NotFound)?;
                let path = preview_path(app, id, "png")?;
                self.publish_preview(id, &path, &png.payload)?;
                open_path(app, path)
            }
            ContentType::Text | ContentType::Color | ContentType::Link => {
                let path = preview_path(app, id, "txt")?;
                self.publish_preview(
                    id,
                    &path,
                    detail
                        .plain_text
                        .unwrap_or(detail.summary.preview)
                        .as_bytes(),
                )?;
                open_path(app, path)
            }
        }
    }

    pub fn open_link(&self, app: &AppHandle, id: &str, origin_only: bool) -> AppResult<()> {
        let detail = self.history.get_full(id)?;
        if detail.summary.content_type != ContentType::Link {
            return Err(AppError::Validation("clip is not a link record".into()));
        }
        let parsed = web_url(&detail.plain_text.unwrap_or(detail.summary.preview))?;
        let target = if origin_only {
            parsed.origin().ascii_serialization()
        } else {
            parsed.as_str().to_owned()
        };
        app.opener()
            .open_url(target, None::<&str>)
            .map_err(|error| AppError::Platform(error.to_string()))
    }

    pub fn open_clip_file(&self, app: &AppHandle, id: &str, index: usize) -> AppResult<()> {
        let detail = self.history.get_full(id)?;
        if detail.summary.content_type != ContentType::File {
            return Err(AppError::Validation("clip is not a file record".into()));
        }
        open_file(app, &detail, index)
    }

    pub fn toggle(
        &self,
        app: &AppHandle,
        state: &PreviewState,
        id: &str,
        index: usize,
    ) -> AppResult<bool> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let path = self.clip_preview_path(app, id, index)?;
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let path = {
            let _ = (id, index);
            PathBuf::new()
        };
        platform::toggle_quicklook(app, state, &path)
    }

    pub fn close_native(&self, app: &AppHandle, state: &PreviewState) -> AppResult<()> {
        platform::close_quicklook(app, state)
    }

    /// Write and preview one of the fixed onboarding resources.
    pub fn toggle_onboarding_example(
        &self,
        app: &AppHandle,
        state: &PreviewState,
        example: OnboardingExample,
    ) -> AppResult<bool> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            let (name, extension, bytes) = onboarding_preview(example);
            let path = preview_path(app, name, extension)?;
            let dir = path.parent().ok_or(AppError::NotFound)?;
            std::fs::create_dir_all(dir).map_err(|error| AppError::Platform(error.to_string()))?;
            std::fs::write(&path, bytes).map_err(|error| AppError::Platform(error.to_string()))?;
            platform::toggle_quicklook(app, state, &path)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = (app, example);
            platform::toggle_quicklook(app, state, &PathBuf::new())
        }
    }

    pub fn delete_cached(&self, app: &AppHandle, id: &str) -> AppResult<()> {
        delete_cached_previews_in(&cached_preview_dir(app)?, id)
    }

    pub fn clear_cached(&self, app: &AppHandle) -> AppResult<()> {
        match std::fs::remove_dir_all(cached_preview_dir(app)?) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(AppError::Platform(error.to_string())),
        }
    }

    pub fn lock_lifecycle(&self) -> AppResult<MutexGuard<'_, ()>> {
        self.lifecycle
            .lock()
            .map_err(|_| AppError::Platform("preview lifecycle lock poisoned".into()))
    }

    fn publish_preview(&self, id: &str, path: &Path, bytes: &[u8]) -> AppResult<()> {
        let temporary = write_preview_temp(path, bytes)?;
        let _guard = self.lock_lifecycle()?;
        if let Err(error) = self.history.get_full(id) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        if path.is_file() {
            let _ = std::fs::remove_file(&temporary);
            return Ok(());
        }
        if let Err(error) = std::fs::rename(&temporary, path) {
            let _ = std::fs::remove_file(temporary);
            return Err(AppError::Platform(error.to_string()));
        }
        Ok(())
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn clip_preview_path(&self, app: &AppHandle, id: &str, index: usize) -> AppResult<PathBuf> {
        let detail = self.history.get_full(id)?;
        match detail.summary.content_type {
            ContentType::File => {
                let path = normalized_file_path(&detail, index).ok_or(AppError::NotFound)?;
                if !path.is_file() {
                    return Err(AppError::NotFound);
                }
                Ok(path)
            }
            ContentType::Image => {
                let flavors = self.history.flavors(id)?;
                let png = flavors
                    .iter()
                    .find(|flavor| flavor.format == "image/png")
                    .ok_or(AppError::NotFound)?;
                let path = preview_path(app, id, "png")?;
                self.publish_preview(id, &path, &png.payload)?;
                Ok(path)
            }
            ContentType::Text | ContentType::Color | ContentType::Link => {
                let path = preview_path(app, id, "txt")?;
                self.publish_preview(
                    id,
                    &path,
                    detail
                        .plain_text
                        .unwrap_or(detail.summary.preview)
                        .as_bytes(),
                )?;
                Ok(path)
            }
        }
    }
}

fn web_url(value: &str) -> AppResult<url::Url> {
    let parsed = url::Url::parse(value.trim())
        .map_err(|_| AppError::Validation("link is not a valid URL".into()))?;
    if matches!(parsed.scheme(), "http" | "https") {
        Ok(parsed)
    } else {
        Err(AppError::Validation("link must use http or https".into()))
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn onboarding_preview(example: OnboardingExample) -> (&'static str, &'static str, &'static [u8]) {
    match example {
        OnboardingExample::Image => ("onboarding-image", "png", ONBOARDING_LOGO),
        OnboardingExample::Text => ("onboarding-text", "txt", ONBOARDING_TEXT),
        OnboardingExample::Link => ("onboarding-link", "txt", ONBOARDING_LINK),
    }
}

fn open_path(app: &AppHandle, path: impl AsRef<Path>) -> AppResult<()> {
    app.opener()
        .open_path(path.as_ref().to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Platform(error.to_string()))
}

fn open_file(app: &AppHandle, detail: &crate::history::ClipDetail, index: usize) -> AppResult<()> {
    let path = normalized_file_path(detail, index).ok_or(AppError::NotFound)?;
    if !path.is_file() {
        return Err(AppError::NotFound);
    }
    open_path(app, path)
}

fn preview_path(app: &AppHandle, id: &str, extension: &str) -> AppResult<PathBuf> {
    let dir = cached_preview_dir(app)?;
    std::fs::create_dir_all(&dir).map_err(|error| AppError::Platform(error.to_string()))?;
    Ok(dir.join(format!("{id}.{extension}")))
}

fn cached_preview_dir(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(app
        .path()
        .app_cache_dir()
        .map_err(|error| AppError::Platform(error.to_string()))?
        .join("external-preview"))
}

fn delete_cached_previews_in(dir: &Path, id: &str) -> AppResult<()> {
    for extension in ["png", "txt"] {
        match std::fs::remove_file(dir.join(format!("{id}.{extension}"))) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppError::Platform(error.to_string())),
        }
    }
    Ok(())
}

fn write_preview_temp(path: &Path, bytes: &[u8]) -> AppResult<PathBuf> {
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::now_v7()));
    std::fs::write(&temporary, bytes).map_err(|error| AppError::Platform(error.to_string()))?;
    Ok(temporary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_links_are_trimmed_and_restricted_to_browsers() {
        assert_eq!(
            web_url("  https://example.com/path  ").unwrap().as_str(),
            "https://example.com/path"
        );
        assert!(web_url("file:///tmp/private").is_err());
        assert!(web_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn browser_link_origins_drop_credentials_paths_queries_and_fragments() {
        let parsed = web_url("https://user:secret@example.com:8443/path?q=1#part").unwrap();
        assert_eq!(
            parsed.origin().ascii_serialization(),
            "https://example.com:8443"
        );
    }
    use chrono::Utc;
    use std::sync::Arc;

    #[test]
    fn onboarding_examples_are_fixed_image_text_and_link_resources() {
        assert_eq!(onboarding_preview(OnboardingExample::Image).1, "png");
        assert_eq!(onboarding_preview(OnboardingExample::Text).2, b"ClipClop");
        assert_eq!(
            onboarding_preview(OnboardingExample::Link).2,
            b"https://github.com/hiQianFan/ClipClop"
        );
    }

    #[test]
    fn cache_cleanup_preserves_unrelated_previews() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("selected.png"), b"png").unwrap();
        std::fs::write(temp.path().join("selected.txt"), b"text").unwrap();
        std::fs::write(temp.path().join("other.png"), b"other").unwrap();

        delete_cached_previews_in(temp.path(), "selected").unwrap();

        assert!(!temp.path().join("selected.png").exists());
        assert!(!temp.path().join("selected.txt").exists());
        assert!(temp.path().join("other.png").exists());
    }

    #[test]
    fn generation_cannot_publish_after_delete_or_clear_wins_the_lifecycle_guard() {
        for clear in [false, true] {
            let history =
                HistoryService::new(Arc::new(crate::storage::Database::in_memory().unwrap()));
            let preview = ExternalPreviewService::new(history.clone());
            let id = history
                .capture(&crate::history::NewClip {
                    content_type: ContentType::Text,
                    plain_text: Some("text".into()),
                    preview: "text".into(),
                    source_app: None,
                    flavors: vec![],
                    metadata: Default::default(),
                    content_hash: format!("race-{clear}"),
                    created_at: Utc::now(),
                })
                .unwrap();
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join(format!("{id}.txt"));

            let guard = preview.lock_lifecycle().unwrap();
            let worker = preview.clone();
            let worker_id = id.clone();
            let worker_path = path.clone();
            let generation =
                std::thread::spawn(move || worker.publish_preview(&worker_id, &worker_path, b"x"));
            if clear {
                history.clear().unwrap();
            } else {
                history.delete(&id).unwrap();
            }
            drop(guard);

            assert!(matches!(
                generation.join().unwrap(),
                Err(AppError::NotFound)
            ));
            assert!(!path.exists());
        }
    }
}
