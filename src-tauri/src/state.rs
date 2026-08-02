use std::sync::Arc;

use crate::{
    history::HistoryService, onboarding::OnboardingService, paste::PasteController,
    preview::PreviewService, settings::SettingsService, storage::Database,
};

pub struct AppState {
    pub history: HistoryService,
    pub preview: PreviewService,
    pub paste: PasteController,
    pub settings: SettingsService,
    pub onboarding: OnboardingService,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let database = Arc::new(database);
        let history = HistoryService::new(database.clone());
        Self {
            preview: PreviewService::new(history.clone()),
            history,
            paste: PasteController::default(),
            settings: SettingsService::new(database.clone()),
            onboarding: OnboardingService::new(database),
        }
    }
}
