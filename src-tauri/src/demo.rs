use std::{path::PathBuf, sync::Arc};

use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager};

use crate::{
    error::{AppError, AppResult},
    history::{ClipMetadata, ContentType, Flavor, NewClip, SourceApp},
    settings::LanguagePreference,
    state::HistoryEnvironment,
    storage::Database,
};

pub const PREVIEW_CACHE_NAMESPACE: &str = "demo-external-preview";
const SESSION_DIRECTORY: &str = "demo-session-";
const APP_ICON: &[u8] = include_bytes!("../../static/app-icon.png");
const DEMO_FILE: &[u8] = b"ClipClop demo file\n";

struct DemoItem {
    content_type: ContentType,
    text: &'static str,
    source: &'static str,
}

pub fn create_environment(
    app: &AppHandle,
    language: LanguagePreference,
) -> AppResult<HistoryEnvironment> {
    let directory = session_directory(app)?;
    std::fs::create_dir_all(&directory).map_err(platform_error)?;
    let file_path = directory.join("ClipClop.dmg");
    std::fs::write(&file_path, DEMO_FILE).map_err(platform_error)?;

    let environment =
        HistoryEnvironment::new(Arc::new(Database::in_memory()?), PREVIEW_CACHE_NAMESPACE)
            .with_demo_directory(directory.clone());
    let chinese = match language {
        LanguagePreference::ChineseSimplified => true,
        LanguagePreference::English => false,
        LanguagePreference::System => sys_locale::get_locale()
            .is_some_and(|locale| locale.to_ascii_lowercase().starts_with("zh")),
    };
    for (index, item) in items(chinese).iter().enumerate() {
        if let Err(error) = environment.history.capture(&new_clip(
            item,
            index,
            &file_path,
            Utc::now() - Duration::minutes(index as i64),
        )) {
            let _ = remove_directory(&directory);
            return Err(error);
        }
    }
    Ok(environment)
}

pub fn cleanup_environment(app: &AppHandle, environment: &HistoryEnvironment) -> AppResult<()> {
    environment.external_preview.clear_cached(app)?;
    if let Some(directory) = &environment.demo_directory {
        remove_directory(directory)?;
    }
    Ok(())
}

pub fn cleanup_stale(app: &AppHandle) -> AppResult<()> {
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|error| AppError::Platform(error.to_string()))?;
    if let Ok(entries) = std::fs::read_dir(&cache) {
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with(SESSION_DIRECTORY)
            {
                remove_directory(&entry.path())?;
            }
        }
    }
    remove_directory(&cache.join(PREVIEW_CACHE_NAMESPACE))
}

fn session_directory(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(app
        .path()
        .app_cache_dir()
        .map_err(|error| AppError::Platform(error.to_string()))?
        .join(format!("{SESSION_DIRECTORY}{}", uuid::Uuid::now_v7())))
}

fn remove_directory(path: &std::path::Path) -> AppResult<()> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(platform_error(error)),
    }
}

fn new_clip(
    item: &DemoItem,
    index: usize,
    file_path: &std::path::Path,
    created_at: chrono::DateTime<Utc>,
) -> NewClip {
    let (plain_text, preview, flavors, metadata) = match item.content_type {
        ContentType::Image => (
            None,
            item.text.into(),
            vec![Flavor {
                format: "image/png".into(),
                payload: APP_ICON.to_vec(),
            }],
            ClipMetadata {
                width: Some(256),
                height: Some(256),
                ..Default::default()
            },
        ),
        ContentType::File => {
            let path = file_path.to_string_lossy().into_owned();
            (
                None,
                item.text.into(),
                vec![Flavor {
                    format: "text/uri-list".into(),
                    payload: serde_json::to_vec(&[&path]).expect("demo path serializes"),
                }],
                ClipMetadata {
                    files: vec![path],
                    file_sizes: vec![Some(DEMO_FILE.len() as u64)],
                    ..Default::default()
                },
            )
        }
        _ => (
            Some(item.text.into()),
            item.text.into(),
            vec![Flavor {
                format: "text/plain".into(),
                payload: item.text.as_bytes().to_vec(),
            }],
            ClipMetadata {
                char_count: Some(item.text.chars().count() as u64),
                ..Default::default()
            },
        ),
    };
    NewClip {
        content_type: item.content_type,
        plain_text,
        preview,
        source_app: Some(SourceApp {
            id: format!(
                "demo:{}",
                item.source.to_ascii_lowercase().replace(' ', "-")
            ),
            name: item.source.into(),
        }),
        flavors,
        metadata,
        content_hash: format!("demo-{index}"),
        created_at,
    }
}

