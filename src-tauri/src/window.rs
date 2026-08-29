mod lifecycle;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};

use serde::Serialize;
#[cfg(not(target_os = "macos"))]
use tauri::PhysicalSize;
use tauri::{Emitter, LogicalSize, Manager, PhysicalPosition, WebviewWindow};

use crate::state::AppState;

pub(crate) use lifecycle::PanelLifecycleState;

pub(crate) const MAIN_LABEL: &str = "main";
pub(crate) const QUICK_LABEL: &str = "quick";

const SHADOW_INSET: f64 = 20.0;
const PANEL_CONTENT_WIDTH: f64 = 800.0;
const PANEL_CONTENT_HEIGHT: f64 = 600.0;
const QUICK_WINDOW_WIDTH: f64 = 360.0;
const QUICK_WINDOW_HEIGHT: f64 = 604.0;
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

#[derive(Default)]
pub struct QuickSelectionState(Mutex<Option<String>>);

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct MainPanelRequest {
    selected_id: Option<String>,
    settings: bool,
}

impl QuickSelectionState {
    pub(crate) fn set(&self, id: Option<String>) {
        *self.0.lock().unwrap_or_else(|state| state.into_inner()) = id;
    }

    pub(crate) fn get(&self) -> Option<String> {
        self.0
            .lock()
            .unwrap_or_else(|state| state.into_inner())
            .clone()
    }
}

fn panel_content_size(work_area_width: f64, work_area_height: f64) -> (f64, f64) {
    let max_width = (work_area_width - SHADOW_INSET * 2.0).max(0.0);
    let max_height = (work_area_height - SHADOW_INSET * 2.0).max(0.0);
    (
        PANEL_CONTENT_WIDTH.min(max_width),
        PANEL_CONTENT_HEIGHT.min(max_height),
    )
}

pub(crate) fn resize_panel_for_monitor(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };
    let work_area = monitor
        .work_area()
        .size
        .to_logical::<f64>(monitor.scale_factor());
    resize_panel(window, work_area.width, work_area.height);
}

fn resize_panel(window: &WebviewWindow, work_area_width: f64, work_area_height: f64) {
    let (content_width, content_height) = panel_content_size(work_area_width, work_area_height);
    let target = LogicalSize::new(
        content_width + SHADOW_INSET * 2.0,
        content_height + SHADOW_INSET * 2.0,
    );
    let current = window
        .inner_size()
        .map(|size| size.to_logical::<f64>(window.scale_factor().unwrap_or(1.0)));
    if current.is_ok_and(|size| size == target) {
        return;
    }
    let _ = window.set_size(target);
}

pub(crate) fn show_panel(app: &tauri::AppHandle) {
    show_full_panel(app);
}

pub(crate) fn show_full_panel(app: &tauri::AppHandle) {
    show(app, MAIN_LABEL, None);
    emit_main_request(app, MainPanelRequest::default());
}

pub(crate) fn open_full_panel(app: &tauri::AppHandle, selected_id: Option<String>, settings: bool) {
    if app.state::<PanelLifecycleState>().is_shown(QUICK_LABEL) {
        let _ = hide_panel(app, QUICK_LABEL, HideReason::Shortcut);
    }
    show(app, MAIN_LABEL, None);
    emit_main_request(
        app,
        MainPanelRequest {
            selected_id,
            settings,
        },
    );
}

pub(crate) fn toggle_quick_panel(app: &tauri::AppHandle, anchor: PhysicalPosition<f64>) {
    toggle_from_tray(app, QUICK_LABEL, Some(anchor));
}

pub(crate) fn toggle_full_panel(app: &tauri::AppHandle) {
    toggle_from_tray(app, MAIN_LABEL, None);
}

fn toggle_from_tray(
    app: &tauri::AppHandle,
    label: &'static str,
    anchor: Option<PhysicalPosition<f64>>,
) {
    if app.state::<PanelLifecycleState>().is_shown(label) {
        if let Err(error) = hide_panel(app, label, HideReason::Shortcut) {
            log::warn!("failed to hide panel from tray: {error}");
        }
    } else {
        if label == MAIN_LABEL {
            show_full_panel(app);
        } else {
            show(app, label, anchor);
        }
    }
}

