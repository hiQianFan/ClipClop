use std::time::Duration;

use super::{wait_until, PasteOutcome, PasteTarget, TARGET_FOCUS_TIMEOUT};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
            KEYEVENTF_KEYUP, VK_CONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU,
            VK_RSHIFT, VK_RWIN,
        },
        WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindow, SetForegroundWindow,
            ShowWindow, SW_RESTORE,
        },
    },
};

const MODIFIER_RELEASE_TIMEOUT: Duration = Duration::from_millis(1_000);

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
        let sent = SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent == inputs.len() as u32 {
            return true;
        }
        let releases = [
            key(b'V' as u16, KEYEVENTF_KEYUP),
            key(VK_CONTROL, KEYEVENTF_KEYUP),
        ];
        let _ = SendInput(
            releases.len() as u32,
            releases.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        false
    }
}
