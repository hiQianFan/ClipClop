use std::sync::Arc;

use crate::{
    assets::AssetService, history::HistoryService, onboarding::OnboardingService,
    paste::PasteController, preview::ExternalPreviewService, settings::SettingsService,
    storage::Database,
};

pub struct AppState {
    pub history: HistoryService,
    pub assets: AssetService,
    pub external_preview: ExternalPreviewService,
    pub paste: PasteController,
    pub settings: SettingsService,
    pub onboarding: OnboardingService,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let database = Arc::new(database);
        let history = HistoryService::new(database.clone());
        Self {
            assets: AssetService::new(history.clone()),
            external_preview: ExternalPreviewService::new(history.clone()),
            history,
            paste: PasteController::default(),
            settings: SettingsService::new(database.clone()),
            onboarding: OnboardingService::new(database),
        }
    }
}