fn show(app: &tauri::AppHandle, label: &'static str, _anchor: Option<PhysicalPosition<f64>>) {
    let lifecycle = app.state::<PanelLifecycleState>();
    if !lifecycle.is_shown(MAIN_LABEL) && !lifecycle.is_shown(QUICK_LABEL) {
        app.state::<AppState>().paste.capture_target();
    }
    let Some(window) = app.get_webview_window(label) else {
        log::warn!("show_panel: {label} window is unavailable");
        return;
    };

    lifecycle.begin_show(label, window.is_focused().unwrap_or(false));

    if label == QUICK_LABEL {
        #[cfg(target_os = "macos")]
        if !macos::layout_quick_panel(app, label) {
            log::warn!("show_panel: keeping the previous quick panel frame");
        }
        #[cfg(not(target_os = "macos"))]
        if let Some(anchor) = _anchor {
            layout_quick_panel(&window, anchor);
        }
    } else {
        #[cfg(target_os = "macos")]
        if !macos::layout_main_panel(app, label) {
            log::warn!("show_panel: falling back to the current main panel layout");
            resize_panel_for_monitor(&window);
            let _ = window.center();
        }
        #[cfg(not(target_os = "macos"))]
        {
            resize_panel_for_monitor(&window);
            let _ = window.center();
        }
    }

    #[cfg(target_os = "macos")]
    if macos::show_as_panel(app, label) {
        lifecycle.mark_focused(label);
        return;
    }

    if let Err(error) = window.show() {
        lifecycle.mark_hidden(label);
        log::error!("show_panel: failed to show window: {error}");
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let outcome = windows::focus_foreground(&window);
        if outcome.has_focus() {
            lifecycle.mark_focused(label);
            log::info!("show_panel: foreground acquired ({outcome:?})");
        } else {
            log::warn!("show_panel: foreground request did not succeed ({outcome:?})");
        }
    }

    #[cfg(not(target_os = "windows"))]
    match window.set_focus() {
        Ok(()) => lifecycle.mark_focused(label),
        Err(error) => log::warn!("show_panel: focus request failed: {error}"),
    }
}

fn emit_main_request(app: &tauri::AppHandle, request: MainPanelRequest) {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        let _ = window.emit("main_panel_shown", request);
    }
}

