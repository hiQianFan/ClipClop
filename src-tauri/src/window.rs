mod lifecycle;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use tauri::{Emitter, LogicalSize, Manager, WebviewWindow};

use crate::state::AppState;

pub(crate) use lifecycle::PanelLifecycleState;

const SHADOW_INSET: f64 = 20.0;
const BLUR_HIDE_DELAY: Duration = Duration::from_millis(180);

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

#[derive(Clone, Copy, Debug)]
pub(crate) enum HideReason {
    Blur,
    Escape,
    Paste,
    Shortcut,
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

pub(crate) fn show_panel(app: &tauri::AppHandle) {
    app.state::<AppState>().paste.capture_target();
    let Some(window) = app.get_webview_window("main") else {
        log::warn!("show_panel: main window is unavailable");
        return;
    };

    let lifecycle = app.state::<PanelLifecycleState>();
    lifecycle.begin_show(window.is_focused().unwrap_or(false));
    resize_panel_for_monitor(&window);
    let _ = window.center();

    #[cfg(target_os = "macos")]
    if macos::show_as_panel(app) {
        lifecycle.mark_focused();
        emit_panel_shown(app);
        return;
    }

    if let Err(error) = window.show() {
        lifecycle.mark_hidden();
        log::error!("show_panel: failed to show window: {error}");
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let outcome = windows::focus_foreground(&window);
        if outcome.has_focus() {
            lifecycle.mark_focused();
            log::info!("show_panel: foreground acquired ({outcome:?})");
        } else {
            log::warn!("show_panel: foreground request did not succeed ({outcome:?})");
        }
    }

    #[cfg(not(target_os = "windows"))]
    match window.set_focus() {
        Ok(()) => lifecycle.mark_focused(),
        Err(error) => log::warn!("show_panel: focus request failed: {error}"),
    }

    // The frontend resets its browsing session only after the native window has been shown
    // and activation has been attempted, so DOM focus cannot race ahead of native focus.
    emit_panel_shown(app);
}

pub(crate) fn show_panel_on_main_thread(app: &tauri::AppHandle) {
    let handle = app.clone();
    if let Err(error) = app.run_on_main_thread(move || show_panel(&handle)) {
        log::error!("show_panel: failed to dispatch to the main thread: {error}");
    }
}

fn emit_panel_shown(app: &tauri::AppHandle) {
    if let Err(error) = app.emit("panel_shown", ()) {
        log::warn!("show_panel: failed to emit panel_shown: {error}");
    }
}

pub(crate) fn hide_panel(app: &tauri::AppHandle, reason: HideReason) -> Result<(), tauri::Error> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    #[cfg(target_os = "macos")]
    macos::hide_preview(app);

    window.hide()?;
    app.state::<PanelLifecycleState>().mark_hidden();
    app.state::<PreviewState>().set_active(false);
    log::info!("panel hidden ({reason:?})");
    Ok(())
}

pub(crate) fn toggle_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            if let Err(error) = hide_panel(app, HideReason::Shortcut) {
                log::warn!("failed to hide panel from shortcut: {error}");
            }
        } else {
            show_panel(app);
        }
    }
}

pub(crate) fn handle_focus_event(app: &tauri::AppHandle, panel: &WebviewWindow, focused: bool) {
    let lifecycle = app.state::<PanelLifecycleState>();
    if focused {
        app.state::<PreviewState>().set_active(false);
        lifecycle.mark_focused();
        log::debug!("panel focus acquired");
        return;
    }

    let Some(token) = lifecycle.begin_blur() else {
        log::debug!("ignoring blur before panel acquired focus");
        return;
    };

    let app = app.clone();
    let panel = panel.clone();
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(BLUR_HIDE_DELAY);
        let app_for_main = app.clone();
        let _ = app.run_on_main_thread(move || {
            let lifecycle = app_for_main.state::<PanelLifecycleState>();
            if !lifecycle.can_hide(token) || app_for_main.state::<PreviewState>().is_active() {
                return;
            }
            if panel.is_focused().unwrap_or(false) {
                lifecycle.mark_focused();
                return;
            }
            if panel.is_visible().unwrap_or(false) {
                log::info!("panel remained unfocused after debounce");
                if let Err(error) = hide_panel(&app_for_main, HideReason::Blur) {
                    log::warn!("failed to hide unfocused panel: {error}");
                }
            }
        });
    });
}

#[cfg(target_os = "macos")]
pub(crate) use macos::install_quicklook_key_handler;

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
