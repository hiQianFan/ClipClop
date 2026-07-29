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

#[cfg(target_os = "macos")]
use clipboard_rs::Clipboard;
use clipboard_rs::ClipboardContext;

use crate::{
    error::{AppError, AppResult},
    history::SourceApp,
};

const RECENT_SOURCE_MAX_AGE: Duration = Duration::from_secs(2);

#[derive(Clone)]
struct RecentSource {
    app: SourceApp,
    seen_at: Instant,
}

static RECENT_SOURCE: OnceLock<Mutex<Option<RecentSource>>> = OnceLock::new();

pub(crate) fn source_app(clipboard: &ClipboardContext) -> Option<SourceApp> {
    resolve_source(
        [
            declared_source_app(clipboard),
            platform_source_app(clipboard),
            window_source_app(),
        ],
        recent_source_app(),
    )
}

fn resolve_source(
    candidates: impl IntoIterator<Item = Option<SourceApp>>,
    recent: Option<SourceApp>,
) -> Option<SourceApp> {
    if let Some(source) = candidates.into_iter().flatten().next() {
        // A rejected source is conclusive. Falling back to the recent app would
        // merely replace one known-wrong attribution with another.
        return source.is_meaningful().then_some(source);
    }
    recent
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
    if !source.is_meaningful() {
        return;
    }
    if let Ok(mut recent) = RECENT_SOURCE.get_or_init(|| Mutex::new(None)).lock() {
        *recent = Some(RecentSource {
            app: source,
            seen_at: Instant::now(),
        });
    }
}

pub(crate) fn start_source_tracker() -> AppResult<()> {
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
    for pasteboard_type in ["org.nspasteboard.source"] {
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
    Some(SourceApp {
        id: app_path.to_string_lossy().into_owned(),
        name: app_path.file_stem()?.to_string_lossy().into_owned(),
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn loginwindow_is_treated_as_unknown_without_recent_fallback() {
        let loginwindow = SourceApp {
            id: "/System/Library/CoreServices/loginwindow.app/Contents/MacOS/loginwindow".into(),
            name: "loginwindow".into(),
        };
        let previous = SourceApp {
            id: "com.example.editor".into(),
            name: "Editor".into(),
        };
        assert_eq!(resolve_source([Some(loginwindow)], Some(previous)), None);

        let bundle_id = SourceApp {
            id: "com.apple.loginwindow".into(),
            name: "Login Window".into(),
        };
        assert!(!bundle_id.is_meaningful());
    }
}
