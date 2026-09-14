//! Opt-in dev experiment: preserve the external foreground window while routing
//! physical keys through WebView2's input pipeline. OS IME composition is not
//! implemented here; this must not become the default until native validation.
use std::{
    cell::RefCell,
    sync::{mpsc, Mutex, OnceLock},
    time::Duration,
};
use tauri::{Manager, WebviewWindow};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyboardLayout, ToUnicodeEx},
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::*,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Session {
    hwnd: isize,
    target: isize,
    generation: u64,
    label: &'static str,
}
static SESSION: Mutex<Option<Session>> = Mutex::new(None);
static DISMISSED: Mutex<Option<Session>> = Mutex::new(None);
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static QUEUE: OnceLock<mpsc::SyncSender<Key>> = OnceLock::new();
static STARTED: OnceLock<bool> = OnceLock::new();
thread_local! {
    static KEYS: RefCell<([u8; 256], [bool; 256])> = const { RefCell::new(([0; 256], [false; 256])) };
}

struct Key {
    session: Session,
    vk: u32,
    scan: u32,
    down: bool,
    repeat: bool,
    state: [u8; 256],
}

pub(crate) fn enabled() -> bool {
    cfg!(debug_assertions) && std::env::var("CLIPCLOP_WINDOWS_NOACTIVATE").as_deref() == Ok("1")
}

fn current() -> Option<Session> {
    *SESSION.lock().unwrap_or_else(|e| e.into_inner())
}

fn dismiss(expected: Session) {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    if *session == Some(expected) {
        *session = None;
        *DISMISSED.lock().unwrap_or_else(|e| e.into_inner()) = Some(expected);
    }
}

pub(crate) fn stop(window: &WebviewWindow) {
    let mut session = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    if session.is_some_and(|s| s.label == window.label()) {
        *session = None;
    }
}

unsafe extern "system" fn subclass(
    hwnd: HWND,
    message: u32,
    w: WPARAM,
    l: LPARAM,
    id: usize,
    _: usize,
) -> LRESULT {
    if message == WM_MOUSEACTIVATE {
        return MA_NOACTIVATE as LRESULT;
    }
    if message == WM_NCDESTROY {
        unsafe {
            RemoveWindowSubclass(hwnd, Some(subclass), id);
        }
    }
    unsafe { DefSubclassProc(hwnd, message, w, l) }
}

// Called on the window thread. No SetFocus / SetForegroundWindow / WebView2
// MoveFocus: any of those could undo the original application's input context.
pub(crate) fn show(window: &WebviewWindow) -> tauri::Result<()> {
    let app = window.app_handle().clone();
    if !*STARTED.get_or_init(|| start(app)) {
        return Err(tauri::Error::FailedToReceiveMessage);
    }
    let hwnd = window.hwnd()?.0;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE as isize);
        if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_NOACTIVATE as isize == 0 {
            return Err(tauri::Error::FailedToReceiveMessage);
        }
        if SetWindowSubclass(hwnd, Some(subclass), 1, 0) == 0 {
            return Err(tauri::Error::FailedToReceiveMessage);
        }
        let target = current().map_or(GetForegroundWindow() as isize, |s| s.target);
        let label = if window.label() == "quick" {
            "quick"
        } else {
            "main"
        };
        *SESSION.lock().unwrap_or_else(|e| e.into_inner()) = Some(Session {
            hwnd: hwnd as isize,
            target,
            label,
            generation: GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        });
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        if SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        ) == 0
        {
            stop(window);
            ShowWindow(hwnd, SW_HIDE);
            return Err(tauri::Error::FailedToReceiveMessage);
        }
    }
    log::info!("Windows nonactivating dev panel shown; IME composition is not supported");
    Ok(())
}

