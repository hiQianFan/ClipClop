use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Emitter, LogicalSize, Manager, WebviewWindow};

use crate::state::AppState;

#[cfg(target_os = "macos")]
use objc::{sel, sel_impl};

const SHADOW_INSET: f64 = 20.0;

// When the panel is summoned from a global shortcut, our process is in the background and
// Windows' foreground lock refuses to let a background process take the foreground — so a
// bare `set_focus()` shows the panel without keyboard focus (arrow keys / Enter do nothing
// until the user clicks).
//
// We take the foreground *without injecting synthetic input*: temporarily set the system
// foreground-lock timeout to 0, which lets SetForegroundWindow succeed directly, then
// restore the previous timeout. Two earlier approaches injected input to lift the lock and
// both crashed the app: AttachThreadInput could deadlock our UI thread, and a synthetic Alt
// key (tao's own trick) reliably triggered an upstream tao event-loop re-entrancy panic
// (tao#1180, "cannot move state from Destroyed") — injecting input enters menu mode and
// pumps re-entrant paint messages. The SPI approach avoids that entirely. Do not reintroduce
// input injection here.
#[cfg(target_os = "windows")]
fn force_foreground(window: &WebviewWindow) {
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::SetFocus,
        WindowsAndMessaging::{
            GetForegroundWindow, IsIconic, SetForegroundWindow, ShowWindow, SystemParametersInfoW,
            SPI_GETFOREGROUNDLOCKTIMEOUT, SPI_SETFOREGROUNDLOCKTIMEOUT, SW_RESTORE, SW_SHOW,
        },
    };

    let Ok(handle) = window.hwnd() else {
        log::warn!("force_foreground: window has no HWND yet; skipping");
        return;
    };
    // Tauri's HWND wraps `*mut c_void`, which is exactly windows-sys' HWND type alias.
    let hwnd = handle.0;

    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        } else {
            ShowWindow(hwnd, SW_SHOW);
        }

        // Only lift the foreground lock when we are not already the foreground window.
        if hwnd != GetForegroundWindow() {
            // Read the current foreground-lock timeout so we can restore it afterwards.
            let mut previous_timeout: u32 = 0;
            let read_ok = SystemParametersInfoW(
                SPI_GETFOREGROUNDLOCKTIMEOUT,
                0,
                (&mut previous_timeout as *mut u32).cast(),
                0,
            ) != 0;

            // For SPI_SETFOREGROUNDLOCKTIMEOUT the new value is passed *in* pvparam itself (as
            // a UINT_PTR), not via a pointer. 0 = allow foreground changes immediately. Flags
            // 0 = update the running value only (no registry write, no WM_SETTINGCHANGE).
            SystemParametersInfoW(SPI_SETFOREGROUNDLOCKTIMEOUT, 0, std::ptr::null_mut(), 0);

            SetForegroundWindow(hwnd);

            if read_ok {
                SystemParametersInfoW(
                    SPI_SETFOREGROUNDLOCKTIMEOUT,
                    0,
                    previous_timeout as usize as *mut core::ffi::c_void,
                    0,
                );
            }
        }

        // Route keyboard input to the window. Raw SetFocus only — never tao's set_focus,
        // whose SetForegroundWindow-failed fallback injects an Alt key (the tao#1180 crash).
        SetFocus(hwnd);
    }
}

#[derive(Default)]
pub struct PreviewState {
    active: AtomicBool,
}

