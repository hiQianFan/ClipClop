#![allow(unexpected_cfgs)]

use super::{wait_until_stable, PasteOutcome, PasteTarget, TARGET_FOCUS_TIMEOUT};
use appkit_nsworkspace_bindings::{INSRunningApplication, INSWorkspace, NSWorkspace};
use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use std::{
    ffi::c_void,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

const COMMAND_FLAG: u64 = (1 << 20) | 0x000008;
const KEY_V: u16 = 9;
const SESSION_EVENT_TAP: u32 = 1;
const COMBINED_SESSION_STATE: i32 = 0;
const ACTIVATE_IGNORING_OTHER_APPS: u64 = 1 << 1;
const REQUIRED_STABLE_POLLS: usize = 3;
const FIRST_RESPONDER_SETTLE_DELAY: Duration = Duration::from_millis(60);

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn CGPreflightPostEventAccess() -> bool;
    fn CGEventSourceCreate(state_id: i32) -> *mut c_void;
    fn CGEventCreateKeyboardEvent(
        source: *mut c_void,
        virtual_key: u16,
        key_down: bool,
    ) -> *mut c_void;
    fn CGEventSetFlags(event: *mut c_void, flags: u64);
    fn CGEventPost(tap: u32, event: *mut c_void);
    fn CFRelease(value: *const c_void);
}

pub(super) fn can_inject() -> bool {
    unsafe { CGPreflightPostEventAccess() }
}

pub(super) fn capture_target() -> Option<PasteTarget> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let app = workspace.frontmostApplication();
        let pid = app.processIdentifier();
        (pid > 0 && pid as u32 != std::process::id()).then_some(PasteTarget::Mac { pid })
    }
}

pub(super) fn paste(
    handle: &tauri::AppHandle,
    target: PasteTarget,
    session: Arc<AtomicU64>,
    expected_session: u64,
) -> PasteOutcome {
    let PasteTarget::Mac { pid } = target;
    let started = Instant::now();
    log::info!(
        "automatic paste restoring target: target_pid={pid}, frontmost_pid={:?}",
        frontmost_pid()
    );
    let activation_session = session.clone();
    let activation = crate::window::on_main(handle, move |handle| {
        if activation_session.load(Ordering::Acquire) != expected_session
            || !crate::window::panels_are_hidden(handle)
        {
            return Err(PasteOutcome::CopiedFocusFailed);
        }
        let app: *mut Object = unsafe {
            msg_send![class!(NSRunningApplication), runningApplicationWithProcessIdentifier: pid]
        };
        if app.is_null() || unsafe { msg_send![app, isTerminated] } {
            return Err(PasteOutcome::CopiedTargetLost);
        }
        match activation_needed(pid, frontmost_pid(), std::process::id() as i32) {
            Some(false) => Ok(false),
            Some(true) => {
                let requested: bool =
                    unsafe { msg_send![app, activateWithOptions: ACTIVATE_IGNORING_OTHER_APPS] };
                if requested {
                    Ok(true)
                } else {
                    Err(PasteOutcome::CopiedFocusFailed)
                }
            }
            None => Err(PasteOutcome::CopiedFocusFailed),
        }
    });
    let activation_requested = match activation {
        Ok(Ok(requested)) => requested,
        Ok(Err(outcome)) => return outcome,
        Err(_) => return PasteOutcome::CopiedFocusFailed,
    };
    let mut cancelled = false;
    let stable = wait_until_stable(
        || {
            let foreground = frontmost_pid();
            cancelled |= session.load(Ordering::Acquire) != expected_session
                || activation_needed(pid, foreground, std::process::id() as i32).is_none();
            !cancelled && foreground == Some(pid)
        },
        REQUIRED_STABLE_POLLS,
        TARGET_FOCUS_TIMEOUT,
    );
    log::info!(
        "automatic paste activation result: target_pid={pid}, requested={activation_requested}, stable={stable}, elapsed_ms={}",
        started.elapsed().as_millis()
    );
    if !stable {
        log::warn!(
            "automatic paste focus restore failed: target_pid={pid}, frontmost_pid={:?}",
            frontmost_pid()
        );
        return PasteOutcome::CopiedFocusFailed;
    }
    thread::sleep(FIRST_RESPONDER_SETTLE_DELAY);
    if frontmost_pid() != Some(pid) {
        log::warn!(
            "automatic paste target changed before injection: target_pid={pid}, frontmost_pid={:?}",
            frontmost_pid()
        );
        return PasteOutcome::CopiedFocusFailed;
    }

    let outcome = crate::window::on_main(handle, move |handle| {
        if session.load(Ordering::Acquire) != expected_session
            || !crate::window::panels_are_hidden(handle)
            || frontmost_pid() != Some(pid)
        {
            return PasteOutcome::CopiedFocusFailed;
        }
        if !can_inject() {
            return PasteOutcome::CopiedPermissionRequired;
        }
        if send_command_v() {
            PasteOutcome::Pasted
        } else {
            PasteOutcome::CopiedInjectionFailed
        }
    })
    .unwrap_or(PasteOutcome::CopiedFocusFailed);
    log::info!(
        "automatic paste completed: target_pid={pid}, outcome={outcome:?}, elapsed_ms={}",
        started.elapsed().as_millis()
    );
    outcome
}

// Only restore from our own explicitly activated UI, never from another external app.
fn activation_needed(target: i32, foreground: Option<i32>, own: i32) -> Option<bool> {
    match foreground {
        Some(pid) if pid == target => Some(false),
        Some(pid) if pid == own => Some(true),
        _ => None,
    }
}

fn frontmost_pid() -> Option<i32> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let app = workspace.frontmostApplication();
        let pid = app.processIdentifier();
        (pid > 0).then_some(pid)
    }
}

fn send_command_v() -> bool {
    unsafe {
        let source = CGEventSourceCreate(COMBINED_SESSION_STATE);
        if source.is_null() {
            return false;
        }
        let down = CGEventCreateKeyboardEvent(source, KEY_V, true);
        let up = CGEventCreateKeyboardEvent(source, KEY_V, false);
        if down.is_null() || up.is_null() {
            if !down.is_null() {
                CFRelease(down);
            }
            if !up.is_null() {
                CFRelease(up);
            }
            CFRelease(source);
            return false;
        }
        CGEventSetFlags(down, COMMAND_FLAG);
        CGEventSetFlags(up, COMMAND_FLAG);
        CGEventPost(SESSION_EVENT_TAP, down);
        CGEventPost(SESSION_EVENT_TAP, up);
        CFRelease(down);
        CFRelease(up);
        CFRelease(source);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::activation_needed;

    #[test]
    fn activation_preserves_the_target_and_respects_external_switches() {
        assert_eq!(activation_needed(10, Some(10), 20), Some(false));
        assert_eq!(activation_needed(10, Some(20), 20), Some(true));
        assert_eq!(activation_needed(10, Some(30), 20), None);
        assert_eq!(activation_needed(10, None, 20), None);
    }
}