fn start(app: tauri::AppHandle) -> bool {
    let (tx, rx) = mpsc::sync_channel(128);
    if QUEUE.set(tx).is_err() {
        return false;
    }
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    if std::thread::Builder::new()
        .name("panel-input-hook".into())
        .spawn(move || unsafe {
            KEYS.with(|keys| {
                let mut keys = keys.borrow_mut();
                for vk in 0..256 {
                    keys.0[vk] = if GetAsyncKeyState(vk as i32) < 0 {
                        0x80
                    } else {
                        0
                    };
                }
            });
            let keyboard =
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), std::ptr::null_mut(), 0);
            let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), std::ptr::null_mut(), 0);
            let ok = !keyboard.is_null() && !mouse.is_null();
            let _ = ready_tx.send(ok);
            if ok {
                let mut message = std::mem::zeroed();
                while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            if !keyboard.is_null() {
                UnhookWindowsHookEx(keyboard);
            }
            if !mouse.is_null() {
                UnhookWindowsHookEx(mouse);
            }
        })
        .is_err()
    {
        return false;
    }
    if ready_rx.recv_timeout(Duration::from_secs(2)) != Ok(true) {
        return false;
    }
    std::thread::Builder::new()
        .name("panel-input-dispatch".into())
        .spawn(move || {
            loop {
                match rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(key) if current() == Some(key.session) => dispatch(&app, key),
                    Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                if let Some(s) = current() {
                    if unsafe { GetForegroundWindow() as isize } != s.target {
                        dismiss(s);
                    }
                }
                let dismissed = DISMISSED.lock().unwrap_or_else(|e| e.into_inner()).take();
                if let Some(s) = dismissed {
                    let app2 = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        // A newly opened panel invalidates this dismissal.
                        if current().is_none()
                            && GENERATION.load(std::sync::atomic::Ordering::Relaxed)
                                == s.generation.wrapping_add(1)
                        {
                            let _ = super::super::hide_panel(
                                &app2,
                                s.label,
                                super::super::HideReason::Blur,
                            );
                        }
                    });
                }
            }
        })
        .is_ok()
}

unsafe extern "system" fn mouse_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 && matches!(w as u32, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN) {
        if let Some(s) = current() {
            let event = unsafe { &*(l as *const MSLLHOOKSTRUCT) };
            let root = unsafe { GetAncestor(WindowFromPoint(event.pt), GA_ROOT) };
            if root as isize != s.hwnd {
                dismiss(s);
            }
        }
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, w, l) }
}

unsafe extern "system" fn keyboard_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(std::ptr::null_mut(), code, w, l) };
    }
    let event = unsafe { &*(l as *const KBDLLHOOKSTRUCT) };
    if event.flags & LLKHF_INJECTED != 0 || event.vkCode > 255 {
        return unsafe { CallNextHookEx(std::ptr::null_mut(), code, w, l) };
    }
    let down = matches!(w as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let consumed = KEYS.with(|keys| {
        let mut keys = keys.borrow_mut();
        let vk = event.vkCode as usize;
        let repeat = keys.0[vk] & 0x80 != 0;
        keys.0[vk] = if down { 0x80 } else { 0 };
        // Generic modifier codes are what ToUnicodeEx and CDP use.
        for (generic, left, right) in [(16, 160, 161), (17, 162, 163), (18, 164, 165)] {
            keys.0[generic] = keys.0[left] | keys.0[right];
        }
        let was_consumed = keys.1[vk];
        if !down {
            keys.1[vk] = false;
        }
        let Some(s) = current() else {
            return consume_key(down, false, was_consumed, false);
        };
        let external_switch =
            keys.0[91] != 0 || keys.0[92] != 0 || (keys.0[18] != 0 && matches!(vk, 9 | 27));
        if external_switch || unsafe { GetForegroundWindow() as isize } != s.target {
            dismiss(s);
            return consume_key(down, false, was_consumed, false);
        }
        let key = Key {
            session: s,
            vk: event.vkCode,
            scan: event.scanCode,
            down,
            repeat,
            state: keys.0,
        };
        if QUEUE.get().is_none_or(|q| q.try_send(key).is_err()) {
            // Never keep intercepting input after the receiver fails.
            dismiss(s);
        }
        // Pass modifier transitions through so OS shortcut state cannot stick.
        let modifier = matches!(vk, 16..=18 | 160..=165);
        if down && !modifier {
            keys.1[vk] = true;
        }
        consume_key(down, modifier, was_consumed, true)
    });
    if consumed {
        1
    } else {
        unsafe { CallNextHookEx(std::ptr::null_mut(), code, w, l) }
    }
}

fn consume_key(down: bool, modifier: bool, was_consumed: bool, capturing: bool) -> bool {
    if !capturing {
        return was_consumed;
    }
    !modifier && (down || was_consumed)
}

