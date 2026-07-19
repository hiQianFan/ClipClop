#[cfg(target_os = "macos")]
use appkit_nsworkspace_bindings::{INSRunningApplication, INSWorkspace, NSWorkspace, INSURL};
#[cfg(target_os = "macos")]
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};
#[cfg(target_os = "macos")]
use objc::runtime::Object;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::path::Path;
#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::{
        DataExchange::GetClipboardOwner,
        Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION},
    },
    UI::WindowsAndMessaging::GetWindowThreadProcessId,
};

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
    clips::{ContentType, Flavor, NewClip, SourceApp},
    commands::Settings,
    error::{AppError, AppResult},
    state::AppState,
};

const MAX_CAPTURE_BYTES: usize = 20 * 1024 * 1024;
const RECENT_SOURCE_MAX_AGE: Duration = Duration::from_secs(2);
// macOS pasteboard custom types must be valid UTIs; a reverse-DNS identifier
// also works as a private clipboard format on Windows and Linux.
const SELF_WRITE_FORMAT: &str = "com.clipclop.self-write";
#[cfg(target_os = "macos")]
const REMOTE_CLIPBOARD_FORMAT: &str = "com.apple.is-remote-clipboard";
const UNIVERSAL_CLIPBOARD_SOURCE_ID: &str = "com.apple.universal-clipboard";
#[cfg(target_os = "macos")]
const UNIVERSAL_CLIPBOARD_SOURCE_NAME: &str = "其他 Apple 设备";

#[derive(Clone)]
struct RecentSource {
    app: SourceApp,
    seen_at: Instant,
}

static RECENT_SOURCE: OnceLock<Mutex<Option<RecentSource>>> = OnceLock::new();

pub struct SystemClipboard;

