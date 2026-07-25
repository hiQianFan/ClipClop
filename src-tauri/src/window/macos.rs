use objc::{sel, sel_impl};

pub(super) fn show_as_panel(app: &tauri::AppHandle) -> bool {
    use objc::{class, msg_send};
    use tauri_nspanel::ManagerExt;

    let Ok(panel) = app.get_webview_panel("main") else {
        return false;
    };
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
