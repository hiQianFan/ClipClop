use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use crate::settings::{LanguagePreference, Settings};

const TRAY_ID: &str = "main-tray";
const OPEN_ID: &str = "tray-open";
const SETTINGS_ID: &str = "tray-settings";
const QUIT_ID: &str = "tray-quit";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MenuLabels {
    open: &'static str,
    settings: &'static str,
    quit: &'static str,
}

pub(crate) fn install(app: &tauri::App, settings: &Settings) -> tauri::Result<()> {
    let labels = menu_labels(settings.language, sys_locale::get_locale().as_deref());
    let open = MenuItem::with_id(
        app,
        OPEN_ID,
        labels.open,
        true,
        Some(settings.hotkey.as_str()),
    )?;
    let settings = MenuItem::with_id(
        app,
        SETTINGS_ID,
        labels.settings,
        true,
        Some("CmdOrCtrl+Comma"),
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT_ID, labels.quit, true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(app, &[&open, &settings, &separator, &quit])?;

    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("ClipClop")
        .icon(platform_icon()?);

    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true).show_menu_on_left_click(true);

    #[cfg(target_os = "windows")]
    let builder = builder
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::window::show_panel(tray.app_handle());
            }
        });

    builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN_ID => crate::window::show_panel(app),
            SETTINGS_ID => crate::window::show_settings(app),
            QUIT_ID => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

pub(crate) fn refresh_menu(app: &tauri::AppHandle, settings: &Settings) -> tauri::Result<()> {
    let labels = menu_labels(settings.language, sys_locale::get_locale().as_deref());
    let open = MenuItem::with_id(
        app,
        OPEN_ID,
        labels.open,
        true,
        Some(settings.hotkey.as_str()),
    )?;
    let settings = MenuItem::with_id(
        app,
        SETTINGS_ID,
        labels.settings,
        true,
        Some("CmdOrCtrl+Comma"),
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT_ID, labels.quit, true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(app, &[&open, &settings, &separator, &quit])?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

fn menu_labels(preference: LanguagePreference, system_locale: Option<&str>) -> MenuLabels {
    let chinese = match preference {
        LanguagePreference::ChineseSimplified => true,
        LanguagePreference::English => false,
        LanguagePreference::System => {
            system_locale.is_some_and(|locale| locale.to_ascii_lowercase().starts_with("zh"))
        }
    };

    if chinese {
        MenuLabels {
            open: "打开",
            settings: "设置",
            quit: "退出",
        }
    } else {
        MenuLabels {
            open: "Open",
            settings: "Settings",
            quit: "Quit",
        }
    }
}

#[cfg(target_os = "macos")]
fn platform_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/tray/macos/trayTemplate@2x.png"))
}

#[cfg(target_os = "windows")]
fn platform_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/tray/windows/tray-32.png"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_language_preferences_ignore_system_locale() {
        assert_eq!(
            menu_labels(LanguagePreference::ChineseSimplified, Some("en-US")),
            MenuLabels {
                open: "打开",
                settings: "设置",
                quit: "退出"
            }
        );
        assert_eq!(
            menu_labels(LanguagePreference::English, Some("zh-CN")),
            MenuLabels {
                open: "Open",
                settings: "Settings",
                quit: "Quit"
            }
        );
    }

    #[test]
    fn system_language_supports_chinese_locale_variants() {
        for locale in ["zh-CN", "zh-Hans-CN", "ZH_tw"] {
            assert_eq!(
                menu_labels(LanguagePreference::System, Some(locale)),
                MenuLabels {
                    open: "打开",
                    settings: "设置",
                    quit: "退出"
                }
            );
        }
    }

    #[test]
    fn system_language_defaults_to_english() {
        for locale in [Some("en-US"), Some("ja-JP"), None] {
            assert_eq!(
                menu_labels(LanguagePreference::System, locale),
                MenuLabels {
                    open: "Open",
                    settings: "Settings",
                    quit: "Quit"
                }
            );
        }
    }
}