impl SystemClipboard {
    pub fn write(flavors: Vec<Flavor>) -> AppResult<()> {
        let context = ClipboardContext::new().map_err(clipboard_error)?;
        let mut contents = Vec::new();
        for flavor in flavors {
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
                format => contents.push(ClipboardContent::Other(format.into(), flavor.payload)),
            }
        }
        if contents.is_empty() {
            return Err(AppError::Clipboard("clip has no writable flavors".into()));
        }
        // A clipboard manager must distinguish replaying history from a new user copy.
        // The marker travels with this clipboard ownership and disappears on the next
        // external copy, so it has no timing race and works for text, images and files.
        contents.push(ClipboardContent::Other(
            SELF_WRITE_FORMAT.into(),
            b"clipclop".to_vec(),
        ));
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
        if self.clipboard.get_buffer(SELF_WRITE_FORMAT).is_ok() {
            return Ok(());
        }
        // Freeze attribution before reading or encoding large clipboard payloads.
        let captured_source = source_app(&self.clipboard)?;
        let state = self.app.state::<AppState>();
        let settings: Settings = state.database.get_setting("app")?.unwrap_or_default();
        state.clips.prune(settings.retention_days)?;
        let Some(mut clip) = read_clip(&self.clipboard)? else {
            return Ok(());
        };
        clip.source_app = captured_source;
        if clip.source_app.as_ref().is_some_and(|source| {
            source.id != UNIVERSAL_CLIPBOARD_SOURCE_ID && settings.ignored_apps.contains(&source.id)
        }) {
            return Ok(());
        }
        if let Some(id) = state.clips.capture(&clip)? {
            let _ = self.app.emit("clips_changed", json!({ "latest_id": id }));
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn source_app(clipboard: &ClipboardContext) -> AppResult<Option<SourceApp>> {
    let formats = clipboard.available_formats().map_err(clipboard_error)?;
    Ok(resolve_macos_source(
        &formats,
        [
            declared_source_app(clipboard),
            platform_source_app(clipboard),
            window_source_app(),
        ],
        recent_source_app(),
    ))
}

#[cfg(not(target_os = "macos"))]
fn source_app(clipboard: &ClipboardContext) -> AppResult<Option<SourceApp>> {
    Ok(resolve_source(
        [
            declared_source_app(clipboard),
            platform_source_app(clipboard),
            window_source_app(),
        ],
        recent_source_app(),
    ))
}

#[cfg(target_os = "macos")]
fn universal_clipboard_source(formats: &[String]) -> Option<SourceApp> {
    formats
        .iter()
        .any(|format| format == REMOTE_CLIPBOARD_FORMAT)
        .then(|| SourceApp {
            id: UNIVERSAL_CLIPBOARD_SOURCE_ID.into(),
            name: UNIVERSAL_CLIPBOARD_SOURCE_NAME.into(),
        })
}

#[cfg(target_os = "macos")]
fn resolve_macos_source(
    formats: &[String],
    candidates: impl IntoIterator<Item = Option<SourceApp>>,
    recent: Option<SourceApp>,
) -> Option<SourceApp> {
    universal_clipboard_source(formats).or_else(|| resolve_source(candidates, recent))
}

fn resolve_source(
    candidates: impl IntoIterator<Item = Option<SourceApp>>,
    recent: Option<SourceApp>,
) -> Option<SourceApp> {
    if let Some(source) = candidates.into_iter().flatten().next() {
        // Finding ClipClop is conclusive. Falling through to RECENT_SOURCE here
        // incorrectly attributes our own write to the previously focused app.
        return is_external_source(&source).then_some(source);
    }
    recent
}

fn is_external_source(source: &SourceApp) -> bool {
    !source.name.eq_ignore_ascii_case("ClipClop")
}

fn window_source_app() -> Option<SourceApp> {
    active_win_pos_rs::get_active_window()
        .ok()
        .map(|window| SourceApp {
            id: window.process_path.to_string_lossy().into_owned(),
            name: window.app_name,
        })
}

fn recent_source_app() -> Option<SourceApp> {
    let source = RECENT_SOURCE.get()?.lock().ok()?.clone()?;
    (source.seen_at.elapsed() <= RECENT_SOURCE_MAX_AGE).then_some(source.app)
}

fn remember_source(source: SourceApp) {
    if !is_external_source(&source) {
        return;
    }
    if let Ok(mut recent) = RECENT_SOURCE.get_or_init(|| Mutex::new(None)).lock() {
        *recent = Some(RecentSource {
            app: source,
            seen_at: Instant::now(),
        });
    }
}

fn start_source_tracker() -> AppResult<()> {
    thread::Builder::new()
        .name("clipclop-source".into())
        .spawn(|| loop {
            if let Some(source) = window_source_app() {
                remember_source(source);
            }
            thread::sleep(Duration::from_millis(150));
        })
        .map(|_| ())
        .map_err(|error| AppError::Platform(error.to_string()))
}

#[cfg(not(target_os = "macos"))]
fn declared_source_app(_: &ClipboardContext) -> Option<SourceApp> {
    None
}

#[cfg(target_os = "macos")]
fn platform_source_app(clipboard: &ClipboardContext) -> Option<SourceApp> {
    signature_source_app(clipboard).or_else(frontmost_source_app)
}

#[cfg(target_os = "macos")]
fn declared_source_app(clipboard: &ClipboardContext) -> Option<SourceApp> {
    const SOURCE_TYPES: [&str; 1] = ["org.nspasteboard.source"];
    for pasteboard_type in SOURCE_TYPES {
        let Ok(payload) = clipboard.get_buffer(pasteboard_type) else {
            continue;
        };
        let identifier = String::from_utf8_lossy(&payload)
            .trim_matches(['\0', ' ', '\n', '\r'])
            .to_owned();
        if let Some(source) = resolve_macos_application(&identifier) {
            return Some(source);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn signature_source_app(clipboard: &ClipboardContext) -> Option<SourceApp> {
    let formats = clipboard.available_formats().ok()?;
    let signature = matching_source_signature(&formats, process_is_running)?;
    resolve_macos_application(signature.bundle_id)
}

#[cfg(target_os = "macos")]
struct MacSourceSignature {
    pasteboard_type: &'static str,
    bundle_id: &'static str,
    process_name: &'static str,
}

#[cfg(target_os = "macos")]
const MAC_SOURCE_SIGNATURES: &[MacSourceSignature] = &[MacSourceSignature {
    pasteboard_type: "com.trolltech.anymime.image--png",
    bundle_id: "com.Snipaste",
    process_name: "Snipaste",
}];

#[cfg(target_os = "macos")]
fn matching_source_signature(
    formats: &[String],
    is_running: impl Fn(&str) -> bool,
) -> Option<&'static MacSourceSignature> {
    MAC_SOURCE_SIGNATURES.iter().find(|signature| {
        formats
            .iter()
            .any(|format| format == signature.pasteboard_type)
            && is_running(signature.process_name)
    })
}

#[cfg(target_os = "macos")]
fn process_is_running(name: &str) -> bool {
    Command::new("pgrep")
        .args(["-x", name])
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(target_os = "macos")]
fn resolve_macos_application(identifier: &str) -> Option<SourceApp> {
    if identifier.is_empty() {
        return None;
    }
    let direct_path = Path::new(identifier);
    let app_path = if direct_path.exists() {
        direct_path
            .ancestors()
            .find(|path| path.extension().is_some_and(|extension| extension == "app"))?
            .to_path_buf()
    } else {
        let escaped_identifier = identifier.replace('\'', "\\'");
        let query = format!("kMDItemCFBundleIdentifier == '{escaped_identifier}'");
        let output = Command::new("/usr/bin/mdfind").arg(query).output().ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8(output.stdout)
            .ok()?
            .lines()
            .map(Path::new)
            .find(|path| path.extension().is_some_and(|extension| extension == "app"))?
            .to_path_buf()
    };
    let name = app_path.file_stem()?.to_string_lossy().into_owned();
    Some(SourceApp {
        id: app_path.to_string_lossy().into_owned(),
        name,
    })
}

#[cfg(target_os = "macos")]
fn frontmost_source_app() -> Option<SourceApp> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let application = workspace.frontmostApplication();
        if application.0.is_null() {
            return None;
        }
        let name = nsstring_to_string(application.localizedName().0);
        let bundle_url = application.bundleURL();
        let id = if bundle_url.0.is_null() {
            nsstring_to_string(application.bundleIdentifier().0)
        } else {
            nsstring_to_string(bundle_url.path().0)
        };
        (!name.is_empty() && !id.is_empty()).then_some(SourceApp { id, name })
    }
}

#[cfg(target_os = "macos")]
unsafe fn nsstring_to_string(value: *mut Object) -> String {
    if value.is_null() {
        return String::new();
    }
    CFString::wrap_under_get_rule(value.cast::<std::ffi::c_void>() as CFStringRef).to_string()
}

#[cfg(target_os = "windows")]
fn platform_source_app(_: &ClipboardContext) -> Option<SourceApp> {
    unsafe {
        let owner = GetClipboardOwner();
        if owner.is_null() {
            return None;
        }
        let mut process_id = 0;
        if GetWindowThreadProcessId(owner, &mut process_id) == 0 || process_id == 0 {
            return None;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if process.is_null() {
            return None;
        }
        let mut buffer = vec![0u16; 32_768];
        let mut length = buffer.len() as u32;
        let read = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length);
        let _ = CloseHandle(process);
        if read == 0 || length == 0 {
            return None;
        }
        let id = String::from_utf16_lossy(&buffer[..length as usize]);
        let name = Path::new(&id)
            .file_stem()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        (!name.is_empty()).then_some(SourceApp { id, name })
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_source_app(_: &ClipboardContext) -> Option<SourceApp> {
    None
}

pub fn start_watcher(app: AppHandle) -> AppResult<()> {
    start_source_tracker()?;
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

    if let Some(value) = &text {
        flavors.push(Flavor {
            format: "text/plain".into(),
            payload: value.as_bytes().to_vec(),
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

    #[cfg(target_os = "macos")]
    #[test]
    fn recognizes_snipaste_image_signature_only() {
        let formats = [
            "public.png".into(),
            "com.trolltech.anymime.image--png".into(),
        ];
        assert_eq!(
            matching_source_signature(&formats, |process| process == "Snipaste")
                .map(|signature| signature.bundle_id),
            Some("com.Snipaste")
        );
        assert!(matching_source_signature(&["public.png".into()], |_| true).is_none());
        assert!(matching_source_signature(&formats, |_| false).is_none());
    }

    #[test]
    fn clipclop_source_blocks_recent_external_fallback() {
        let clipclop = SourceApp {
            id: "com.clipclop.desktop".into(),
            name: "ClipClop".into(),
        };
        let previous = SourceApp {
            id: "com.example.editor".into(),
            name: "Editor".into(),
        };

        assert_eq!(resolve_source([Some(clipclop)], Some(previous)), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn universal_clipboard_marker_resolves_to_dedicated_source() {
        let formats = vec![
            "public.utf8-plain-text".into(),
            REMOTE_CLIPBOARD_FORMAT.into(),
        ];

        assert_eq!(
            universal_clipboard_source(&formats),
            Some(SourceApp {
                id: UNIVERSAL_CLIPBOARD_SOURCE_ID.into(),
                name: UNIVERSAL_CLIPBOARD_SOURCE_NAME.into(),
            })
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn universal_clipboard_source_takes_priority_over_local_attribution() {
        let local = SourceApp {
            id: "/Applications/Editor.app".into(),
            name: "Editor".into(),
        };

        assert_eq!(
            resolve_macos_source(
                &[REMOTE_CLIPBOARD_FORMAT.into()],
                [Some(local.clone())],
                Some(local),
            )
            .map(|source| source.id),
            Some(UNIVERSAL_CLIPBOARD_SOURCE_ID.into())
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn ordinary_clipboard_formats_do_not_create_remote_source() {
        let local = SourceApp {
            id: "/Applications/Editor.app".into(),
            name: "Editor".into(),
        };
        assert_eq!(
            resolve_macos_source(
                &["public.utf8-plain-text".into()],
                [Some(local.clone())],
                None,
            ),
            Some(local)
        );
    }
}
