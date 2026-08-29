use objc::{sel, sel_impl};
use tauri_nspanel::{
    objc2::{rc::Retained, MainThreadMarker},
    objc2_app_kit::{NSEvent, NSScreen},
    objc2_foundation::NSPoint,
};

const QUICK_MARGIN: f64 = 6.0;

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
    use objc::{class, msg_send};
    use tauri_nspanel::{CollectionBehavior, ManagerExt};

    let Ok(panel) = app.get_webview_panel(label) else {
        return false;
    };
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .can_join_all_spaces()
            .full_screen_auxiliary()
            .into(),
    );
    unsafe {
        let application: *mut objc::runtime::Object =
            msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![application, activateIgnoringOtherApps: true];
    }
    panel.make_key_and_order_front();
    true
}

pub(super) fn hide_preview(app: &tauri::AppHandle) {
    use tauri_plugin_quicklook::QuicklookExt;

    if let Err(error) = app.quicklook().queue_hide() {
        log::warn!("failed to hide Quick Look with panel: {error}");
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
