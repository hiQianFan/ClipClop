use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Emitter, LogicalSize, Manager, WebviewWindow};

use crate::state::AppState;

#[cfg(target_os = "macos")]
use objc::{sel, sel_impl};

const SHADOW_INSET: f64 = 20.0;

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
