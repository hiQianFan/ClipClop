use objc::{sel, sel_impl};
use tauri_nspanel::{
    objc2::{rc::Retained, MainThreadMarker},
    objc2_app_kit::{NSEvent, NSScreen},
    objc2_foundation::NSPoint,
};

const QUICK_MARGIN: f64 = 6.0;

// AppKit work must finish before a background paste can proceed. Cancel work that
// has not started if the main thread cannot service it within the focus deadline.
pub(crate) fn on_main<T: Send + 'static>(
    app: &tauri::AppHandle,
    action: impl FnOnce(&tauri::AppHandle) -> T + Send + 'static,
) -> tauri::Result<T> {
    if MainThreadMarker::new().is_some() {
        return Ok(action(app));
    }
    let pending = std::sync::Arc::new(std::sync::Mutex::new(Some(action)));
    let work = pending.clone();
    let app_for_main = app.clone();
    let (send, receive) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let action = work
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if let Some(action) = action {
            let _ = send.send(action(&app_for_main));
        }
    })?;
    receive
        .recv_timeout(crate::paste::TARGET_FOCUS_TIMEOUT)
        .map_err(|_| {
            pending
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .take();
            tauri::Error::FailedToReceiveMessage
        })
}

pub(super) fn hide_native_panel(app: &tauri::AppHandle, label: &str) -> tauri::Result<()> {
    use tauri_nspanel::ManagerExt;
    let panel = app
        .get_webview_panel(label)
        .map_err(|_| tauri::Error::FailedToReceiveMessage)?;
    panel.hide();
    let native = panel.as_panel();
    log::info!(
        "panel handoff: label={label}, key={}, visible={}",
        native.isKeyWindow(),
        native.isVisible()
    );
    if native.isKeyWindow() || native.isVisible() {
        return Err(tauri::Error::FailedToReceiveMessage);
    }
    Ok(())
}

pub(crate) fn panels_are_hidden(app: &tauri::AppHandle) -> bool {
    use tauri_nspanel::ManagerExt;
    [super::MAIN_LABEL, super::QUICK_LABEL]
        .into_iter()
        .all(|label| {
            app.get_webview_panel(label)
                .is_ok_and(|panel| !panel.as_panel().isKeyWindow() && !panel.as_panel().isVisible())
        })
}