impl PreviewState {
    pub(crate) fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

fn panel_content_size(work_area_width: f64, work_area_height: f64) -> (f64, f64) {
    let (target_width, target_height): (f64, f64) =
        if work_area_width >= 1600.0 && work_area_height >= 900.0 {
            (960.0, 720.0)
        } else if work_area_width >= 1100.0 && work_area_height >= 720.0 {
            (800.0, 600.0)
        } else {
            (720.0, 540.0)
        };
    let max_width = (work_area_width - SHADOW_INSET * 2.0).max(0.0);
    let max_height = (work_area_height - SHADOW_INSET * 2.0).max(0.0);
    (target_width.min(max_width), target_height.min(max_height))
}

pub(crate) fn resize_panel_for_monitor(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let work_area = monitor
        .work_area()
        .size
        .to_logical::<f64>(monitor.scale_factor());
    let (content_width, content_height) = panel_content_size(work_area.width, work_area.height);
    let _ = window.set_size(LogicalSize::new(
        content_width + SHADOW_INSET * 2.0,
        content_height + SHADOW_INSET * 2.0,
    ));
}

#[cfg(target_os = "macos")]
fn activate_clipclop() {
    use objc::{class, msg_send};

    unsafe {
        let application: *mut objc::runtime::Object =
            msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![application, activateIgnoringOtherApps: true];
    }
}

pub(crate) fn show_panel(app: &tauri::AppHandle) {
    app.state::<AppState>().paste.capture_target();
    if let Some(window) = app.get_webview_window("main") {
        resize_panel_for_monitor(&window);
        let _ = window.center();
        let _ = app.emit("panel_shown", ());
        #[cfg(target_os = "macos")]
        {
            use tauri_nspanel::ManagerExt;
            if let Ok(panel) = app.get_webview_panel("main") {
                activate_clipclop();
                panel.make_key_and_order_front();
                return;
            }
        }
        let _ = window.show();
        #[cfg(target_os = "windows")]
        {
            log::info!("show_panel: forcing foreground");
            force_foreground(&window);
            log::info!("show_panel: foreground done");
        }
        #[cfg(not(target_os = "windows"))]
        let _ = window.set_focus();
    }
}

pub(crate) fn toggle_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            #[cfg(target_os = "macos")]
            {
                use tauri_plugin_quicklook::QuicklookExt;
                let _ = app.quicklook().queue_hide();
                app.state::<PreviewState>().set_active(false);
            }
            let _ = window.hide();
        } else {
            show_panel(app);
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn handle_quicklook_key(
    _delegate: &objc::runtime::Object,
    _selector: objc::runtime::Sel,
    panel: *mut objc::runtime::Object,
    event: *mut objc::runtime::Object,
) -> bool {
    let event_type: u64 = unsafe { objc::msg_send![event, type] };
    let key_code: u16 = unsafe { objc::msg_send![event, keyCode] };
    if event_type == 10 && matches!(key_code, 49 | 53) {
        let _: () = unsafe {
            objc::msg_send![panel, orderOut: std::ptr::null_mut::<objc::runtime::Object>()]
        };
        true
    } else {
        false
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn install_quicklook_key_handler() {
    use objc::runtime::{method_setImplementation, Class, Imp};
    let class = [
        "QLPreviewPanelDelegate",
        "quicklook::interop::qlpreviewpaneldelegate::QLPreviewPanelDelegate0.2.0",
    ]
    .into_iter()
    .find_map(Class::get);
    let Some(class) = class else {
        log::warn!("Quick Look delegate class is unavailable");
        return;
    };
    let Some(method) = class.instance_method(sel!(previewPanel:handleEvent:)) else {
        log::warn!("Quick Look delegate event handler is unavailable");
        return;
    };
    unsafe {
        method_setImplementation(
            method as *const _ as *mut _,
            std::mem::transmute::<
                unsafe extern "C" fn(
                    &objc::runtime::Object,
                    objc::runtime::Sel,
                    *mut objc::runtime::Object,
                    *mut objc::runtime::Object,
                ) -> bool,
                Imp,
            >(handle_quicklook_key),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::panel_content_size;

    #[test]
    fn panel_uses_bounded_size_tiers() {
        assert_eq!(panel_content_size(1000.0, 700.0), (720.0, 540.0));
        assert_eq!(panel_content_size(1440.0, 900.0), (800.0, 600.0));
        assert_eq!(panel_content_size(1920.0, 1080.0), (960.0, 720.0));
    }

    #[test]
    fn panel_never_exceeds_the_monitor_work_area() {
        assert_eq!(panel_content_size(800.0, 560.0), (720.0, 520.0));
    }
}
