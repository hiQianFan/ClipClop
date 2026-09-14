use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InjectionPermission {
    Ready,
    PermissionRequired,
    Unsupported,
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
    session: Arc<AtomicU64>,
}

impl Default for PasteController {
    fn default() -> Self {
        Self {
            target: Arc::new(Mutex::new(None)),
            in_flight: Arc::new(AtomicBool::new(false)),
            session: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl PasteController {
    pub(crate) fn begin_panel_session(&self) {
        self.session.fetch_add(1, Ordering::AcqRel);
    }

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
        Some(InFlightGuard {
            in_flight: &self.in_flight,
            session: self.session.load(Ordering::Acquire),
            target: self.target.lock().ok().and_then(|target| *target),
        })
    }

    pub(crate) fn can_inject(&self) -> bool {
        platform::can_inject()
    }

    pub(crate) fn is_current(&self, guard: &InFlightGuard<'_>) -> bool {
        self.session_is_current(guard.session)
    }

    pub(crate) fn session_is_current(&self, session: u64) -> bool {
        self.session.load(Ordering::Acquire) == session
    }

    pub(crate) fn paste_to_target(
        &self,
        _app: &tauri::AppHandle,
        guard: InFlightGuard<'_>,
    ) -> PasteOutcome {
        if !self.is_current(&guard) {
            return PasteOutcome::CopiedFocusFailed;
        }
        let Some(target) = guard.target else {
            return PasteOutcome::CopiedTargetLost;
        };
        #[cfg(target_os = "macos")]
        {
            platform::paste(_app, target, self.session.clone(), guard.session)
        }
        #[cfg(target_os = "windows")]
        {
            platform::paste(target, self.session.clone(), guard.session)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        platform::paste(target)
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn injection_permission() -> InjectionPermission {
    if macos::can_inject() {
        InjectionPermission::Ready
    } else {
        InjectionPermission::PermissionRequired
    }
}

pub(crate) struct InFlightGuard<'a> {
    in_flight: &'a AtomicBool,
    pub(crate) session: u64,
    target: Option<PasteTarget>,
}

impl Drop for InFlightGuard<'_> {
    fn drop(&mut self) {
        self.in_flight.store(false, Ordering::Release);
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

#[cfg(any(target_os = "macos", test))]
pub(super) fn wait_until_stable(
    mut predicate: impl FnMut() -> bool,
    consecutive_matches: usize,
    timeout: Duration,
) -> bool {
    let mut matches = 0;
    wait_until(
        || {
            matches = if predicate() { matches + 1 } else { 0 };
            matches >= consecutive_matches
        },
        timeout,
    )
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
    fn stable_wait_resets_after_a_transient_match() {
        let states = [true, false, true, true, true];
        let mut index = 0;
        assert!(wait_until_stable(
            || {
                let state = states[index];
                index += 1;
                state
            },
            3,
            Duration::from_millis(500)
        ));
        assert_eq!(index, states.len());
    }

    #[test]
    fn stable_wait_rejects_a_target_that_never_becomes_ready() {
        assert!(!wait_until_stable(|| false, 3, Duration::ZERO));
        assert!(!wait_until_stable(|| true, 3, Duration::ZERO));
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

    #[test]
    fn reopening_a_panel_invalidates_pending_paste_even_for_the_same_target() {
        let controller = PasteController::default();
        controller.begin_panel_session();
        let permit = controller.try_begin().unwrap();
        assert!(controller.is_current(&permit));
        controller.begin_panel_session();
        assert!(!controller.is_current(&permit));
        drop(permit);
        assert!(controller.is_current(&controller.try_begin().unwrap()));
    }
}