pub(super) fn quicklook_has_focus() -> bool {
    use objc::{class, msg_send};
    unsafe {
        let exists: bool = msg_send![class!(QLPreviewPanel), sharedPreviewPanelExists];
        if !exists {
            return false;
        }
        let panel: *mut objc::runtime::Object =
            msg_send![class!(QLPreviewPanel), sharedPreviewPanel];
        let visible: bool = msg_send![panel, isVisible];
        let key: bool = msg_send![panel, isKeyWindow];
        visible && key
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn quick_frame(
    visible: Rect,
    anchor_x: f64,
    preferred_content: (f64, f64),
    frame_inset: (f64, f64),
) -> Option<Rect> {
    let available_width = visible.width - QUICK_MARGIN * 2.0 - frame_inset.0;
    let available_height = visible.height - QUICK_MARGIN * 2.0 - frame_inset.1;
    if available_width <= 0.0 || available_height <= 0.0 {
        return None;
    }
    let width = preferred_content.0.min(available_width) + frame_inset.0;
    let height = preferred_content.1.min(available_height) + frame_inset.1;
    let min_x = visible.x + QUICK_MARGIN;
    let max_x = visible.x + visible.width - QUICK_MARGIN - width;
    Some(Rect {
        x: (anchor_x - width / 2.0).clamp(min_x, max_x),
        y: visible.y + visible.height - QUICK_MARGIN - height,
        width,
        height,
    })
}

fn centered_frame(visible: Rect, size: (f64, f64)) -> Rect {
    Rect {
        x: visible.x + (visible.width - size.0) / 2.0,
        y: visible.y + (visible.height - size.1) / 2.0,
        width: size.0,
        height: size.1,
    }
}

fn cursor_screen(mtm: MainThreadMarker) -> Option<(Retained<NSScreen>, NSPoint)> {
    let mouse = NSEvent::mouseLocation();
    let screens = NSScreen::screens(mtm);
    let screen = screens
        .iter()
        .find(|screen| {
            let frame = screen.frame();
            mouse.x >= frame.origin.x
                && mouse.x < frame.origin.x + frame.size.width
                && mouse.y >= frame.origin.y
                && mouse.y < frame.origin.y + frame.size.height
        })
        .or_else(|| NSScreen::mainScreen(mtm))
        .or_else(|| screens.iter().next())?;
    Some((screen, mouse))
}

pub(super) fn layout_quick_panel(app: &tauri::AppHandle, label: &str) -> bool {
    use tauri_nspanel::{
        objc2_foundation::{NSRect, NSSize},
        ManagerExt,
    };

    let Some(mtm) = MainThreadMarker::new() else {
        log::warn!("layout_quick_panel: not running on the main thread");
        return false;
    };
    let Some((screen, mouse)) = cursor_screen(mtm) else {
        log::warn!("layout_quick_panel: no screen is available");
        return false;
    };
    let Ok(panel) = app.get_webview_panel(label) else {
        log::warn!("layout_quick_panel: {label} panel is unavailable");
        return false;
    };
    let panel = panel.as_panel();
    let preferred_content = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(super::QUICK_WINDOW_WIDTH, super::QUICK_WINDOW_HEIGHT),
    );
    let preferred_frame = panel.frameRectForContentRect(preferred_content);
    let visible = screen.visibleFrame();
    let Some(frame) = quick_frame(
        Rect {
            x: visible.origin.x,
            y: visible.origin.y,
            width: visible.size.width,
            height: visible.size.height,
        },
        mouse.x,
        (super::QUICK_WINDOW_WIDTH, super::QUICK_WINDOW_HEIGHT),
        (
            preferred_frame.size.width - preferred_content.size.width,
            preferred_frame.size.height - preferred_content.size.height,
        ),
    ) else {
        log::warn!("layout_quick_panel: target screen has no usable visible frame");
        return false;
    };
    panel.setFrame_display(
        NSRect::new(
            NSPoint::new(frame.x, frame.y),
            NSSize::new(frame.width, frame.height),
        ),
        false,
    );
    true
}

pub(super) fn layout_main_panel(app: &tauri::AppHandle, label: &str) -> bool {
    use tauri_nspanel::{
        objc2_foundation::{NSRect, NSSize},
        ManagerExt,
    };

    let Some(mtm) = MainThreadMarker::new() else {
        log::warn!("layout_main_panel: not running on the main thread");
        return false;
    };
    let Some((screen, _)) = cursor_screen(mtm) else {
        log::warn!("layout_main_panel: no screen is available");
        return false;
    };
    let Ok(panel) = app.get_webview_panel(label) else {
        log::warn!("layout_main_panel: {label} panel is unavailable");
        return false;
    };
    let visible = screen.visibleFrame();
    let (content_width, content_height) =
        super::panel_content_size(visible.size.width, visible.size.height);
    let content_rect = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(
            content_width + super::SHADOW_INSET * 2.0,
            content_height + super::SHADOW_INSET * 2.0,
        ),
    );
    let frame = panel.as_panel().frameRectForContentRect(content_rect);
    let target = centered_frame(
        Rect {
            x: visible.origin.x,
            y: visible.origin.y,
            width: visible.size.width,
            height: visible.size.height,
        },
        (frame.size.width, frame.size.height),
    );
    panel.as_panel().setFrame_display(
        NSRect::new(
            NSPoint::new(target.x, target.y),
            NSSize::new(target.width, target.height),
        ),
        false,
    );
    true
}

pub(super) fn show_as_panel(app: &tauri::AppHandle, label: &str) -> bool {
    use tauri_nspanel::{CollectionBehavior, ManagerExt};

    install_deactivation_observer(app);
    let Ok(panel) = app.get_webview_panel(label) else {
        return false;
    };
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .can_join_all_spaces()
            .full_screen_auxiliary()
            .into(),
    );
    // Keep the external application active while this panel receives keyboard input.
    panel.show_and_make_key();
    let focused = panel.as_panel().isKeyWindow();
    log::info!(
        "show_panel: label={label}, key={focused}, app_active={}",
        application_is_active()
    );
    focused
}

pub(super) fn hide_preview(app: &tauri::AppHandle) {
    use tauri_plugin_quicklook::QuicklookExt;

    if let Err(error) = app.quicklook().queue_hide() {
        log::warn!("failed to hide Quick Look with panel: {error}");
    }
}

pub(crate) fn prepare_quicklook_level(app: &tauri::AppHandle) -> tauri::Result<()> {
    use objc::{class, msg_send};
    use tauri_nspanel::ManagerExt;

    let app_for_main = app.clone();
    app.run_on_main_thread(move || {
        let panel_level = [super::MAIN_LABEL, super::QUICK_LABEL]
            .into_iter()
            .filter_map(|label| app_for_main.get_webview_panel(label).ok())
            .map(|panel| panel.as_panel().level())
            .max()
            .unwrap_or(0);
        // Ordering front only affects windows within the same level.
        // Quick Look must also sit above our always-on-top clipboard panels.
        unsafe {
            let preview: *mut objc::runtime::Object =
                msg_send![class!(QLPreviewPanel), sharedPreviewPanel];
            // Quick Look shares the nonactivating clipboard session. A regular
            // panel cannot reliably take key focus while our app is inactive.
            let style: usize = msg_send![preview, styleMask];
            let _: () = msg_send![preview, setStyleMask: style | (1usize << 7)];
            let supported: bool =
                msg_send![preview, respondsToSelector: sel!(_setPreventsActivation:)];
            if supported {
                let _: () = msg_send![preview, _setPreventsActivation: true];
            }
            let _: () = msg_send![preview, setHidesOnDeactivate: false];
            let _: () = msg_send![preview, setLevel: panel_level + 1];
            let actual_level: isize = msg_send![preview, level];
            debug_assert!(actual_level > panel_level);
        }
    })
}

