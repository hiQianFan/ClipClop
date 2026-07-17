#![allow(unexpected_cfgs)] // objc 0.2 macros probe a legacy `cargo-clippy` cfg.

pub mod clipboard;
pub mod clips;
pub mod commands;
pub mod error;
pub mod paste;
pub mod state;
pub mod storage;

use commands::{
    clear_history, copy_clip, delete_clip, get_clip, get_clip_asset, get_clip_file_asset,
    get_clip_file_thumbnail, get_clip_thumbnail, get_settings, get_source_app_icon, hide_panel,
    ignore_source, list_clips, open_clip, open_clip_file, open_settings, paste_clip, quit_app,
    toggle_clip_preview, update_settings, DEFAULT_HOTKEY,
};
#[cfg(target_os = "macos")]
use objc::{sel, sel_impl};
use state::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use storage::Database;
use tauri::{Emitter, LogicalSize, Manager, WebviewWindow, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg(target_os = "macos")]
unsafe extern "C" fn handle_quicklook_key(
    _delegate: &objc::runtime::Object,
    _selector: objc::runtime::Sel,
    panel: *mut objc::runtime::Object,
    event: *mut objc::runtime::Object,
) -> bool {
    // NSEventTypeKeyDown = 10. The key-up from the Space press that opened
    // Quick Look is delivered after the panel becomes key; ignoring it keeps
    // a short tap from immediately closing the preview.
    let event_type: u64 = unsafe { objc::msg_send![event, type] };
    let key_code: u16 = unsafe { objc::msg_send![event, keyCode] };
    if event_type == 10 && matches!(key_code, 49 | 53) {
        // The upstream delegate leaves every event unhandled. Consume Space
        // and Escape here so the system preview can be dismissed while it is
        // the key window; focus then returns to the unchanged clipboard row.
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
        eprintln!("Quick Look delegate class is unavailable");
        return;
    };
    let Some(method) = class.instance_method(sel!(previewPanel:handleEvent:)) else {
        eprintln!("Quick Look delegate event handler is unavailable");
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

#[cfg(target_os = "macos")]
tauri_nspanel::tauri_panel! {
    panel!(ClipboardPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

const SHADOW_INSET: f64 = 20.0;

#[derive(Default)]
pub struct PreviewState {
    active: AtomicBool,
}

impl PreviewState {
    pub(crate) fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }

    fn is_active(&self) -> bool {
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

fn resize_panel_for_monitor(window: &WebviewWindow) {
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

fn show_panel(app: &tauri::AppHandle) {
    app.state::<AppState>().paste.capture_target();
    if let Some(window) = app.get_webview_window("main") {
        resize_panel_for_monitor(&window);
        let _ = window.center();
        let _ = app.emit("panel_shown", ());
        #[cfg(target_os = "macos")]
        {
            use tauri_nspanel::ManagerExt;
            if let Ok(panel) = app.get_webview_panel("main") {
                // Capture happened above while the target app was still active.
                // ClipClop must now own keyboard focus so Space, arrows and Enter
                // cannot leak into that target. Enter restores the captured app.
                activate_clipclop();
                panel.make_key_and_order_front();
                return;
            }
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_panel(app: &tauri::AppHandle) {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_panel(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build());

    #[cfg(target_os = "macos")]
    let builder = builder
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_quicklook::init());

    builder
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            let data_dir = app.path().app_data_dir()?;
            let database = Database::open(&data_dir.join("clipclop.db"))?;
            app.manage(AppState::new(database));
            app.manage(PreviewState::default());
            clipboard::start_watcher(app.handle().clone())?;

            app.global_shortcut()
                .on_shortcut(DEFAULT_HOTKEY, |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_panel(app);
                    }
                })?;

            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                {
                    use tauri_nspanel::{StyleMask, WebviewWindowExt};
                    let panel = window.to_panel::<ClipboardPanel>()?;
                    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
                }
                resize_panel_for_monitor(&window);
                let _ = window.center();
                let panel = window.clone();
                let app = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(focused) = event {
                        let preview = app.state::<PreviewState>();
                        if *focused {
                            preview.set_active(false);
                        } else if !preview.is_active() {
                            let _ = panel.hide();
                        }
                    }
                });
            }

            // The bundle is an accessory app: launching it should reveal the
            // panel without turning ClipClop into a regular Dock application.
            show_panel(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_clips,
            get_clip,
            get_clip_asset,
            get_clip_file_asset,
            get_clip_thumbnail,
            get_clip_file_thumbnail,
            open_clip,
            open_clip_file,
            toggle_clip_preview,
            get_source_app_icon,
            hide_panel,
            delete_clip,
            clear_history,
            copy_clip,
            paste_clip,
            get_settings,
            update_settings,
            open_settings,
            quit_app,
            ignore_source
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
