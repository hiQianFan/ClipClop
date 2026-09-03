use std::{thread, time::Duration};

use chrono::Utc;
use clipboard_rs::{
    common::RustImage, Clipboard, ClipboardContent, ClipboardContext, ClipboardHandler,
    ClipboardWatcher, ClipboardWatcherContext,
};
use sha2::{Digest, Sha256};
use url::Url;

use crate::{
    error::{AppError, AppResult},
    history::{ClipMetadata, ContentType, Flavor, NewClip},
};

use super::source::{source_app, start_source_tracker};

const MAX_CAPTURE_BYTES: usize = 20 * 1024 * 1024;
// macOS pasteboard custom types must be valid UTIs; reverse-DNS also works as
// a private clipboard format on Windows and Linux.
const SELF_WRITE_FORMAT: &str = "com.clipclop.self-write";

pub struct SystemClipboard;

impl SystemClipboard {
    pub fn write(
        flavors: Vec<Flavor>,
        plain_text_only: bool,
        trim_whitespace: bool,
    ) -> AppResult<()> {
        let context = ClipboardContext::new().map_err(clipboard_error)?;
        let mut contents = clipboard_contents(flavors, plain_text_only, trim_whitespace)?;
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
}

fn clipboard_contents(
    flavors: Vec<Flavor>,
    plain_text_only: bool,
    trim_whitespace: bool,
) -> AppResult<Vec<ClipboardContent>> {
    let mut contents = Vec::new();
    for flavor in flavors {
        if plain_text_only && flavor.format != "text/plain" {
            continue;
        }
        match flavor.format.as_str() {
            "text/plain" => {
                let text = String::from_utf8_lossy(&flavor.payload);
                contents.push(ClipboardContent::Text(if trim_whitespace {
                    text.trim().into()
                } else {
                    text.into_owned()
                }));
            }
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
    clipboard: ClipboardContext,
    capture: std::sync::Arc<dyn Fn(NewClip) -> AppResult<()> + Send + Sync>,
}

impl ClipboardHandler for CaptureHandler {
    fn on_clipboard_change(&mut self) {
        if let Err(error) = self.capture() {
            log::warn!("clipboard capture failed: {error}");
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
        let Some(mut clip) = read_clip(&self.clipboard)? else {
            return Ok(());
        };
        clip.source_app = captured_source;
        (self.capture)(clip)
    }
}

pub fn start_watcher(
    capture: impl Fn(NewClip) -> AppResult<()> + Send + Sync + 'static,
) -> AppResult<()> {
    start_source_tracker()?;
    let capture = std::sync::Arc::new(capture);
    thread::Builder::new()
        .name("clipclop-clipboard".into())
        .spawn(move || loop {
            let clipboard = match ClipboardContext::new() {
                Ok(clipboard) => clipboard,
                Err(error) => {
                    log::error!("clipboard context unavailable: {error}");
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };
            let mut watcher = match ClipboardWatcherContext::new() {
                Ok(watcher) => watcher,
                Err(error) => {
                    log::error!("clipboard watcher unavailable: {error}");
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };
            watcher.add_handler(CaptureHandler {
                clipboard,
                capture: capture.clone(),
            });
            watcher.start_watch();
            log::warn!("clipboard watcher stopped; restarting");
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
    let image = clipboard_image(context);
    // Photos also advertises protected file promises for copied images. Prefer
    // the image payload so capture does not trigger a Photos access prompt.
    let files = image
        .is_none()
        .then(|| context.get_files().ok())
        .flatten()
        .filter(|items| !items.is_empty());
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

fn clipboard_image(context: &ClipboardContext) -> Option<clipboard_rs::RustImageData> {
    [
        "public.png",
        "PNG",
        "image/png",
        "public.tiff",
        "TIFF",
        "image/tiff",
    ]
    .into_iter()
    .find_map(|format| {
        context
            .get_buffer(format)
            .ok()
            .and_then(|payload| clipboard_rs::RustImageData::from_bytes(&payload).ok())
    })
    .or_else(|| context.get_image().ok())
}

fn classify_text(text: &str) -> ContentType {
    let trimmed = text.trim();
    if Url::parse(trimmed).is_ok_and(|url| matches!(url.scheme(), "http" | "https")) {
        ContentType::Link
    } else if is_color(trimmed) {
        ContentType::Color
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
            ContentType::Text
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
        let rich = clipboard_contents(flavors(), false, false).unwrap();
        assert_eq!(rich.len(), 3);
        assert!(matches!(
            rich[1].get_format(),
            clipboard_rs::ContentFormat::Html
        ));
        assert!(matches!(
            rich[2].get_format(),
            clipboard_rs::ContentFormat::Rtf
        ));

        let plain = clipboard_contents(flavors(), true, false).unwrap();
        assert_eq!(plain.len(), 1);
        assert!(matches!(
            plain[0].get_format(),
            clipboard_rs::ContentFormat::Text
        ));
    }

    #[test]
    fn trims_only_plain_text_when_enabled() {
        let unchanged = clipboard_contents(
            vec![Flavor {
                format: "text/plain".into(),
                payload: b"  hello\n".to_vec(),
            }],
            false,
            false,
        )
        .unwrap();
        assert_eq!(unchanged[0].as_str().unwrap(), "  hello\n");

        let contents = clipboard_contents(
            vec![
                Flavor {
                    format: "text/plain".into(),
                    payload: " \u{2003}hello  world\n ".as_bytes().to_vec(),
                },
                Flavor {
                    format: "text/html".into(),
                    payload: b"  <b>hello</b>  ".to_vec(),
                },
            ],
            false,
            true,
        )
        .unwrap();
        assert_eq!(contents[0].as_str().unwrap(), "hello  world");
        assert_eq!(contents[1].as_str().unwrap(), "  <b>hello</b>  ");

        let blank = clipboard_contents(
            vec![Flavor {
                format: "text/plain".into(),
                payload: b" \n\t".to_vec(),
            }],
            false,
            true,
        )
        .unwrap();
        assert_eq!(blank[0].as_str().unwrap(), "");
    }
}
