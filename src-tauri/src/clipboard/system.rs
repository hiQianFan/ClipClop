use std::thread;
#[cfg(target_os = "macos")]
use std::{path::Path, process::Command};

use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use clipboard_rs::{
    common::RustImage, Clipboard, ClipboardContent, ClipboardContext, ClipboardHandler,
    ClipboardWatcher, ClipboardWatcherContext,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

use crate::{
    clips::{ContentType, CopyMode, Flavor, NewClip, SourceApp},
    commands::Settings,
    error::{AppError, AppResult},
    state::AppState,
};

const MAX_CAPTURE_BYTES: usize = 20 * 1024 * 1024;

pub struct SystemClipboard;

impl SystemClipboard {
    pub fn write(flavors: Vec<Flavor>, mode: CopyMode) -> AppResult<()> {
        let context = ClipboardContext::new().map_err(clipboard_error)?;
        let plain = flavors.iter().find(|item| item.format == "text/plain");
        if mode == CopyMode::Plain {
            let text = plain.ok_or_else(|| {
                AppError::Clipboard("this clip does not contain plain text".into())
            })?;
            return context
                .set_text(String::from_utf8_lossy(&text.payload).into_owned())
                .map_err(clipboard_error);
        }

        let mut contents = Vec::new();
        for flavor in flavors {
            match flavor.format.as_str() {
                "text/plain" => contents.push(ClipboardContent::Text(
                    String::from_utf8_lossy(&flavor.payload).into_owned(),
                )),
                "text/html" => contents.push(ClipboardContent::Html(
                    String::from_utf8_lossy(&flavor.payload).into_owned(),
                )),
                "text/rtf" => contents.push(ClipboardContent::Rtf(
                    String::from_utf8_lossy(&flavor.payload).into_owned(),
                )),
                "image/png" => {
                    let image = clipboard_rs::RustImageData::from_bytes(&flavor.payload)
                        .map_err(clipboard_error)?;
                    contents.push(ClipboardContent::Image(image));
                }
                "text/uri-list" => {
                    let files: Vec<String> = serde_json::from_slice(&flavor.payload)?;
                    contents.push(ClipboardContent::Files(files));
                }
                format => contents.push(ClipboardContent::Other(format.into(), flavor.payload)),
            }
        }
        if contents.is_empty() {
            return Err(AppError::Clipboard("clip has no writable flavors".into()));
        }
        context.set(contents).map_err(clipboard_error)
    }

    pub fn preview_asset(flavors: &[Flavor], file_path: Option<&str>) -> AppResult<Option<String>> {
        let png = if let Some(path) = file_path {
            let path = path.strip_prefix("file://").unwrap_or(path);
            clipboard_rs::RustImageData::from_path(path)
                .and_then(|image| image.thumbnail(960, 640))
                .and_then(|image| image.to_png())
                .ok()
                .map(|buffer| buffer.get_bytes().to_vec())
        } else {
            flavors
                .iter()
                .find(|item| item.format == "image/png")
                .map(|flavor| flavor.payload.clone())
        };
        Ok(png.map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes))))
    }

    pub fn thumbnail_asset(
        flavors: &[Flavor],
        file_path: Option<&str>,
    ) -> AppResult<Option<String>> {
        let image = if let Some(path) = file_path {
            let path = path.strip_prefix("file://").unwrap_or(path);
            clipboard_rs::RustImageData::from_path(path)
                .and_then(|image| image.thumbnail(56, 56))
                .and_then(|image| image.to_png())
                .ok()
        } else if let Some(flavor) = flavors.iter().find(|item| item.format == "image/png") {
            clipboard_rs::RustImageData::from_bytes(&flavor.payload)
                .and_then(|image| image.thumbnail(56, 56))
                .and_then(|image| image.to_png())
                .ok()
        } else {
            None
        };
        Ok(image.map(|png| format!("data:image/png;base64,{}", STANDARD.encode(png.get_bytes()))))
    }

    pub fn source_app_icon(app_id: &str) -> Option<String> {
        source_app_icon(app_id)
            .map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
    }
}

