#![allow(unexpected_cfgs)]

use super::{wait_until, PasteOutcome, PasteTarget, TARGET_FOCUS_TIMEOUT};
use appkit_nsworkspace_bindings::{INSRunningApplication, INSWorkspace, NSWorkspace};
use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use std::ffi::c_void;

const COMMAND_FLAG: u64 = (1 << 20) | 0x000008;
const KEY_V: u16 = 9;
const SESSION_EVENT_TAP: u32 = 1;
const COMBINED_SESSION_STATE: i32 = 0;
const ACTIVATE_IGNORING_OTHER_APPS: u64 = 1 << 1;

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
    if !unsafe { CGPreflightPostEventAccess() } {
        return PasteOutcome::CopiedPermissionRequired;
    }

    let app: *mut Object = unsafe {
        msg_send![class!(NSRunningApplication), runningApplicationWithProcessIdentifier: pid]
    };
    if app.is_null() || unsafe { msg_send![app, isTerminated] } {
        return PasteOutcome::CopiedTargetLost;
    }

    if frontmost_pid() != Some(pid) {
        let activated: bool =
            unsafe { msg_send![app, activateWithOptions: ACTIVATE_IGNORING_OTHER_APPS] };
        if !activated || !wait_until(|| frontmost_pid() == Some(pid), TARGET_FOCUS_TIMEOUT) {
            return PasteOutcome::CopiedFocusFailed;
        }
    }

    if send_command_v() {
        PasteOutcome::Pasted
    } else {
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
