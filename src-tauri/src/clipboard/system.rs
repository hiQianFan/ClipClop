use std::{thread, time::Duration};

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
    clips::{ClipMetadata, ContentType, Flavor, NewClip},
    error::{AppError, AppResult},
    settings::{Settings, SETTINGS_KEY},
    state::AppState,
};

use super::source::{source_app, source_app_icon, start_source_tracker};

const MAX_CAPTURE_BYTES: usize = 20 * 1024 * 1024;
// macOS pasteboard custom types must be valid UTIs; reverse-DNS also works as
// a private clipboard format on Windows and Linux.
const SELF_WRITE_FORMAT: &str = "com.clipclop.self-write";

pub struct SystemClipboard;

impl SystemClipboard {
    pub fn write(flavors: Vec<Flavor>, plain_text_only: bool) -> AppResult<()> {
        let context = ClipboardContext::new().map_err(clipboard_error)?;
        let mut contents = clipboard_contents(flavors, plain_text_only)?;
        if contents.is_empty() {
            return Err(AppError::Clipboard("clip has no writable flavors".into()));
        }
        // The marker belongs to this clipboard ownership and disappears on the
        // next external copy, avoiding timing races for every content type.
        contents.push(ClipboardContent::Other(
            SELF_WRITE_FORMAT.into(),
            b"clipclop".to_vec(),
        ));
        context.set(contents).map_err(clipboard_error)
    }