fn items(chinese: bool) -> &'static [DemoItem; 10] {
    if chinese {
        &ZH_ITEMS
    } else {
        &EN_ITEMS
    }
}

const ZH_ITEMS: [DemoItem; 10] = [
    DemoItem { content_type: ContentType::Link, text: "https://github.com/hiQianFan/ClipClop", source: "Google Chrome" },
    DemoItem { content_type: ContentType::Text, text: "macOS  ⌃⌘C  呼出\nWindows  Ctrl+Alt+C  呼出\n↑ ↓  选择\n← →  翻页\n1–0  快速选择\nSpace  预览\nEnter  粘贴\nShift+Enter  纯文本粘贴\nEsc  关闭", source: "ClipClop" },
    DemoItem { content_type: ContentType::Text, text: "console.log(\"Hello, ClipClop\");", source: "Codex" },
    DemoItem { content_type: ContentType::Link, text: "https://github.com/hiQianFan/ClipClop/releases/latest", source: "Safari" },
    DemoItem { content_type: ContentType::Image, text: "ClipClop 应用图标", source: "预览" },
    DemoItem { content_type: ContentType::File, text: "ClipClop.dmg", source: "访达" },
    DemoItem { content_type: ContentType::Color, text: "#ECEEF0", source: "Claude" },
    DemoItem { content_type: ContentType::Text, text: "保存、搜索、预览与粘贴，都在这台设备上完成。", source: "Microsoft Edge" },
    DemoItem { content_type: ContentType::Text, text: "无法自动粘贴时，内容仍会留在系统剪贴板。", source: "Google Chrome" },
    DemoItem { content_type: ContentType::Text, text: "剪贴历史保留时间\n1 天 / 7 天 / 30 天 / 90 天 / 1 年 / 永久\n\n剪贴历史数量上限\n100 条 / 500 条 / 1,000 条 / 5,000 条 / 不限制", source: "ClipClop" },
];

const EN_ITEMS: [DemoItem; 10] = [
    DemoItem { content_type: ContentType::Link, text: "https://github.com/hiQianFan/ClipClop", source: "Google Chrome" },
    DemoItem { content_type: ContentType::Text, text: "macOS  ⌃⌘C  Open\nWindows  Ctrl+Alt+C  Open\n↑ ↓  Select\n← →  Change page\n1–0  Quick select\nSpace  Preview\nEnter  Paste\nShift+Enter  Paste plain text\nEsc  Close", source: "ClipClop" },
    DemoItem { content_type: ContentType::Text, text: "console.log(\"Hello, ClipClop\");", source: "Codex" },
    DemoItem { content_type: ContentType::Link, text: "https://github.com/hiQianFan/ClipClop/releases/latest", source: "Safari" },
    DemoItem { content_type: ContentType::Image, text: "ClipClop app icon", source: "Preview" },
    DemoItem { content_type: ContentType::File, text: "ClipClop.dmg", source: "Finder" },
    DemoItem { content_type: ContentType::Color, text: "#ECEEF0", source: "Claude" },
    DemoItem { content_type: ContentType::Text, text: "Save, search, preview, and paste—all on this device.", source: "Microsoft Edge" },
    DemoItem { content_type: ContentType::Text, text: "If direct paste is unavailable, your content stays on the system clipboard.", source: "Google Chrome" },
    DemoItem { content_type: ContentType::Text, text: "Clipboard history retention\n1 / 7 / 30 / 90 days / 1 year / Forever\n\nClipboard history limit\n100 / 500 / 1,000 / 5,000 items / Unlimited", source: "ClipClop" },
];

fn platform_error(error: std::io::Error) -> AppError {
    AppError::Platform(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn website_fixture_has_the_expected_shape_and_order() {
        for items in [&ZH_ITEMS, &EN_ITEMS] {
            assert_eq!(items.len(), 10);
            assert_eq!(items[0].text, "https://github.com/hiQianFan/ClipClop");
            assert_eq!(
                items
                    .iter()
                    .filter(|item| item.content_type == ContentType::Text)
                    .count(),
                5
            );
            assert_eq!(
                items
                    .iter()
                    .filter(|item| item.content_type == ContentType::Link)
                    .count(),
                2
            );
            for content_type in [ContentType::Image, ContentType::File, ContentType::Color] {
                assert_eq!(
                    items
                        .iter()
                        .filter(|item| item.content_type == content_type)
                        .count(),
                    1
                );
            }
        }
    }
}
