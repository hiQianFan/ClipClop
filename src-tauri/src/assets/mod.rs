mod platform;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::Serialize;

use crate::{
    error::{AppError, AppResult},
    history::{normalized_file_path, HistoryService},
};

#[derive(Clone)]
pub struct AssetService {
    history: HistoryService,
    icon_cache: Arc<Mutex<HashMap<String, Option<String>>>>,
}

#[derive(Serialize)]
pub struct PreviewResource {
    pub data_url: Option<String>,
    pub byte_size: Option<u64>,
    pub access_denied: bool,
    pub is_directory: bool,
}

impl AssetService {
    pub fn new(history: HistoryService) -> Self {
        Self {
            history,
            icon_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn asset(&self, id: &str) -> AppResult<PreviewResource> {
        let detail = self.history.get_full(id)?;
        let flavors = self.history.flavors(id)?;
        Ok(PreviewResource {
            data_url: platform::preview_asset(
                &flavors,
                normalized_file_path(&detail, 0).as_deref(),
            ),
            byte_size: None,
            access_denied: false,
            is_directory: false,
        })
    }

    pub fn file_asset(&self, id: &str, index: usize) -> AppResult<PreviewResource> {
        let detail = self.history.get_full(id)?;
        if detail.summary.content_type != crate::history::ContentType::File {
            return Err(AppError::Validation("clip is not a file".into()));
        }
        let flavors = self.history.flavors(id)?;
        let path = normalized_file_path(&detail, index);
        let access_denied = file_access_denied(path.as_deref());
        let is_directory = path.as_deref().is_some_and(|path| path.is_dir());
        Ok(PreviewResource {
            data_url: platform::preview_asset(&flavors, path.as_deref()),
            byte_size: path
                .and_then(|path| std::fs::metadata(path).ok())
                .filter(|metadata| metadata.is_file())
                .map(|metadata| metadata.len()),
            access_denied,
            is_directory,
        })
    }

    pub fn thumbnail(&self, id: &str) -> AppResult<PreviewResource> {
        self.history.get_full(id)?;
        Ok(PreviewResource {
            data_url: platform::thumbnail_asset(&self.history.flavors(id)?),
            byte_size: None,
            access_denied: false,
            is_directory: false,
        })
    }

    pub fn source_app_icon(&self, id: &str) -> AppResult<PreviewResource> {
        let detail = self.history.get_full(id)?;
        let Some(source) = detail.summary.source_app else {
            return Ok(PreviewResource {
                data_url: None,
                byte_size: None,
                access_denied: false,
                is_directory: false,
            });
        };
        let mut cache = self
            .icon_cache
            .lock()
            .map_err(|_| AppError::Platform("asset icon cache lock poisoned".into()))?;
        let data_url = cache
            .entry(source.id.clone())
            .or_insert_with(|| {
                let icon = platform::source_app_icon(&source.id);
                if icon.is_none() {
                    log::warn!("source icon unavailable for {}", source.id);
                }
                icon
            })
            .clone();
        Ok(PreviewResource {
            data_url,
            byte_size: None,
            access_denied: false,
            is_directory: false,
        })
    }
}

#[cfg(target_os = "macos")]
fn file_access_denied(path: Option<&std::path::Path>) -> bool {
    path.is_some_and(|path| {
        std::fs::File::open(path)
            .is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied)
    })
}

#[cfg(not(target_os = "macos"))]
fn file_access_denied(_path: Option<&std::path::Path>) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    use chrono::Utc;
    use std::sync::Arc;

    use crate::{
        history::{ClipMetadata, ContentType, Flavor, NewClip},
        storage::Database,
    };

    #[test]
    fn preview_resource_ipc_shape_is_stable() {
        assert_eq!(
            serde_json::to_value(PreviewResource {
                data_url: Some("data:image/png;base64,AA==".into()),
                byte_size: Some(1),
                access_denied: false,
                is_directory: false,
            })
            .unwrap(),
            serde_json::json!({
                "data_url": "data:image/png;base64,AA==",
                "byte_size": 1,
                "access_denied": false,
                "is_directory": false
            })
        );
    }

    #[test]
    fn derivation_preserves_resource_shapes() {
        let png = STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
            .unwrap();
        let flavors = vec![Flavor {
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
    }

    #[test]
    fn source_icon_rejects_unknown_clip_and_accepts_missing_source() {
        let history = HistoryService::new(Arc::new(Database::in_memory().unwrap()));
        let assets = AssetService::new(history.clone());
        assert!(assets.source_app_icon("missing").is_err());
        let id = history
            .capture(&NewClip {
                content_type: ContentType::Text,
                plain_text: Some("hello".into()),
                preview: "hello".into(),
                source_app: None,
                flavors: vec![Flavor {
                    format: "text/plain".into(),
                    payload: b"hello".to_vec(),
                }],
                metadata: ClipMetadata::default(),
                content_hash: "icon-contract".into(),
                created_at: Utc::now(),
            })
            .unwrap();
        assert!(assets.source_app_icon(&id).unwrap().data_url.is_none());
    }

    #[test]
    fn non_file_metadata_never_grants_file_access() {
        let history = HistoryService::new(Arc::new(Database::in_memory().unwrap()));
        let assets = AssetService::new(history.clone());
        let id = history
            .capture(&NewClip {
                content_type: ContentType::Image,
                plain_text: None,
                preview: "malformed".into(),
                source_app: None,
                flavors: vec![],
                metadata: ClipMetadata {
                    files: vec!["/tmp/should-not-be-read.png".into()],
                    ..Default::default()
                },
                content_hash: "malformed-file-metadata".into(),
                created_at: Utc::now(),
            })
            .unwrap();

        assert!(assets.asset(&id).unwrap().data_url.is_none());
        assert!(matches!(
            assets.file_asset(&id, 0),
            Err(AppError::Validation(_))
        ));
    }
}
