use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod fallback;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use fallback as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
use windows as platform;

pub(super) const TARGET_FOCUS_TIMEOUT: Duration = Duration::from_millis(500);
const FOCUS_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PasteOutcome {
    Pasted,
    CopiedPermissionRequired,
    CopiedTargetLost,
    CopiedFocusFailed,
    CopiedInjectionFailed,
    AlreadyInProgress,
    CopiedUnsupportedPlatform,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum PasteTarget {
    #[cfg(target_os = "macos")]
    Mac { pid: i32 },
    #[cfg(target_os = "windows")]
    Windows { hwnd: isize, pid: u32 },
}

#[derive(Clone)]
pub struct PasteController {
    target: Arc<Mutex<Option<PasteTarget>>>,
    in_flight: Arc<AtomicBool>,
}

impl Default for PasteController {
    fn default() -> Self {
        Self {
            target: Arc::new(Mutex::new(None)),
            in_flight: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl PasteController {
    pub(crate) fn capture_target(&self) {
        let target = platform::capture_target();
        if let Ok(mut stored) = self.target.lock() {
            *stored = target;
        }
    }

    pub(crate) fn try_begin(&self) -> Option<InFlightGuard<'_>> {
        if self.in_flight.swap(true, Ordering::AcqRel) {
            return None;
        }
        Some(InFlightGuard(&self.in_flight))
    }

    pub(crate) fn paste_to_target(&self, _guard: InFlightGuard<'_>) -> PasteOutcome {
        let target = self.target.lock().ok().and_then(|target| *target);
        let Some(target) = target else {
            return PasteOutcome::CopiedTargetLost;
        };
        platform::paste(target)
    }
}

pub(crate) struct InFlightGuard<'a>(&'a AtomicBool);

impl Drop for InFlightGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

pub(super) fn wait_until(mut predicate: impl FnMut() -> bool, timeout: Duration) -> bool {
    let started = Instant::now();
    loop {
        if predicate() {
            return true;
        }
        if started.elapsed() >= timeout {
            return false;
        }
        thread::sleep(FOCUS_POLL_INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wait_until_stops_when_condition_becomes_true() {
        let mut attempts = 0;
        assert!(wait_until(
            || {
                attempts += 1;
                attempts == 2
            },
            Duration::from_millis(50)
        ));
    }

    #[test]
    fn paste_outcomes_use_stable_snake_case_values() {
        assert_eq!(
            serde_json::to_string(&PasteOutcome::CopiedFocusFailed).unwrap(),
            "\"copied_focus_failed\""
        );
        assert_eq!(
            serde_json::to_string(&PasteOutcome::AlreadyInProgress).unwrap(),
            "\"already_in_progress\""
        );
    }

    #[test]
    fn only_one_paste_can_hold_the_clipboard_write_permit() {
        let controller = PasteController::default();
        let permit = controller.try_begin().expect("first paste should start");
        assert!(controller.try_begin().is_none());
        drop(permit);
        assert!(controller.try_begin().is_some());
    }
}
