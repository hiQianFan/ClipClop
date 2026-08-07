use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{DateTime, Duration, Utc};

use crate::error::AppResult;
use crate::storage::Database;

use super::{ClipDetail, ContentType, Flavor, HistoryPage, HistoryQuery, NewClip};

#[derive(Clone)]
pub struct HistoryService {
    database: Arc<Database>,
    lifecycle: Arc<Mutex<()>>,
}

impl HistoryService {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            lifecycle: Arc::new(Mutex::new(())),
        }
    }

    pub fn capture(&self, clip: &NewClip) -> AppResult<Option<String>> {
        let deduplication_start = Utc::now() - Duration::seconds(2);
        if self
            .database
            .exists_recent_hash(&clip.content_hash, deduplication_start)?
        {
            return Ok(None);
        }
        self.database.insert_clip(clip).map(Some)
    }

    pub fn query(&self, request: &HistoryQuery) -> AppResult<HistoryPage> {
        self.database.query_history(request)
    }

    pub fn get(&self, id: &str) -> AppResult<ClipDetail> {
        let mut detail = self.get_full(id)?;
        if let Some(text) = &mut detail.plain_text {
            truncate_preview(text);
        }
        Ok(detail)
    }

    pub(crate) fn get_full(&self, id: &str) -> AppResult<ClipDetail> {
        self.database.get_clip(id)
    }

    pub fn flavors(&self, id: &str) -> AppResult<Vec<Flavor>> {
        self.database.get_flavors(id)
    }

    pub fn content_type(&self, id: &str) -> AppResult<ContentType> {
        Ok(self.get_full(id)?.summary.content_type)
    }

    pub fn mark_used(&self, id: &str) -> AppResult<bool> {
        let _guard = self.lock_lifecycle()?;
        self.database.touch_clip(id)
    }

    pub fn lock_lifecycle(&self) -> AppResult<MutexGuard<'_, ()>> {
        self.lifecycle
            .lock()
            .map_err(|_| crate::error::AppError::Storage("history lifecycle lock poisoned".into()))
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.database.delete_clip(id)
    }

    pub fn clear(&self) -> AppResult<u64> {
        self.database.clear()
    }

    pub fn retention_cutoff(retention_days: u32) -> DateTime<Utc> {
        Utc::now() - Duration::days(i64::from(retention_days))
    }

    pub fn cleanup_candidates(
        &self,
        retention_days: Option<u32>,
        history_limit: Option<u32>,
    ) -> AppResult<Vec<String>> {
        self.database
            .cleanup_candidate_ids(retention_days.map(Self::retention_cutoff), history_limit)
    }

    pub fn delete_ids(&self, ids: &[String]) -> AppResult<u64> {
        self.database.delete_clip_ids(ids)
    }
}

const MAX_PREVIEW_CHARS: usize = 100_000;

fn truncate_preview(text: &mut String) {
    if let Some((byte_index, _)) = text.char_indices().nth(MAX_PREVIEW_CHARS) {
        text.truncate(byte_index);
        text.push('…');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;
    use chrono::Duration;
    use std::sync::Arc;

    #[test]
    fn preview_limit_preserves_utf8_boundaries() {
        let mut text = "界".repeat(MAX_PREVIEW_CHARS + 1);
        truncate_preview(&mut text);
        assert_eq!(text.chars().count(), MAX_PREVIEW_CHARS + 1);
        assert!(text.ends_with('…'));
    }

    #[test]
    fn capture_deduplicates_for_two_seconds_and_prunes_by_retention() {
        let temp = tempfile::tempdir().unwrap();
        let history = HistoryService::new(Arc::new(
            Database::open(&temp.path().join("history.db")).unwrap(),
        ));
        let clip = NewClip {
            content_type: crate::history::ContentType::Text,
            plain_text: Some("same".into()),
            preview: "same".into(),
            source_app: None,
            flavors: vec![Flavor {
                format: "text/plain".into(),
                payload: b"same".to_vec(),
            }],
            metadata: Default::default(),
            content_hash: "same-hash".into(),
            created_at: Utc::now(),
        };
        assert!(history.capture(&clip).unwrap().is_some());
        assert!(history.capture(&clip).unwrap().is_none());
        let mut old = clip.clone();
        old.content_hash = "old-hash".into();
        old.created_at = Utc::now() - Duration::days(8);
        assert!(history.capture(&old).unwrap().is_some());
        let expired = history.cleanup_candidates(Some(7), None).unwrap();
        assert_eq!(history.delete_ids(&expired).unwrap(), 1);
        assert_eq!(history.query(&HistoryQuery::default()).unwrap().total, 1);
    }
}
