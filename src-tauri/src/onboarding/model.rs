use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ONBOARDING_KEY: &str = "onboarding";
pub const ONBOARDING_REVISION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStep {
    Overview,
    Practice,
    AutoPaste,
    FilePreview,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingExample {
    Image,
    Link,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnboardingState {
    pub completed_revision: Option<u32>,
    pub current_step: Option<OnboardingStep>,
    pub visited_steps: Vec<OnboardingStep>,
    pub selected_example: Option<OnboardingExample>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl OnboardingState {
    pub fn pending() -> Self {
        Self {
            completed_revision: None,
            current_step: Some(OnboardingStep::Overview),
            visited_steps: vec![OnboardingStep::Overview],
            selected_example: Some(OnboardingExample::Image),
            extra: BTreeMap::new(),
        }
    }

    pub fn completed() -> Self {
        Self {
            completed_revision: Some(ONBOARDING_REVISION),
            current_step: None,
            visited_steps: vec![],
            selected_example: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        match (self.completed_revision, self.current_step) {
            (None, Some(step)) => self.visited_steps.contains(&step),
            (Some(revision), None) => {
                revision > 0 && self.visited_steps.is_empty() && self.selected_example.is_none()
            }
            _ => false,
        }
    }
}