    pub fn preview_asset(flavors: &[Flavor], file_path: Option<&str>) -> AppResult<Option<String>> {
        let png = if let Some(path) = file_path {
            let path = local_file_path(path);
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
        Ok(png.map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes))))
    }

    pub fn thumbnail_asset(flavors: &[Flavor]) -> AppResult<Option<String>> {
        let image = if let Some(flavor) = flavors.iter().find(|item| item.format == "image/png") {
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

fn clipboard_contents(
    flavors: Vec<Flavor>,
    plain_text_only: bool,
) -> AppResult<Vec<ClipboardContent>> {
    let mut contents = Vec::new();
    for flavor in flavors {
        if plain_text_only && flavor.format != "text/plain" {
            continue;
        }
        match flavor.format.as_str() {
            "text/plain" => contents.push(ClipboardContent::Text(
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
            "text/html" => contents.push(ClipboardContent::Html(
                String::from_utf8_lossy(&flavor.payload).into_owned(),
            )),
            "text/rtf" => contents.push(ClipboardContent::Rtf(
                String::from_utf8_lossy(&flavor.payload).into_owned(),
            )),
            format => contents.push(ClipboardContent::Other(format.into(), flavor.payload)),
        }
    }
    Ok(contents)
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
        if self.clipboard.get_buffer(SELF_WRITE_FORMAT).is_ok() {
            return Ok(());
        }
        // Freeze attribution before reading or encoding large clipboard payloads.
        let captured_source = source_app(&self.clipboard);
        let state = self.app.state::<AppState>();
        let settings: Settings = state
            .database
            .get_setting(SETTINGS_KEY)?
            .unwrap_or_default();
        state.clips.prune(settings.retention_days)?;
        let Some(mut clip) = read_clip(&self.clipboard)? else {
            return Ok(());
        };
        clip.source_app = captured_source;
        if let Some(id) = state.clips.capture(&clip)? {
            let _ = self.app.emit("clips_changed", json!({ "latest_id": id }));
        }
        Ok(())
    }
}

pub fn start_watcher(app: AppHandle) -> AppResult<()> {
    start_source_tracker()?;
    thread::Builder::new()
        .name("clipclop-clipboard".into())
        .spawn(move || loop {
            let clipboard = match ClipboardContext::new() {
                Ok(clipboard) => clipboard,
                Err(error) => {
                    eprintln!("clipboard context unavailable: {error}");
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };
            let mut watcher = match ClipboardWatcherContext::new() {
                Ok(watcher) => watcher,
                Err(error) => {
                    eprintln!("clipboard watcher unavailable: {error}");
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };
            watcher.add_handler(CaptureHandler {
                app: app.clone(),
                clipboard,
            });
            watcher.start_watch();
            eprintln!("clipboard watcher stopped; restarting");
            thread::sleep(Duration::from_secs(1));
        })
        .map_err(|error| AppError::Platform(error.to_string()))?;
    Ok(())
}

fn read_clip(context: &ClipboardContext) -> AppResult<Option<NewClip>> {
    let mut flavors = Vec::new();
    // ponytail: clipboard-rs has no metadata-only size probe. Checking common raw
    // formats avoids decoding known oversized images; native per-platform probes
    // are only warranted if an unsupported producer causes measured memory issues.
    for format in [
        "public.png",
        "PNG",
        "image/png",
        "public.tiff",
        "TIFF",
        "image/tiff",
        "CF_DIB",
        "CF_DIBV5",
        "image/bmp",
    ] {
        if context
            .get_buffer(format)
            .is_ok_and(|payload| payload.len() > MAX_CAPTURE_BYTES)
        {
            return Err(AppError::Validation(format!(
                "clipboard image exceeds {} MiB",
                MAX_CAPTURE_BYTES / 1024 / 1024
            )));
        }
    }
    let files = context.get_files().ok().filter(|items| !items.is_empty());
    let image = context.get_image().ok();
    let text = context.get_text().ok().filter(|value| !value.is_empty());
    let html = text
        .as_ref()
        .and_then(|_| context.get_html().ok())
        .filter(|value| !value.is_empty());
    let rich_text = text
        .as_ref()
        .and_then(|_| context.get_rich_text().ok())
        .filter(|value| !value.is_empty());

    if let Some(value) = &text {
        flavors.push(Flavor {
            format: "text/plain".into(),
            payload: value.as_bytes().to_vec(),
        });
    }
    if let Some(value) = &html {
        flavors.push(Flavor {
            format: "text/html".into(),
            payload: value.as_bytes().to_vec(),
        });
    }
    if let Some(value) = &rich_text {
        flavors.push(Flavor {
            format: "text/rtf".into(),
            payload: value.as_bytes().to_vec(),
        });
    }
    let image_dimensions = if let Some(image) = image {
        let size = image.get_size();
        let png = image.to_png().map_err(clipboard_error)?;
        flavors.push(Flavor {
            format: "image/png".into(),
            payload: png.get_bytes().to_vec(),
        });
        Some(size)
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
    } else if image_dimensions.is_some() {
        ContentType::Image
    } else {
        classify_text(text.as_deref().unwrap_or_default())
    };
    let preview = if content_type == ContentType::Image {
        image_dimensions
            .map(|(width, height)| format!("{width} × {height}"))
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
        Some(files) => ClipMetadata {
            files,
            // ponytail: explicit preview requests probe file metadata; capture stays non-blocking.
            file_sizes: Vec::new(),
            ..Default::default()
        },
        None => match image_dimensions {
            Some((width, height)) => ClipMetadata {
                width: Some(width),
                height: Some(height),
                ..Default::default()
            },
            None => ClipMetadata {
                char_count: text.as_ref().map(|value| value.chars().count() as u64),
                ..Default::default()
            },
        },
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
            .unwrap_or_default();
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

fn local_file_path(path: &str) -> std::path::PathBuf {
    Url::parse(path)
        .ok()
        .filter(|url| url.scheme() == "file")
        .and_then(|url| url.to_file_path().ok())
        .unwrap_or_else(|| path.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipboard_rs::common::ContentData;

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

    #[test]
    fn rich_and_plain_copy_modes_select_the_expected_flavors() {
        let flavors = || {
            vec![
                Flavor {
                    format: "text/plain".into(),
                    payload: b"hello".to_vec(),
                },
                Flavor {
                    format: "text/html".into(),
                    payload: b"<b>hello</b>".to_vec(),
                },
                Flavor {
                    format: "text/rtf".into(),
                    payload: br"{\rtf1 hello}".to_vec(),
                },
            ]
        };
        let rich = clipboard_contents(flavors(), false).unwrap();
        assert_eq!(rich.len(), 3);
        assert!(matches!(
            rich[1].get_format(),
            clipboard_rs::ContentFormat::Html
        ));
        assert!(matches!(
            rich[2].get_format(),
            clipboard_rs::ContentFormat::Rtf
        ));

        let plain = clipboard_contents(flavors(), true).unwrap();
        assert_eq!(plain.len(), 1);
        assert!(matches!(
            plain[0].get_format(),
            clipboard_rs::ContentFormat::Text
        ));
    }
}