fn dispatch(app: &tauri::AppHandle, key: Key) {
    let Some(window) = app.get_webview_window(key.session.label) else {
        return;
    };
    let mut chars = [0u16; 8];
    let mut text_state = key.state;
    for vk in [17, 18, 162, 163, 164, 165] {
        text_state[vk] = 0;
    }
    let count = unsafe {
        let thread = GetWindowThreadProcessId(key.session.target as HWND, std::ptr::null_mut());
        ToUnicodeEx(
            key.vk,
            key.scan,
            text_state.as_ptr(),
            chars.as_mut_ptr(),
            chars.len() as i32,
            4,
            GetKeyboardLayout(thread),
        )
    };
    let text = if count > 0 {
        String::from_utf16_lossy(&chars[..(count as usize).min(chars.len())])
    } else {
        String::new()
    };
    let modifiers = i32::from(key.state[18] != 0)
        | (i32::from(key.state[17] != 0) << 1)
        | (i32::from(key.state[16] != 0) << 3);
    let printable =
        key.down && modifiers & 3 == 0 && !text.is_empty() && !text.chars().any(char::is_control);
    let name = key_name(key.vk)
        .map(str::to_owned)
        .unwrap_or_else(|| text.clone());
    let params = serde_json::json!({
        "type": if !key.down { "keyUp" } else if printable { "keyDown" } else { "rawKeyDown" },
        "key": name, "windowsVirtualKeyCode": key.vk,
        "modifiers": modifiers, "autoRepeat": key.down && key.repeat,
        "text": if printable { text.as_str() } else { "" },
    })
    .to_string();
    let session = key.session;
    if let Err(error) = window.with_webview(move |webview| {
        if current() != Some(session) || unsafe { GetForegroundWindow() as isize } != session.target
        {
            return;
        }
        let result = unsafe {
            webview.controller().CoreWebView2().and_then(|core| {
                let handler = webview2_com::CallDevToolsProtocolMethodCompletedHandler::create(
                    Box::new(move |result, _| {
                        if let Err(error) = result {
                            log::warn!("panel keyboard dispatch failed: {error}");
                            dismiss(session);
                        }
                        Ok(())
                    }),
                );
                core.CallDevToolsProtocolMethod(
                    &::windows::core::HSTRING::from("Input.dispatchKeyEvent"),
                    &::windows::core::HSTRING::from(params),
                    &handler,
                )
            })
        };
        if let Err(error) = result {
            log::warn!("panel keyboard dispatch failed: {error}");
            dismiss(session);
        }
    }) {
        log::warn!("panel keyboard scheduling failed: {error}");
        dismiss(session);
    }
}

fn key_name(vk: u32) -> Option<&'static str> {
    Some(match vk {
        8 => "Backspace",
        9 => "Tab",
        13 => "Enter",
        16 | 160 | 161 => "Shift",
        17 | 162 | 163 => "Control",
        18 | 164 | 165 => "Alt",
        27 => "Escape",
        32 => " ",
        33 => "PageUp",
        34 => "PageDown",
        35 => "End",
        36 => "Home",
        37 => "ArrowLeft",
        38 => "ArrowUp",
        39 => "ArrowRight",
        40 => "ArrowDown",
        45 => "Insert",
        46 => "Delete",
        121 => "F10",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn closing_panel_drains_held_keys_without_swallowing_new_input() {
        use super::consume_key;
        // Enter closes the panel on keydown. Repeats and its matching keyup
        // must not leak into the editor after the panel has disappeared.
        assert!(consume_key(true, false, false, true));
        assert!(consume_key(true, false, true, false));
        assert!(consume_key(false, false, true, false));
        assert!(!consume_key(true, false, false, false));
        assert!(!consume_key(false, false, false, true));
        assert!(!consume_key(true, true, false, true));
        assert!(!consume_key(false, true, false, true));
    }

    #[test]
    fn old_session_cannot_dismiss_new_panel() {
        let old = super::Session {
            hwnd: 1,
            target: 2,
            generation: 1,
            label: "quick",
        };
        let new = super::Session {
            hwnd: 3,
            generation: 2,
            label: "main",
            ..old
        };
        *super::SESSION.lock().unwrap() = Some(new);
        super::dismiss(old);
        assert!(super::current() == Some(new));
        super::dismiss(new);
        assert!(super::current().is_none());
        *super::DISMISSED.lock().unwrap() = None;
    }

    #[test]
    fn native_navigation_keys_match_browser_handlers() {
        assert_eq!(super::key_name(13), Some("Enter"));
        assert_eq!(super::key_name(38), Some("ArrowUp"));
        assert_eq!(super::key_name(27), Some("Escape"));
        assert_eq!(super::key_name(65), None);
    }
}
