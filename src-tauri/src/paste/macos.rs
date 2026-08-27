#![allow(unexpected_cfgs)]

use super::{wait_until_stable, PasteOutcome, PasteTarget, TARGET_FOCUS_TIMEOUT};
use appkit_nsworkspace_bindings::{INSRunningApplication, INSWorkspace, NSWorkspace};
use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use std::{
    ffi::c_void,
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

pub(super) fn capture_target() -> Option<PasteTarget> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let app = workspace.frontmostApplication();
        let pid = app.processIdentifier();
        (pid > 0 && pid as u32 != std::process::id()).then_some(PasteTarget::Mac { pid })
    }
}

pub(super) fn paste(target: PasteTarget) -> PasteOutcome {
    let PasteTarget::Mac { pid } = target;
    let started = Instant::now();
    log::info!(
        "automatic paste restoring target: target_pid={pid}, frontmost_pid={:?}",
        frontmost_pid()
    );
    let app: *mut Object = unsafe {
        msg_send![class!(NSRunningApplication), runningApplicationWithProcessIdentifier: pid]
    };
    if app.is_null() || unsafe { msg_send![app, isTerminated] } {
        log::warn!("automatic paste target unavailable: target_pid={pid}");
        return PasteOutcome::CopiedTargetLost;
    }

    // A non-activating panel can leave the target reported as frontmost while its
    // key window is still recovering. Always request activation, require a stable
    // frontmost result, then allow the original first responder to settle.
    let activation_requested: bool =
        unsafe { msg_send![app, activateWithOptions: ACTIVATE_IGNORING_OTHER_APPS] };
    let stable = wait_until_stable(
        || frontmost_pid() == Some(pid),
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

    if !unsafe { CGPreflightPostEventAccess() } {
        log::warn!("automatic paste event access denied: target_pid={pid}");
        return PasteOutcome::CopiedPermissionRequired;
    }

    if send_command_v() {
        log::info!(
            "automatic paste event posted: target_pid={pid}, elapsed_ms={}",
            started.elapsed().as_millis()
        );
        PasteOutcome::Pasted
    } else {
        log::warn!("automatic paste event injection failed: target_pid={pid}");
        PasteOutcome::CopiedInjectionFailed
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