pub(crate) fn application_is_active() -> bool {
    use objc::{class, msg_send};
    unsafe {
        let application: *mut objc::runtime::Object =
            msg_send![class!(NSApplication), sharedApplication];
        msg_send![application, isActive]
    }
}

pub(crate) fn install_deactivation_observer(app: &tauri::AppHandle) {
    use block2::RcBlock;
    use objc2_foundation::{
        NSNotification, NSNotificationCenter, NSNotificationName, NSOperationQueue,
    };
    use std::sync::Once;

    static INSTALL: Once = Once::new();
    let app = app.clone();
    INSTALL.call_once(|| {
        // Nonactivating panels never activate NSApplication, so clicking the
        // already-active app (or Finder desktop) need not produce a resign-active
        // notification. Global mouse monitors receive OTHER apps' events only:
        // clicks within our panels and Quick Look must not dismiss the session.
        let mouse_app = app.clone();
        let mouse = RcBlock::new(move |_event: std::ptr::NonNull<NSEvent>| {
            dismiss_panels(&mouse_app);
        });
        use tauri_nspanel::objc2_app_kit::NSEventMask;
        if let Some(monitor) = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
            NSEventMask::LeftMouseDown | NSEventMask::RightMouseDown | NSEventMask::OtherMouseDown,
            &mouse,
        ) {
            // One monitor for the process lifetime, installed once on the main
            // thread. No keyboard monitoring or suppression of the external click.
            std::mem::forget(monitor);
        } else {
            log::warn!("external-click monitor unavailable");
        }
        let name = NSNotificationName::from_str("NSApplicationDidResignActiveNotification");
        let block = RcBlock::new(move |_notification: std::ptr::NonNull<NSNotification>| {
            dismiss_panels(&app);
        });
        let center = NSNotificationCenter::defaultCenter();
        let observer = unsafe {
            center.addObserverForName_object_queue_usingBlock(
                Some(&name),
                None,
                Some(&NSOperationQueue::mainQueue()),
                &block,
            )
        };
        // NSNotificationCenter retains the observer for the application's lifetime.
        drop(observer);
    });
}

fn dismiss_panels(app: &tauri::AppHandle) {
    use tauri::Manager;
    let lifecycle = app.state::<super::PanelLifecycleState>();
    for label in [super::MAIN_LABEL, super::QUICK_LABEL] {
        if lifecycle.is_shown(label) {
            // hide_panel also hides Quick Look and clears preview state.
            if let Err(error) = super::hide_panel(app, label, super::HideReason::Blur) {
                log::warn!("failed to dismiss {label} after external interaction: {error}");
            }
        }
    }
}

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
    use super::{centered_frame, quick_frame, Rect};

    #[test]
    fn quick_frame_uses_preferred_size_and_clamps_x() {
        let visible = Rect {
            x: -1920.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        assert_eq!(
            quick_frame(visible, -1915.0, (360.0, 604.0), (0.0, 0.0)),
            Some(Rect {
                x: -1914.0,
                y: 470.0,
                width: 360.0,
                height: 604.0,
            })
        );
    }

    #[test]
    fn quick_frame_shrinks_to_small_visible_frame() {
        assert_eq!(
            quick_frame(
                Rect {
                    x: 0.0,
                    y: 24.0,
                    width: 320.0,
                    height: 500.0,
                },
                160.0,
                (360.0, 604.0),
                (0.0, 0.0),
            ),
            Some(Rect {
                x: 6.0,
                y: 30.0,
                width: 308.0,
                height: 488.0,
            })
        );
    }

    #[test]
    fn quick_frame_rejects_unusable_visible_frame() {
        assert_eq!(
            quick_frame(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 12.0,
                    height: 12.0,
                },
                6.0,
                (360.0, 604.0),
                (0.0, 0.0),
            ),
            None
        );
    }

    #[test]
    fn main_frame_centers_inside_nonzero_visible_frame() {
        assert_eq!(
            centered_frame(
                Rect {
                    x: 1440.0,
                    y: 24.0,
                    width: 1920.0,
                    height: 1056.0,
                },
                (840.0, 640.0),
            ),
            Rect {
                x: 1980.0,
                y: 232.0,
                width: 840.0,
                height: 640.0,
            }
        );
    }
}
