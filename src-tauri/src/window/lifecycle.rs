use std::sync::{Mutex, MutexGuard};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PanelPhase {
    Hidden,
    Showing,
    Focused,
    BlurPending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlurToken {
    generation: u64,
    revision: u64,
}

#[derive(Clone, Copy, Debug)]
struct Lifecycle {
    generation: u64,
    revision: u64,
    phase: PanelPhase,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self {
            generation: 0,
            revision: 0,
            phase: PanelPhase::Hidden,
        }
    }
}

#[derive(Default)]
pub(crate) struct PanelLifecycleState {
    main: Mutex<Lifecycle>,
    quick: Mutex<Lifecycle>,
}

impl PanelLifecycleState {
    fn lock(&self, label: &str) -> MutexGuard<'_, Lifecycle> {
        let state = if label == "quick" {
            &self.quick
        } else {
            &self.main
        };
        state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn begin_show(&self, label: &str, already_focused: bool) {
        let mut state = self.lock(label);
        state.generation = state.generation.wrapping_add(1);
        state.revision = state.revision.wrapping_add(1);
        state.phase = if already_focused {
            PanelPhase::Focused
        } else {
            PanelPhase::Showing
        };
    }

    pub(crate) fn mark_focused(&self, label: &str) {
        let mut state = self.lock(label);
        state.revision = state.revision.wrapping_add(1);
        state.phase = PanelPhase::Focused;
    }

    pub(crate) fn begin_blur(&self, label: &str) -> Option<BlurToken> {
        let mut state = self.lock(label);
        if state.phase != PanelPhase::Focused {
            return None;
        }
        state.revision = state.revision.wrapping_add(1);
        state.phase = PanelPhase::BlurPending;
        Some(BlurToken {
            generation: state.generation,
            revision: state.revision,
        })
    }

    pub(crate) fn can_hide(&self, label: &str, token: BlurToken) -> bool {
        let state = self.lock(label);
        state.phase == PanelPhase::BlurPending
            && state.generation == token.generation
            && state.revision == token.revision
    }

    pub(crate) fn mark_hidden(&self, label: &str) {
        let mut state = self.lock(label);
        state.generation = state.generation.wrapping_add(1);
        state.revision = state.revision.wrapping_add(1);
        state.phase = PanelPhase::Hidden;
    }

    pub(crate) fn is_shown(&self, label: &str) -> bool {
        self.lock(label).phase != PanelPhase::Hidden
    }
}

#[cfg(test)]
mod tests {
    use super::PanelLifecycleState;

    #[test]
    fn startup_blur_is_ignored_until_focus_is_acquired() {
        let state = PanelLifecycleState::default();
        state.begin_show("main", false);
        assert_eq!(state.begin_blur("main"), None);

        state.mark_focused("main");
        assert!(state.begin_blur("main").is_some());
    }

    #[test]
    fn refocus_invalidates_pending_blur() {
        let state = PanelLifecycleState::default();
        state.begin_show("main", true);
        let token = state.begin_blur("main").unwrap();
        state.mark_focused("main");
        assert!(!state.can_hide("main", token));
    }

    #[test]
    fn new_show_or_hide_invalidates_pending_blur() {
        let state = PanelLifecycleState::default();
        state.begin_show("main", true);
        let token = state.begin_blur("main").unwrap();
        state.begin_show("main", false);
        assert!(!state.can_hide("main", token));

        state.mark_focused("main");
        let token = state.begin_blur("main").unwrap();
        state.mark_hidden("main");
        assert!(!state.can_hide("main", token));
    }

    #[test]
    fn duplicate_blur_creates_only_one_pending_hide() {
        let state = PanelLifecycleState::default();
        state.begin_show("main", true);
        assert!(state.begin_blur("main").is_some());
        assert_eq!(state.begin_blur("main"), None);
    }

    #[test]
    fn shown_state_changes_synchronously() {
        let state = PanelLifecycleState::default();
        assert!(!state.is_shown("main"));
        state.begin_show("main", false);
        assert!(state.is_shown("main"));
        assert!(!state.is_shown("quick"));
        state.mark_hidden("main");
        assert!(!state.is_shown("main"));
    }
}
