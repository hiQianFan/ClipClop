#![allow(unexpected_cfgs)] // objc 0.2 macros probe a legacy `cargo-clippy` cfg.

pub mod assets;
pub mod clipboard;
pub mod commands;
pub mod error;
pub mod history;
pub mod onboarding;
pub mod paste;
pub mod preview;
pub mod settings;
pub mod state;
pub mod storage;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod tray;
pub mod window;
pub mod workflows;

use commands::{
    clear_history, copy_clip, delete_clip, get_auto_paste_readiness, get_clip, get_clip_asset,
    get_clip_file_asset, get_clip_thumbnail, get_onboarding_state, get_settings,
    get_source_app_icon, hide_panel, open_auto_paste_settings, open_clip_link,
    open_file_preview_settings, open_log_dir, open_release_page, paste_clip, perform_pager_haptic,
    preview_clip, preview_onboarding_example, query_history, quit_app, record_update_check,
    request_auto_paste_access, save_onboarding_state, set_file_preview_enabled,
    set_language_preference, skip_update_version, update_settings,
};
use settings::{validate_hotkey, Settings, DEFAULT_HOTKEY, SETTINGS_KEY};
use state::AppState;
use storage::Database;
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_log::{Target, TargetKind};

// Diagnostic logs are written to the OS-standard per-app log directory
// (macOS: ~/Library/Logs/<identifier>, Windows: %LOCALAPPDATA%\<identifier>\logs).
// Rotate at 5 MB, keeping only the current file so disk growth stays bounded.
// Privacy: only operational events and error text are logged here; clipboard and
// preview payloads are never passed to the logger.
const LOG_ROTATION_BYTES: u128 = 5 * 1024 * 1024;
const AUTOSTART_ARG: &str = "--autostart";

fn is_autostart_launch(args: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
    args.into_iter().any(|arg| arg.as_ref() == AUTOSTART_ARG)
}

fn build_log_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    // A Windows GUI process can outlive the terminal that launched `tauri dev`. Once that
    // terminal closes, fern treats writes to the broken stderr pipe as a fatal logging error
    // and panics the application. The file target is always available and is the canonical
    // diagnostic source on Windows.
    #[cfg(target_os = "windows")]
    let targets = vec![Target::new(TargetKind::LogDir { file_name: None })];
    #[cfg(not(target_os = "windows"))]
    let targets = vec![
        Target::new(TargetKind::LogDir { file_name: None }),
        Target::new(TargetKind::Stderr),
    ];

    tauri_plugin_log::Builder::new()
        .max_file_size(LOG_ROTATION_BYTES)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
        .level(log::LevelFilter::Info)
        .targets(targets)
        .build()
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(build_log_plugin())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !is_autostart_launch(&args) {
                window::show_panel_on_main_thread(app);
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build());

    #[cfg(target_os = "macos")]
    let builder = builder
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_quicklook::init());

    builder
        .setup(|app| {
            // Route panics to the log file. Under windows_subsystem="windows" there is no
            // console, so the default stderr panic output is lost — without this hook a crash
            // leaves nothing in the log beyond the session-start line.
            let default_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                let location = info
                    .location()
                    .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                    .unwrap_or_else(|| "unknown".into());
                let message = info
                    .payload()
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| info.payload().downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "<non-string panic payload>".into());
                log::error!("panic at {location}: {message}");
                default_hook(info);
            }));

            log::info!(
                "ClipClop session start: version={} os={} arch={}",
                app.package_info().version,
                std::env::consts::OS,
                std::env::consts::ARCH
            );

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            let data_dir = app.path().app_data_dir()?;
            let database_existed = data_dir.join("clipclop.db").exists();
            log::info!("opening database at {}", data_dir.display());
            let database = Database::open(&data_dir.join("clipclop.db"))?;
            let mut startup_settings: Settings =
                database.get_setting(SETTINGS_KEY)?.unwrap_or_default();
            if let Err(message) = validate_hotkey(&startup_settings.hotkey) {
                log::warn!(
                    "stored global shortcut is invalid; restoring the platform default: {message:?}"
                );
                startup_settings.hotkey = DEFAULT_HOTKEY.into();
                database.set_setting(SETTINGS_KEY, &startup_settings)?;
            }
            let startup_hotkey = startup_settings.hotkey.clone();
            let app_state = AppState::new(database);
            app_state.onboarding.initialize(database_existed)?;
            app.manage(app_state);
            app.manage(window::PanelLifecycleState::default());
            app.manage(window::PreviewState::default());
            if let Err(error) = workflows::settings_update::reconcile_autostart(
                app.handle(),
                &app.state::<AppState>().settings,
            ) {
                log::error!("failed to reconcile persisted autostart state: {error}");
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            tray::install(app, &startup_settings)?;
            log::info!("starting clipboard watcher");
            let state = app.state::<AppState>();
            workflows::capture::start(
                app.handle().clone(),
                state.history.clone(),
                state.external_preview.clone(),
                state.settings.clone(),
            )?;
            log::info!("registering global shortcut: {startup_hotkey}");

            if let Err(error) = register_panel_hotkey(app.handle(), &startup_hotkey) {
                log::error!("failed to register stored global shortcut: {error}");
                if startup_hotkey != DEFAULT_HOTKEY {
                    if let Err(default_error) = register_panel_hotkey(app.handle(), DEFAULT_HOTKEY)
                    {
                        log::error!(
                            "failed to register the default global shortcut: {default_error}"
                        );
                    } else {
                        startup_settings.hotkey = DEFAULT_HOTKEY.into();
                        app.state::<AppState>()
                            .settings
                            .set_internal(&startup_settings)?;
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
                let panel = window.clone();
                let app = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(focused) = event {
                        window::handle_focus_event(&app, &panel, *focused);
                    }
                });
            }

            // Login startup keeps the window hidden while the tray, clipboard watcher, and
            // global shortcut remain active. A normal user launch still reveals the panel.
            if is_autostart_launch(std::env::args()) {
                log::info!("autostart launch: keeping panel hidden");
            } else {
                log::info!("interactive launch: showing panel");
                window::show_panel(app.handle());
            }
            log::info!("setup complete");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            query_history,
            get_clip,
            get_clip_asset,
            get_clip_file_asset,
            get_clip_thumbnail,
            preview_clip,
            open_clip_link,
            preview_onboarding_example,
            get_source_app_icon,
            hide_panel,
            delete_clip,
            clear_history,
            copy_clip,
            paste_clip,
            get_settings,
            update_settings,
            record_update_check,
            skip_update_version,
            set_file_preview_enabled,
            open_file_preview_settings,
            open_log_dir,
            open_release_page,
            quit_app,
            perform_pager_haptic,
            get_onboarding_state,
            save_onboarding_state,
            get_auto_paste_readiness,
            request_auto_paste_access,
            open_auto_paste_settings,
            set_language_preference
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

#[cfg(test)]
mod tests {
    use super::is_autostart_launch;

    #[test]
    fn detects_only_the_dedicated_autostart_argument() {
        assert!(is_autostart_launch(["clipclop.exe", "--autostart"]));
        assert!(!is_autostart_launch(["clipclop.exe"]));
        assert!(!is_autostart_launch(["clipclop.exe", "--autostart-extra"]));
    }
}
