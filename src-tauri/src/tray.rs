use tauri::{image::Image, tray::TrayIconBuilder, Manager};

use crate::{
    settings::{Settings, TrayClickAction},
    state::AppState,
};

const TRAY_ID: &str = "main-tray";

pub(crate) fn install(app: &tauri::App, _: &Settings) -> tauri::Result<()> {
    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("ClipClop")
        .icon(platform_icon()?);

    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);

    let builder = builder
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

            let TrayIconEvent::Click {
                rect,
                position,
                button,
                button_state: MouseButtonState::Up,
                ..
            } = event
            else {
                return;
            };
            if button != MouseButton::Left && button != MouseButton::Right {
                return;
            }

            let app = tray.app_handle();
            let action = app
                .state::<AppState>()
                .settings
                .get_stored()
                .map(|settings| settings.tray_click_action)
                .unwrap_or(TrayClickAction::Recent);
            match action {
                TrayClickAction::Recent => {
                    let anchor = match (rect.position, rect.size) {
                        (tauri::Position::Physical(position), tauri::Size::Physical(size)) => {
                            tauri::PhysicalPosition::new(
                                f64::from(position.x) + f64::from(size.width) / 2.0,
                                f64::from(position.y) + f64::from(size.height) / 2.0,
                            )
                        }
                        _ => position,
                    };
                    crate::window::toggle_quick_panel(app, anchor);
                }
                TrayClickAction::History => crate::window::toggle_full_panel(app),
            }
        });

    builder.build(app)?;

    Ok(())
}

pub(crate) fn refresh_menu(_: &tauri::AppHandle, _: &Settings) -> tauri::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn platform_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/tray/macos/trayTemplate@2x.png"))
}

#[cfg(target_os = "windows")]
fn platform_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/tray/windows/tray-32.png"))
}
