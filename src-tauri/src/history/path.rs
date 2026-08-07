use std::path::PathBuf;

use super::{ClipDetail, ContentType};

pub(crate) fn normalized_file_path(detail: &ClipDetail, index: usize) -> Option<PathBuf> {
    if detail.summary.content_type != ContentType::File {
        return None;
    }
    detail.summary.metadata.files.get(index).map(|path| {
        url::Url::parse(path)
            .ok()
            .filter(|url| url.scheme() == "file")
            .and_then(|url| url.to_file_path().ok())
            .unwrap_or_else(|| PathBuf::from(path))
    })
}
