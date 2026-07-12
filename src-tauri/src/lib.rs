pub mod clipboard;
pub mod clips;
pub mod commands;
pub mod error;
pub mod state;
pub mod storage;

use commands::{
    clear_history, copy_clip, delete_clip, get_clip, get_clip_asset, get_settings, ignore_source,
    list_clips, open_settings, update_settings,
};
use state::AppState;
use storage::Database;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn show_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let database = Database::open(&data_dir.join("clipclop.db"))?;
            app.manage(AppState::new(database));
            clipboard::start_watcher(app.handle().clone())?;

            app.global_shortcut().on_shortcut(
                "CommandOrControl+Shift+C",
                |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        show_panel(app);
                    }
                },
            )?;

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
            delete_clip,
            clear_history,
            copy_clip,
            get_settings,
            update_settings,
            open_settings,
            ignore_source
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
