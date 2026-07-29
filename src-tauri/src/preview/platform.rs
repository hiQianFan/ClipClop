use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine};
use clipboard_rs::common::RustImage;

use crate::{error::AppResult, history::Flavor};

pub(super) fn preview_asset(flavors: &[Flavor], file_path: Option<&Path>) -> Option<String> {
    let png = if let Some(path) = file_path {
        clipboard_rs::RustImageData::from_path(&path.to_string_lossy())
            .and_then(|image| image.thumbnail(960, 640))
            .and_then(|image| image.to_png())
            .ok()
            .map(|buffer| buffer.get_bytes().to_vec())
    } else if let Some(flavor) = flavors.iter().find(|item| item.format == "image/png") {
        clipboard_rs::RustImageData::from_bytes(&flavor.payload)
            .and_then(|image| image.thumbnail(960, 640))
            .and_then(|image| image.to_png())
            .ok()
            .map(|buffer| buffer.get_bytes().to_vec())
    } else {
        None
    };
    png.map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

pub(super) fn thumbnail_asset(flavors: &[Flavor]) -> Option<String> {
    flavors
        .iter()
        .find(|item| item.format == "image/png")
        .and_then(|flavor| {
            clipboard_rs::RustImageData::from_bytes(&flavor.payload)
                .and_then(|image| image.thumbnail(56, 56))
                .and_then(|image| image.to_png())
                .ok()
        })
        .map(|png| format!("data:image/png;base64,{}", STANDARD.encode(png.get_bytes())))
}

#[cfg(target_os = "macos")]
pub(super) fn source_app_icon(app_id: &str) -> Option<String> {
    use std::process::Command;

    let executable = Path::new(app_id);
    let app_bundle = executable
        .ancestors()
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))?;
    let resources = app_bundle.join("Contents/Resources");
    let declared_name = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", "Print :CFBundleIconFile"])
        .arg(app_bundle.join("Contents/Info.plist"))
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|name| name.trim().to_owned());
    let declared_icon = declared_name.map(|name| {
        resources.join(if name.ends_with(".icns") {
            name
        } else {
            format!("{name}.icns")
        })
    });
    let icon = declared_icon.filter(|path| path.exists()).or_else(|| {
        std::fs::read_dir(&resources)
            .ok()?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "icns")
            })
    })?;
    convert_icon(
        Command::new("/usr/bin/sips")
            .args(["-s", "format", "png"])
            .arg(&icon)
            .arg("--out"),
    )
}

#[cfg(target_os = "macos")]
fn convert_icon(command: &mut std::process::Command) -> Option<String> {
    let output_path =
        std::env::temp_dir().join(format!("clipclop-icon-{}.png", uuid::Uuid::now_v7()));
    let output = command.arg(&output_path).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes = std::fs::read(&output_path).ok();
    let _ = std::fs::remove_file(output_path);
    bytes.map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

#[cfg(target_os = "windows")]
pub(super) fn source_app_icon(app_id: &str) -> Option<String> {
    use std::os::windows::process::CommandExt;

    let output_path =
        std::env::temp_dir().join(format!("clipclop-icon-{}.png", uuid::Uuid::now_v7()));
    let escaped_app = app_id.replace('\'', "''");
    let escaped_output = output_path.to_string_lossy().replace('\'', "''");
    let script = format!(
        "Add-Type -AssemblyName System.Drawing; $i=[System.Drawing.Icon]::ExtractAssociatedIcon('{escaped_app}'); if ($i) {{ $i.ToBitmap().Save('{escaped_output}', [System.Drawing.Imaging.ImageFormat]::Png); $i.Dispose() }}"
    );
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes = std::fs::read(&output_path).ok();
    let _ = std::fs::remove_file(output_path);
    bytes.map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn source_app_icon(_app_id: &str) -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
pub(super) fn toggle_quicklook(
    app: &tauri::AppHandle,
    state: &crate::window::PreviewState,
    path: &Path,
) -> AppResult<bool> {
    use tauri_plugin_quicklook::{PreviewItem, QuicklookExt};

    crate::window::install_quicklook_key_handler();
    let url = url::Url::from_file_path(path)
        .map_err(|_| crate::error::AppError::Validation("preview path is invalid".into()))?;
    state.set_active(true);
    let result = (|| {
        app.quicklook()
            .set_items(vec![PreviewItem::new(url.to_string(), None)])
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
        app.quicklook()
            .queue_reload_if_dirty()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))?;
        app.quicklook()
            .queue_toggle_visible()
            .map_err(|error| crate::error::AppError::Platform(error.to_string()))
    })();
    if result.is_err() {
        state.set_active(false);
    }
    result?;
    Ok(true)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn toggle_quicklook(
    _app: &tauri::AppHandle,
    _state: &crate::window::PreviewState,
    _path: &Path,
) -> AppResult<bool> {
    Ok(false)
}