#[cfg(not(target_os = "macos"))]
fn layout_quick_panel(window: &WebviewWindow, anchor: PhysicalPosition<f64>) {
    let Ok(monitors) = window.available_monitors() else {
        return;
    };
    let Some(monitor) = monitors
        .iter()
        .find(|monitor| monitor_contains_point(monitor.position(), monitor.size(), anchor))
    else {
        return;
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let logical_width = QUICK_WINDOW_WIDTH.min((area.size.width as f64 / scale - 12.0).max(164.0));
    let logical_height =
        QUICK_WINDOW_HEIGHT.min((area.size.height as f64 / scale - 12.0).max(164.0));
    let size = PhysicalSize::new(
        (logical_width * scale).round() as u32,
        (logical_height * scale).round() as u32,
    );
    let area_right = area.position.x + area.size.width as i32;
    let area_bottom = area.position.y + area.size.height as i32;
    let anchor_x = anchor.x.round() as i32;
    let anchor_y = anchor.y.round() as i32;
    let mut x = anchor_x - size.width as i32 / 2;
    let mut y = if anchor_y < area.position.y {
        anchor_y + 6
    } else if anchor_y >= area_bottom {
        anchor_y - size.height as i32 - 6
    } else if anchor_x < area.position.x {
        x = anchor_x + 6;
        anchor_y - size.height as i32 / 2
    } else if anchor_x >= area_right {
        x = anchor_x - size.width as i32 - 6;
        anchor_y - size.height as i32 / 2
    } else if anchor_y < area.position.y + area.size.height as i32 / 2 {
        anchor_y + 6
    } else {
        anchor_y - size.height as i32 - 6
    };
    x = x.clamp(area.position.x, area_right - size.width as i32);
    y = y.clamp(area.position.y, area_bottom - size.height as i32);
    let _ = window.set_position(PhysicalPosition::new(x, y));
    let _ = window.set_size(size);
}

#[cfg(not(target_os = "macos"))]
fn monitor_contains_point(
    position: &PhysicalPosition<i32>,
    size: &PhysicalSize<u32>,
    point: PhysicalPosition<f64>,
) -> bool {
    point.x >= position.x as f64
        && point.x < (position.x + size.width as i32) as f64
        && point.y >= position.y as f64
        && point.y < (position.y + size.height as i32) as f64
}

pub(crate) fn show_panel_on_main_thread(app: &tauri::AppHandle) {
    let handle = app.clone();
    if let Err(error) = app.run_on_main_thread(move || show_panel(&handle)) {
        log::error!("show_panel: failed to dispatch to the main thread: {error}");
    }
}

pub(crate) fn hide_panel(
    app: &tauri::AppHandle,
    label: &str,
    reason: HideReason,
) -> Result<(), tauri::Error> {
    let Some(window) = app.get_webview_window(label) else {
        return Ok(());
    };

    #[cfg(target_os = "macos")]
    macos::hide_preview(app);

    window.hide()?;
    app.state::<PanelLifecycleState>().mark_hidden(label);
    app.state::<PreviewState>().set_active(false);
    log::info!("{label} panel hidden ({reason:?})");
    Ok(())
}

pub(crate) fn toggle_panel(app: &tauri::AppHandle) {
    let lifecycle = app.state::<PanelLifecycleState>();
    if lifecycle.is_shown(QUICK_LABEL) {
        let selected_id = app.state::<QuickSelectionState>().get();
        let _ = hide_panel(app, QUICK_LABEL, HideReason::Shortcut);
        open_full_panel(app, selected_id, false);
    } else if lifecycle.is_shown(MAIN_LABEL) {
        if let Err(error) = hide_panel(app, MAIN_LABEL, HideReason::Shortcut) {
            log::warn!("failed to hide panel from shortcut: {error}");
        }
    } else {
        show_panel(app);
    }
}

pub(crate) fn handle_focus_event(app: &tauri::AppHandle, panel: &WebviewWindow, focused: bool) {
    let lifecycle = app.state::<PanelLifecycleState>();
    let label = panel.label().to_string();
    if focused {
        app.state::<PreviewState>().set_active(false);
        lifecycle.mark_focused(&label);
        log::debug!("panel focus acquired");
        return;
    }

    let Some(token) = lifecycle.begin_blur(&label) else {
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
            if !lifecycle.can_hide(&label, token)
                || app_for_main.state::<PreviewState>().is_active()
            {
                return;
            }
            if panel.is_focused().unwrap_or(false) {
                lifecycle.mark_focused(&label);
                return;
            }
            if panel.is_visible().unwrap_or(false) {
                log::info!("panel remained unfocused after debounce");
                if let Err(error) = hide_panel(&app_for_main, &label, HideReason::Blur) {
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
    fn panel_keeps_one_size_on_normal_displays() {
        assert_eq!(panel_content_size(1000.0, 700.0), (800.0, 600.0));
        assert_eq!(panel_content_size(1440.0, 900.0), (800.0, 600.0));
        assert_eq!(panel_content_size(1920.0, 1080.0), (800.0, 600.0));
    }

    #[test]
    fn panel_never_exceeds_the_monitor_work_area() {
        assert_eq!(panel_content_size(800.0, 560.0), (760.0, 520.0));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn taskbar_anchor_belongs_to_the_full_monitor_bounds() {
        use super::monitor_contains_point;
        use tauri::{PhysicalPosition, PhysicalSize};

        assert!(monitor_contains_point(
            &PhysicalPosition::new(0, 0),
            &PhysicalSize::new(1920, 1080),
            PhysicalPosition::new(1800.0, 1060.0),
        ));
    }
}
