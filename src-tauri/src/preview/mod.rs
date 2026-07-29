mod platform;

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::{
    error::{AppError, AppResult},
    history::{ClipDetail, ContentType, HistoryService},
    window::PreviewState,
};

#[derive(Clone)]
pub struct PreviewService {
    history: HistoryService,
}

#[derive(Serialize)]
pub struct PreviewResource {
    pub data_url: Option<String>,
    pub byte_size: Option<u64>,
}

impl PreviewService {
    pub fn new(history: HistoryService) -> Self {
        Self { history }
    }

    pub fn asset(&self, id: &str) -> AppResult<PreviewResource> {
        let detail = self.history.get_full(id)?;
        let flavors = self.history.flavors(id)?;
        Ok(PreviewResource {
            data_url: platform::preview_asset(
                &flavors,
                file_path(&detail).map(normalized_path).as_deref(),
            ),
            byte_size: None,
        })
    }

    pub fn file_asset(&self, id: &str, index: usize) -> AppResult<PreviewResource> {
        let detail = self.history.get_full(id)?;
        let flavors = self.history.flavors(id)?;
        let path = file_path_at(&detail, index).map(normalized_path);
        Ok(PreviewResource {
            data_url: platform::preview_asset(&flavors, path.as_deref()),
            byte_size: path
                .and_then(|path| std::fs::metadata(path).ok())
                .filter(|metadata| metadata.is_file())
                .map(|metadata| metadata.len()),
        })
    }

    pub fn thumbnail(&self, id: &str) -> AppResult<PreviewResource> {
        self.history.get_full(id)?;
        Ok(PreviewResource {
            data_url: platform::thumbnail_asset(&self.history.flavors(id)?),
            byte_size: None,
        })
    }

    pub fn source_app_icon(&self, app_id: &str) -> PreviewResource {
        PreviewResource {
            data_url: platform::source_app_icon(app_id),
            byte_size: None,
        }
    }

    pub fn open_clip(&self, app: &AppHandle, id: &str) -> AppResult<()> {
        let detail = self.history.get_full(id)?;
        match detail.summary.content_type {
            ContentType::File => self.open_file(app, &detail, 0),
            ContentType::Link => {
                let url = detail.plain_text.unwrap_or(detail.summary.preview);
                let parsed = url::Url::parse(&url)
                    .map_err(|_| AppError::Validation("link is not a valid URL".into()))?;
                if !matches!(parsed.scheme(), "http" | "https") {
                    return Err(AppError::Validation("link must use http or https".into()));
                }
                app.opener()
                    .open_url(url, None::<&str>)
                    .map_err(|error| AppError::Platform(error.to_string()))
            }
            ContentType::Image => {
                let flavors = self.history.flavors(id)?;
                let png = flavors
                    .iter()
                    .find(|flavor| flavor.format == "image/png")
                    .ok_or(AppError::NotFound)?;
                let path = preview_path(app, id, "png")?;
                write_preview(&path, &png.payload)?;
                open_path(app, path)
            }
            ContentType::Text | ContentType::Color | ContentType::Code => {
                let path = preview_path(app, id, "txt")?;
                write_preview(
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

    pub fn open_clip_file(&self, app: &AppHandle, id: &str, index: usize) -> AppResult<()> {
        let detail = self.history.get_full(id)?;
        if detail.summary.content_type != ContentType::File {
            return Err(AppError::Validation("clip is not a file record".into()));
        }
        self.open_file(app, &detail, index)
    }

    pub fn toggle(
        &self,
        app: &AppHandle,
        state: &PreviewState,
        id: &str,
        index: usize,
    ) -> AppResult<bool> {
        #[cfg(target_os = "macos")]
        let path = self.clip_preview_path(app, id, index)?;
        #[cfg(not(target_os = "macos"))]
        let path = PathBuf::new();
        platform::toggle_quicklook(app, state, &path)
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

    fn open_file(&self, app: &AppHandle, detail: &ClipDetail, index: usize) -> AppResult<()> {
        let path = file_path_at(detail, index)
            .map(normalized_path)
            .ok_or(AppError::NotFound)?;
        if !path.is_file() {
            return Err(AppError::NotFound);
        }
        open_path(app, path)
    }

    #[cfg(target_os = "macos")]
    fn clip_preview_path(&self, app: &AppHandle, id: &str, index: usize) -> AppResult<PathBuf> {
        let detail = self.history.get_full(id)?;
        match detail.summary.content_type {
            ContentType::File => {
                let path = file_path_at(&detail, index)
                    .map(normalized_path)
                    .ok_or(AppError::NotFound)?;
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
                write_preview(&path, &png.payload)?;
                Ok(path)
            }
            ContentType::Text | ContentType::Color | ContentType::Code | ContentType::Link => {
                let path = preview_path(app, id, "txt")?;
                write_preview(
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

fn open_path(app: &AppHandle, path: impl AsRef<Path>) -> AppResult<()> {
    app.opener()
        .open_path(path.as_ref().to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Platform(error.to_string()))
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

fn write_preview(path: &Path, bytes: &[u8]) -> AppResult<()> {
    std::fs::write(path, bytes).map_err(|error| AppError::Platform(error.to_string()))
}

fn file_path(detail: &ClipDetail) -> Option<&str> {
    file_path_at(detail, 0)
}

fn file_path_at(detail: &ClipDetail, index: usize) -> Option<&str> {
    detail.summary.metadata.files.get(index).map(String::as_str)
}

fn normalized_path(path: &str) -> PathBuf {
    url::Url::parse(path)
        .ok()
        .filter(|url| url.scheme() == "file")
        .and_then(|url| url.to_file_path().ok())
        .unwrap_or_else(|| PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};

    #[test]
    fn preview_derivation_and_cache_cleanup_preserve_shapes() {
        let png = STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
            .unwrap();
        let flavors = vec![crate::history::Flavor {
            format: "image/png".into(),
            payload: png,
        }];
        assert!(platform::preview_asset(&flavors, None)
            .unwrap()
            .starts_with("data:image/png;base64,"));
        assert!(platform::thumbnail_asset(&flavors)
            .unwrap()
            .starts_with("data:image/png;base64,"));
        assert!(platform::source_app_icon("/not/a/real/application").is_none());

        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("selected.png"), b"png").unwrap();
        std::fs::write(temp.path().join("selected.txt"), b"text").unwrap();
        std::fs::write(temp.path().join("other.png"), b"other").unwrap();

        delete_cached_previews_in(temp.path(), "selected").unwrap();

        assert!(!temp.path().join("selected.png").exists());
        assert!(!temp.path().join("selected.txt").exists());
        assert!(temp.path().join("other.png").exists());
    }
}