#[cfg(target_os = "macos")]
fn source_app_icon(app_id: &str) -> Option<Vec<u8>> {
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
    let output_path =
        std::env::temp_dir().join(format!("clipclop-icon-{}.png", uuid::Uuid::now_v7()));
    let output = Command::new("/usr/bin/sips")
        .args(["-s", "format", "png"])
        .arg(&icon)
        .arg("--out")
        .arg(&output_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes = std::fs::read(&output_path).ok();
    let _ = std::fs::remove_file(output_path);
    bytes
}

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
fn source_app_icon(_app_id: &str) -> Option<Vec<u8>> {
    None
}

#[cfg(target_os = "windows")]
fn source_app_icon(app_id: &str) -> Option<Vec<u8>> {
    let output_path =
        std::env::temp_dir().join(format!("clipclop-icon-{}.png", uuid::Uuid::now_v7()));
    let escaped_app = app_id.replace('\'', "''");
    let escaped_output = output_path.to_string_lossy().replace('\'', "''");
    let script = format!(
        "Add-Type -AssemblyName System.Drawing; $i=[System.Drawing.Icon]::ExtractAssociatedIcon('{escaped_app}'); if ($i) {{ $i.ToBitmap().Save('{escaped_output}', [System.Drawing.Imaging.ImageFormat]::Png); $i.Dispose() }}"
    );
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes = std::fs::read(&output_path).ok();
    let _ = std::fs::remove_file(output_path);
    bytes
}

struct CaptureHandler {
    app: AppHandle,
    clipboard: ClipboardContext,
}

impl ClipboardHandler for CaptureHandler {
    fn on_clipboard_change(&mut self) {
        if let Err(error) = self.capture() {
            eprintln!("clipboard capture failed: {error}");
        }
    }
}

impl CaptureHandler {
    fn capture(&self) -> AppResult<()> {
        let state = self.app.state::<AppState>();
        let settings: Settings = state.database.get_setting("app")?.unwrap_or_default();
        state.clips.prune(settings.retention_days)?;
        let Some(mut clip) = read_clip(&self.clipboard)? else {
            return Ok(());
        };
        clip.source_app = active_win_pos_rs::get_active_window()
            .ok()
            .filter(|window| window.app_name != "ClipClop")
            .map(|window| SourceApp {
                id: window.process_path.to_string_lossy().into_owned(),
                name: window.app_name,
            });
        if clip
            .source_app
            .as_ref()
            .is_some_and(|source| settings.ignored_apps.contains(&source.id))
        {
            return Ok(());
        }
        if let Some(id) = state.clips.capture(&clip)? {
            let _ = self.app.emit("clips_changed", json!({ "latest_id": id }));
        }
        Ok(())
    }
}

pub fn start_watcher(app: AppHandle) -> AppResult<()> {
    let handler = CaptureHandler {
        app,
        clipboard: ClipboardContext::new().map_err(clipboard_error)?,
    };
    let mut watcher = ClipboardWatcherContext::new().map_err(clipboard_error)?;
    watcher.add_handler(handler);
    thread::Builder::new()
        .name("clipclop-clipboard".into())
        .spawn(move || watcher.start_watch())
        .map_err(|error| AppError::Platform(error.to_string()))?;
    Ok(())
}

fn read_clip(context: &ClipboardContext) -> AppResult<Option<NewClip>> {
    let mut flavors = Vec::new();
    let files = context.get_files().ok().filter(|items| !items.is_empty());
    let image = context.get_image().ok();
    let text = context.get_text().ok().filter(|value| !value.is_empty());
    let html = context.get_html().ok().filter(|value| !value.is_empty());
    let rtf = context
        .get_rich_text()
        .ok()
        .filter(|value| !value.is_empty());

    if let Some(value) = &text {
        flavors.push(Flavor {
            format: "text/plain".into(),
            payload: value.as_bytes().to_vec(),
        });
    }
    if let Some(value) = html {
        flavors.push(Flavor {
            format: "text/html".into(),
            payload: value.into_bytes(),
        });
    }
    if let Some(value) = rtf {
        flavors.push(Flavor {
            format: "text/rtf".into(),
            payload: value.into_bytes(),
        });
    }
    let image_meta = if let Some(image) = image {
        let size = image.get_size();
        let png = image.to_png().map_err(clipboard_error)?;
        flavors.push(Flavor {
            format: "image/png".into(),
            payload: png.get_bytes().to_vec(),
        });
        Some(json!({ "width": size.0, "height": size.1 }))
    } else {
        None
    };
    if let Some(files) = &files {
        flavors.push(Flavor {
            format: "text/uri-list".into(),
            payload: serde_json::to_vec(files)?,
        });
    }
    if flavors.is_empty() {
        return Ok(None);
    }
    let total_bytes: usize = flavors.iter().map(|item| item.payload.len()).sum();
    if total_bytes > MAX_CAPTURE_BYTES {
        return Err(AppError::Validation(format!(
            "clipboard payload exceeds {} MiB",
            MAX_CAPTURE_BYTES / 1024 / 1024
        )));
    }

    let content_type = if files.is_some() {
        ContentType::File
    } else if image_meta.is_some() {
        ContentType::Image
    } else if html_is_present(&flavors) || rtf_is_present(&flavors) {
        ContentType::FormattedText
    } else {
        classify_text(text.as_deref().unwrap_or_default())
    };
    let preview = if content_type == ContentType::Image {
        image_meta
            .as_ref()
            .and_then(|dimensions| {
                Some(format!(
                    "{} × {}",
                    dimensions.get("width")?,
                    dimensions.get("height")?
                ))
            })
            .unwrap_or_default()
    } else {
        preview_for(content_type, text.as_deref(), files.as_deref())
    };
    let mut hasher = Sha256::new();
    for flavor in &flavors {
        hasher.update(flavor.format.as_bytes());
        hasher.update(&flavor.payload);
    }
    let metadata = match files {
        Some(files) => {
            let mut metadata = json!({
            "files": files,
            "file_sizes": files.iter().map(|path| file_size(path)).collect::<Vec<_>>(),
            });
            if let Some(dimensions) = image_meta {
                metadata["image_dimensions"] = dimensions;
            }
            metadata
        }
        None => image_meta.unwrap_or_else(
            || json!({ "char_count": text.as_ref().map(|value| value.chars().count()) }),
        ),
    };
    Ok(Some(NewClip {
        content_type,
        plain_text: text,
        preview,
        source_app: None,
        flavors,
        metadata,
        content_hash: hex::encode(hasher.finalize()),
        created_at: Utc::now(),
    }))
}

fn file_size(path: &str) -> Option<u64> {
    let path = path.strip_prefix("file://").unwrap_or(path);
    std::fs::metadata(path)
        .ok()
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
}

fn html_is_present(flavors: &[Flavor]) -> bool {
    flavors.iter().any(|item| item.format == "text/html")
}

fn rtf_is_present(flavors: &[Flavor]) -> bool {
    flavors.iter().any(|item| item.format == "text/rtf")
}

fn classify_text(text: &str) -> ContentType {
    let trimmed = text.trim();
    if Url::parse(trimmed).is_ok_and(|url| matches!(url.scheme(), "http" | "https")) {
        ContentType::Link
    } else if is_color(trimmed) {
        ContentType::Color
    } else if trimmed.contains('\n')
        && [
            "const ", "let ", "fn ", "def ", "class ", "import ", "SELECT ",
        ]
        .iter()
        .any(|marker| trimmed.contains(marker))
    {
        ContentType::Code
    } else {
        ContentType::Text
    }
}

fn is_color(value: &str) -> bool {
    let hex = value.strip_prefix('#').unwrap_or_default();
    matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|character| character.is_ascii_hexdigit())
}

fn preview_for(content_type: ContentType, text: Option<&str>, files: Option<&[String]>) -> String {
    if content_type == ContentType::File {
        return files
            .and_then(|items| items.first())
            .and_then(|path| std::path::Path::new(path).file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "文件".into());
    }
    text.unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(160)
        .collect()
}

fn clipboard_error(error: impl std::fmt::Display) -> AppError {
    AppError::Clipboard(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_text_hints() {
        assert_eq!(classify_text("https://example.com"), ContentType::Link);
        assert_eq!(classify_text("#1a2B3c"), ContentType::Color);
        assert_eq!(
            classify_text("const x = 1;\nconsole.log(x)"),
            ContentType::Code
        );
        assert_eq!(classify_text("ordinary text"), ContentType::Text);
    }
}
