#![allow(unexpected_cfgs)] // objc 0.2 macros probe a legacy `cargo-clippy` cfg.

pub mod clipboard;
pub mod clips;
pub mod commands;
pub mod error;
pub mod paste;
pub mod settings;
pub mod state;
pub mod storage;
pub mod window;

use commands::{
    clear_history, copy_clip, delete_clip, get_clip, get_clip_asset, get_clip_file_asset,
    get_clip_thumbnail, get_settings, get_source_app_icon, hide_panel, list_clips, open_clip,
    open_clip_file, paste_clip, quit_app, record_update_check, toggle_clip_preview,
    update_settings,
};
use settings::{validate_hotkey, Settings, DEFAULT_HOTKEY, SETTINGS_KEY};
use state::AppState;
use storage::Database;
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show_panel(app);
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
            let mut startup_settings: Settings =
                database.get_setting(SETTINGS_KEY)?.unwrap_or_default();
            if let Err(message) = validate_hotkey(&startup_settings.hotkey) {
                eprintln!(
                    "stored global shortcut is invalid; restoring the platform default: {message}"
                );
                startup_settings.hotkey = DEFAULT_HOTKEY.into();
                database.set_setting(SETTINGS_KEY, &startup_settings)?;
            }
            let startup_hotkey = startup_settings.hotkey.clone();
            app.manage(AppState::new(database));
            app.manage(window::PreviewState::default());
            clipboard::start_watcher(app.handle().clone())?;

            if let Err(error) = register_panel_hotkey(app.handle(), &startup_hotkey) {
                eprintln!("failed to register stored global shortcut: {error}");
                if startup_hotkey != DEFAULT_HOTKEY {
                    if let Err(default_error) = register_panel_hotkey(app.handle(), DEFAULT_HOTKEY)
                    {
                        eprintln!(
                            "failed to register the default global shortcut: {default_error}"
                        );
                    } else {
                        startup_settings.hotkey = DEFAULT_HOTKEY.into();
                        app.state::<AppState>()
                            .database
                            .set_setting(SETTINGS_KEY, &startup_settings)?;
                    }
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                {
                    use tauri_nspanel::{StyleMask, WebviewWindowExt};
                    let panel = window.to_panel::<ClipboardPanel>()?;
                    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
                }
                window::resize_panel_for_monitor(&window);
                let _ = window.center();
                let panel = window.clone();
                let app = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(focused) = event {
                        let preview = app.state::<window::PreviewState>();
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
            window::show_panel(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_clips,
            get_clip,
            get_clip_asset,
            get_clip_file_asset,
            get_clip_thumbnail,
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
            record_update_check,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn register_panel_hotkey(
    app: &tauri::AppHandle,
    hotkey: &str,
) -> Result<(), tauri_plugin_global_shortcut::Error> {
    app.global_shortcut()
        .on_shortcut(hotkey, |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                window::toggle_panel(app);
            }
        })
}
