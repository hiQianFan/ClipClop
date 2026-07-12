use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::error::AppResult;
use crate::storage::Database;

use super::{ClipDetail, ClipPage, ListClipsRequest, NewClip};

#[derive(Clone)]
pub struct ClipService {
    database: Arc<Database>,
}

impl ClipService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
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

    pub fn list(&self, request: &ListClipsRequest) -> AppResult<ClipPage> {
        self.database.list_clips(request)
    }

    pub fn get(&self, id: &str) -> AppResult<ClipDetail> {
        self.database.get_clip(id)
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.database.delete_clip(id)
    }

    pub fn clear(&self) -> AppResult<u64> {
        self.database.clear()
    }
}
