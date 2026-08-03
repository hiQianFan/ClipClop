use tauri::WebviewWindow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ActivationOutcome {
    AlreadyForeground,
    ActivatedNormally,
    ActivatedWithFallback,
    Rejected,
    NoWindowHandle,
}

impl ActivationOutcome {
    pub(super) fn has_focus(self) -> bool {
        matches!(
            self,
            Self::AlreadyForeground | Self::ActivatedNormally | Self::ActivatedWithFallback
        )
    }
}

fn with_keyboard_focus(outcome: ActivationOutcome, focused: bool) -> ActivationOutcome {
    if focused {
        outcome
    } else {
        ActivationOutcome::Rejected
    }
}

struct ForegroundLockGuard {
    previous_timeout: Option<u32>,
}

impl ForegroundLockGuard {
    unsafe fn lift() -> Self {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SPI_GETFOREGROUNDLOCKTIMEOUT, SPI_SETFOREGROUNDLOCKTIMEOUT,
        };

        let mut previous_timeout = 0_u32;
        let read_ok = unsafe {
            SystemParametersInfoW(
                SPI_GETFOREGROUNDLOCKTIMEOUT,
                0,
                (&mut previous_timeout as *mut u32).cast(),
                0,
            )
        } != 0;
        let set_ok = unsafe {
            SystemParametersInfoW(SPI_SETFOREGROUNDLOCKTIMEOUT, 0, std::ptr::null_mut(), 0)
        } != 0;
        Self {
            previous_timeout: (read_ok && set_ok).then_some(previous_timeout),
        }
    }
}

impl Drop for ForegroundLockGuard {
    fn drop(&mut self) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SPI_SETFOREGROUNDLOCKTIMEOUT,
        };

        if let Some(previous_timeout) = self.previous_timeout {
            unsafe {
                SystemParametersInfoW(
                    SPI_SETFOREGROUNDLOCKTIMEOUT,
                    0,
                    previous_timeout as usize as *mut core::ffi::c_void,
                    0,
                );
            }
        }
    }
}

// Never use tao's set_focus fallback here. It injects Alt when Windows rejects foreground
// activation and can trigger tao#1180 ("cannot move state from Destroyed").
pub(super) fn focus_foreground(window: &WebviewWindow) -> ActivationOutcome {
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::{GetFocus, SetFocus},
        WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow, IsChild, IsIconic, SetForegroundWindow,
            ShowWindow, SW_RESTORE, SW_SHOW,
        },
    };

    let Ok(handle) = window.hwnd() else {
        return ActivationOutcome::NoWindowHandle;
    };
    let hwnd = handle.0;

    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        } else {
            ShowWindow(hwnd, SW_SHOW);
        }

        if hwnd == GetForegroundWindow() {
            SetFocus(hwnd);
            let focused = GetFocus();
            return with_keyboard_focus(
                ActivationOutcome::AlreadyForeground,
                focused == hwnd || (!focused.is_null() && IsChild(hwnd, focused) != 0),
            );
        }

        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);
        if hwnd == GetForegroundWindow() {
            SetFocus(hwnd);
            let focused = GetFocus();
            return with_keyboard_focus(
                ActivationOutcome::ActivatedNormally,
                focused == hwnd || (!focused.is_null() && IsChild(hwnd, focused) != 0),
            );
        }

        let _foreground_lock = ForegroundLockGuard::lift();
        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);
        if hwnd == GetForegroundWindow() {
            SetFocus(hwnd);
            let focused = GetFocus();
            with_keyboard_focus(
                ActivationOutcome::ActivatedWithFallback,
                focused == hwnd || (!focused.is_null() && IsChild(hwnd, focused) != 0),
            )
        } else {
            ActivationOutcome::Rejected
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_requires_keyboard_focus() {
        assert_eq!(
            with_keyboard_focus(ActivationOutcome::ActivatedNormally, true),
            ActivationOutcome::ActivatedNormally
        );
        assert_eq!(
            with_keyboard_focus(ActivationOutcome::ActivatedWithFallback, false),
            ActivationOutcome::Rejected
        );
    }
}
