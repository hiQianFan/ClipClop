use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use crate::settings::{LanguagePreference, Settings};

const TRAY_ID: &str = "main-tray";
const OPEN_ID: &str = "tray-open";
const QUIT_ID: &str = "tray-quit";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MenuLabels {
    open: &'static str,
    quit: &'static str,
}

pub(crate) fn install(app: &tauri::App, settings: &Settings) -> tauri::Result<()> {
    let labels = menu_labels(settings.language, sys_locale::get_locale().as_deref());
    let open = MenuItem::with_id(app, OPEN_ID, labels.open, true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT_ID, labels.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &separator, &quit])?;

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
            QUIT_ID => app.exit(0),
            _ => {}
        })
        .build(app)?;

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
            open: "打开 ClipClop",
            quit: "退出 ClipClop",
        }
    } else {
        MenuLabels {
            open: "Open ClipClop",
            quit: "Quit ClipClop",
        }
    }
}

#[cfg(target_os = "macos")]
fn platform_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/tray/macos/trayTemplate@2x.png"))
}

#[cfg(target_os = "windows")]
fn platform_icon() -> tauri::Result<Image<'static>> {
    let light_taskbar = windows_uses_light_taskbar();
    if light_taskbar.is_none() {
        log::warn!("failed to read SystemUsesLightTheme; using the light glyph fallback");
    }
    Image::from_bytes(windows_icon_bytes(light_taskbar))
}

#[cfg(any(target_os = "windows", test))]
fn windows_icon_bytes(light_taskbar: Option<bool>) -> &'static [u8] {
    match light_taskbar {
        Some(true) => include_bytes!("../icons/tray/windows/tray-light-32.png"),
        Some(false) | None => include_bytes!("../icons/tray/windows/tray-dark-32.png"),
    }
}

#[cfg(target_os = "windows")]
fn windows_uses_light_taskbar() -> Option<bool> {
    use std::{ffi::c_void, mem::size_of};
    use windows_sys::Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{RegGetValueW, HKEY_CURRENT_USER, REG_VALUE_TYPE, RRF_RT_REG_DWORD},
    };

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let key = wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let value_name = wide("SystemUsesLightTheme");
    let mut value = 0_u32;
    let mut value_type: REG_VALUE_TYPE = 0;
    let mut value_size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value_name.as_ptr(),
            RRF_RT_REG_DWORD,
            &mut value_type,
            (&mut value as *mut u32).cast::<c_void>(),
            &mut value_size,
        )
    };

    (status == ERROR_SUCCESS && value_size == size_of::<u32>() as u32).then_some(value != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_language_preferences_ignore_system_locale() {
        assert_eq!(
            menu_labels(LanguagePreference::ChineseSimplified, Some("en-US")),
            MenuLabels {
                open: "打开 ClipClop",
                quit: "退出 ClipClop"
            }
        );
        assert_eq!(
            menu_labels(LanguagePreference::English, Some("zh-CN")),
            MenuLabels {
                open: "Open ClipClop",
                quit: "Quit ClipClop"
            }
        );
    }

    #[test]
    fn system_language_supports_chinese_locale_variants() {
        for locale in ["zh-CN", "zh-Hans-CN", "ZH_tw"] {
            assert_eq!(
                menu_labels(LanguagePreference::System, Some(locale)),
                MenuLabels {
                    open: "打开 ClipClop",
                    quit: "退出 ClipClop"
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
                    open: "Open ClipClop",
                    quit: "Quit ClipClop"
                }
            );
        }
    }

    #[test]
    fn windows_icon_selection_covers_light_dark_and_fallback() {
        let light = include_bytes!("../icons/tray/windows/tray-light-32.png").as_slice();
        let dark = include_bytes!("../icons/tray/windows/tray-dark-32.png").as_slice();

        assert_eq!(windows_icon_bytes(Some(true)), light);
        assert_eq!(windows_icon_bytes(Some(false)), dark);
        assert_eq!(windows_icon_bytes(None), dark);
        assert_ne!(light, dark);
    }
}
