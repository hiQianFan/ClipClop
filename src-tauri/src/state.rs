use std::sync::Arc;

use crate::{clips::ClipService, storage::Database};

pub struct AppState {
    pub clips: ClipService,
    pub database: Arc<Database>,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let database = Arc::new(database);
        Self {
            clips: ClipService::new(database.clone()),
            database,
        }
    }
}
