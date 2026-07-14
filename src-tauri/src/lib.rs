pub mod clipboard;
pub mod clips;
pub mod commands;
pub mod error;
pub mod state;
pub mod storage;

use commands::{
    clear_history, copy_clip, delete_clip, get_clip, get_clip_asset, get_clip_file_asset,
    get_clip_file_thumbnail, get_clip_thumbnail, get_settings, get_source_app_icon, hide_panel,
    ignore_source, list_clips, open_clip, open_clip_file, open_settings, quit_app, update_settings,
    DEFAULT_HOTKEY,
};
use state::AppState;
use storage::Database;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    LogicalSize, Manager, WebviewWindow, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

const SHADOW_INSET: f64 = 20.0;

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

fn show_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        resize_panel_for_monitor(&window);
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
            let _ = window.hide();
        } else {
            show_panel(app);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_panel(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let database = Database::open(&data_dir.join("clipclop.db"))?;
            app.manage(AppState::new(database));
            clipboard::start_watcher(app.handle().clone())?;

            app.global_shortcut()
                .on_shortcut(DEFAULT_HOTKEY, |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_panel(app);
                    }
                })?;

            if let Some(window) = app.get_webview_window("main") {
                resize_panel_for_monitor(&window);
                let _ = window.center();
                let panel = window.clone();
                window.on_window_event(move |event| {
                    if matches!(event, WindowEvent::Focused(false)) {
                        let _ = panel.hide();
                    }
                });
            }

            let show = MenuItem::with_id(app, "show", "打开 ClipClop", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(
                    app.default_window_icon()
                        .expect("bundle icon missing")
                        .clone(),
                )
                .tooltip("ClipClop")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_panel(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
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
            get_source_app_icon,
            hide_panel,
            delete_clip,
            clear_history,
            copy_clip,
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
