use std::sync::Arc;

use crate::{
    error::{AppError, AppResult},
    storage::Database,
};

use super::{OnboardingState, ONBOARDING_KEY};

#[derive(Clone)]
pub struct OnboardingService {
    database: Arc<Database>,
}

impl OnboardingService {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn initialize(&self, database_existed: bool) -> AppResult<()> {
        if self
            .database
            .get_setting::<OnboardingState>(ONBOARDING_KEY)?
            .is_none()
        {
            self.database.set_setting(
                ONBOARDING_KEY,
                &if database_existed {
                    OnboardingState::completed()
                } else {
                    OnboardingState::pending()
                },
            )?;
        }
        Ok(())
    }

    pub fn get(&self) -> AppResult<OnboardingState> {
        self.database
            .get_setting(ONBOARDING_KEY)?
            .ok_or_else(|| AppError::Storage("onboarding state is not initialized".into()))
    }

    pub fn save(&self, state: OnboardingState) -> AppResult<OnboardingState> {
        if !state.is_valid() {
            return Err(AppError::Validation("invalid onboarding state".into()));
        }
        self.database.set_setting(ONBOARDING_KEY, &state)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::onboarding::{OnboardingExample, OnboardingStep, ONBOARDING_REVISION};

    fn service() -> OnboardingService {
        OnboardingService::new(Arc::new(Database::in_memory().unwrap()))
    }

    #[test]
    fn new_and_existing_databases_get_different_initial_states() {
        let fresh = service();
        fresh.initialize(false).unwrap();
        assert_eq!(fresh.get().unwrap(), OnboardingState::pending());

        let upgrade = service();
        upgrade.initialize(true).unwrap();
        assert_eq!(upgrade.get().unwrap(), OnboardingState::completed());
    }

    #[test]
    fn initialization_never_overwrites_a_saved_journey() {
        let service = service();
        service.initialize(false).unwrap();
        let state = OnboardingState {
            completed_revision: None,
            current_step: Some(OnboardingStep::Practice),
            visited_steps: vec![OnboardingStep::Overview, OnboardingStep::Practice],
            selected_example: Some(OnboardingExample::Text),
            extra: Default::default(),
        };
        service.save(state.clone()).unwrap();
        service.initialize(true).unwrap();
        assert_eq!(service.get().unwrap(), state);
    }

    #[test]
    fn rejects_invalid_states_and_accepts_completed_state() {
        let service = service();
        service.initialize(false).unwrap();
        let invalid = OnboardingState {
            completed_revision: None,
            current_step: Some(OnboardingStep::Practice),
            visited_steps: vec![OnboardingStep::Overview],
            selected_example: None,
            extra: Default::default(),
        };
        assert!(service.save(invalid).is_err());
        assert_eq!(
            service
                .save(OnboardingState::completed())
                .unwrap()
                .completed_revision,
            Some(ONBOARDING_REVISION)
        );
    }
}
