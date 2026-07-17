use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;

const TARGET_FOCUS_TIMEOUT: Duration = Duration::from_millis(500);
const FOCUS_POLL_INTERVAL: Duration = Duration::from_millis(10);
#[cfg(target_os = "windows")]
const MODIFIER_RELEASE_TIMEOUT: Duration = Duration::from_millis(1_000);

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PasteOutcome {
    Pasted,
    CopiedPermissionRequired,
    CopiedTargetLost,
    CopiedFocusFailed,
    CopiedInjectionFailed,
    CopiedAlreadyInProgress,
    CopiedUnsupportedPlatform,
}

#[derive(Debug, Clone, Copy)]
enum PasteTarget {
    #[cfg(target_os = "macos")]
    Mac { pid: i32 },
    #[cfg(target_os = "windows")]
    Windows { hwnd: isize, pid: u32 },
}

pub struct PasteController {
    target: Mutex<Option<PasteTarget>>,
    in_flight: AtomicBool,
}

impl Default for PasteController {
    fn default() -> Self {
        Self {
            target: Mutex::new(None),
            in_flight: AtomicBool::new(false),
        }
    }
}

impl PasteController {
    pub fn capture_target(&self) {
        let target = platform::capture_target();
        if let Ok(mut stored) = self.target.lock() {
            *stored = target;
        }
    }

    pub fn paste_to_target(&self) -> PasteOutcome {
        if self.in_flight.swap(true, Ordering::AcqRel) {
            return PasteOutcome::CopiedAlreadyInProgress;
        }
        let _guard = InFlightGuard(&self.in_flight);
        let target = self.target.lock().ok().and_then(|target| *target);
        let Some(target) = target else {
            return PasteOutcome::CopiedTargetLost;
        };
        platform::paste(target)
    }
}

struct InFlightGuard<'a>(&'a AtomicBool);

impl Drop for InFlightGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
mod platform {
    use super::*;
    use appkit_nsworkspace_bindings::{INSRunningApplication, INSWorkspace, NSWorkspace};
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};
    use std::ffi::c_void;

    // maskCommand plus the left/right modifier marker used by macOS event consumers.
    const COMMAND_FLAG: u64 = (1 << 20) | 0x000008;
    const KEY_V: u16 = 9;
    const SESSION_EVENT_TAP: u32 = 1;
    const COMBINED_SESSION_STATE: i32 = 0;
    const ACTIVATE_IGNORING_OTHER_APPS: u64 = 1 << 1;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGPreflightPostEventAccess() -> bool;
        fn CGRequestPostEventAccess() -> bool;
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
            // The permission sheet is asynchronous; this attempt still degrades to copied-only.
            unsafe { CGRequestPostEventAccess() };
            return PasteOutcome::CopiedPermissionRequired;
        }

        let app: *mut Object = unsafe {
            msg_send![class!(NSRunningApplication), runningApplicationWithProcessIdentifier: pid]
        };
        if app.is_null() || unsafe { msg_send![app, isTerminated] } {
            return PasteOutcome::CopiedTargetLost;
        }

        // A non-activating panel keeps the original application active. In that
        // normal path, activating it again only creates another focus race.
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
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
                KEYEVENTF_KEYUP, VK_CONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU,
                VK_RSHIFT, VK_RWIN,
            },
            WindowsAndMessaging::{
                GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindow,
                SetForegroundWindow, ShowWindow, SW_RESTORE,
            },
        },
    };

    pub(super) fn capture_target() -> Option<PasteTarget> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            (pid != 0 && pid != std::process::id()).then_some(PasteTarget::Windows {
                hwnd: hwnd as isize,
                pid,
            })
        }
    }

    pub(super) fn paste(target: PasteTarget) -> PasteOutcome {
        let PasteTarget::Windows { hwnd, pid } = target;
        let hwnd = hwnd as HWND;
        if !valid_target(hwnd, pid) {
            return PasteOutcome::CopiedTargetLost;
        }

        unsafe {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            }
            if SetForegroundWindow(hwnd) == 0
                || !wait_until(|| GetForegroundWindow() == hwnd, TARGET_FOCUS_TIMEOUT)
            {
                return PasteOutcome::CopiedFocusFailed;
            }
        }

        if !wait_until(modifiers_released, MODIFIER_RELEASE_TIMEOUT) {
            return PasteOutcome::CopiedInjectionFailed;
        }

        if send_ctrl_v() {
            PasteOutcome::Pasted
        } else {
            PasteOutcome::CopiedInjectionFailed
        }
    }

    fn valid_target(hwnd: HWND, expected_pid: u32) -> bool {
        unsafe {
            if IsWindow(hwnd) == 0 {
                return false;
            }
            let mut actual_pid = 0;
            GetWindowThreadProcessId(hwnd, &mut actual_pid);
            actual_pid == expected_pid
        }
    }

    fn modifiers_released() -> bool {
        const MODIFIERS: [u16; 8] = [
            VK_CONTROL,
            VK_RCONTROL,
            VK_LMENU,
            VK_RMENU,
            VK_LSHIFT,
            VK_RSHIFT,
            VK_LWIN,
            VK_RWIN,
        ];
        unsafe {
            MODIFIERS
                .iter()
                .all(|key| GetAsyncKeyState(i32::from(*key)) & i16::MIN == 0)
        }
    }

    fn send_ctrl_v() -> bool {
        fn key(vk: u16, flags: u32) -> INPUT {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }
        }

        let inputs = [
            key(VK_CONTROL, 0),
            key(b'V' as u16, 0),
            key(b'V' as u16, KEYEVENTF_KEYUP),
            key(VK_CONTROL, KEYEVENTF_KEYUP),
        ];
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            ) == inputs.len() as u32
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    use super::*;

    pub(super) fn capture_target() -> Option<PasteTarget> {
        None
    }

    pub(super) fn paste(_: PasteTarget) -> PasteOutcome {
        PasteOutcome::CopiedUnsupportedPlatform
    }
}

fn wait_until(mut predicate: impl FnMut() -> bool, timeout: Duration) -> bool {
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
    }
}
