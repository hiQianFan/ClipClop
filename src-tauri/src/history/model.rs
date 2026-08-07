use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Text,
    Link,
    Color,
    Code,
    Image,
    File,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Text => "text",
            Self::Link => "link",
            Self::Color => "color",
            Self::Code => "code",
            Self::Image => "image",
            Self::File => "file",
        })
    }
}

impl FromStr for ContentType {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "text" => Ok(Self::Text),
            "link" => Ok(Self::Link),
            "color" => Ok(Self::Color),
            "code" => Ok(Self::Code),
            "image" => Ok(Self::Image),
            "file" => Ok(Self::File),
            _ => Err(AppError::Storage(format!("unknown content type: {value}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceApp {
    pub id: String,
    pub name: String,
}

impl SourceApp {
    pub(crate) fn is_meaningful(&self) -> bool {
        let process_name = self.id.rsplit(['/', '\\']).next().unwrap_or(&self.id);
        !self.name.eq_ignore_ascii_case("ClipClop")
            && !self.name.eq_ignore_ascii_case("loginwindow")
            && !self.id.eq_ignore_ascii_case("com.apple.loginwindow")
            && !process_name.eq_ignore_ascii_case("loginwindow")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Flavor {
    pub format: String,
    #[serde(skip_serializing)]
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewClip {
    pub content_type: ContentType,
    pub plain_text: Option<String>,
    pub preview: String,
    pub source_app: Option<SourceApp>,
    pub flavors: Vec<Flavor>,
    #[serde(default)]
    pub metadata: ClipMetadata,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub char_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_sizes: Vec<Option<u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipSummary {
    pub id: String,
    pub content_type: ContentType,
    pub preview: String,
    pub source_app: Option<SourceApp>,
    pub created_at: DateTime<Utc>,
    pub byte_size: u64,
    pub metadata: ClipMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipDetail {
    #[serde(flatten)]
    pub summary: ClipSummary,
    pub plain_text: Option<String>,
    pub flavors: Vec<FlavorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlavorInfo {
    pub format: String,
    pub byte_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQuery {
    #[serde(default)]
    pub query: String,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

impl Default for HistoryQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            page: default_page(),
            page_size: default_page_size(),
        }
    }
}

const fn default_page() -> u32 {
    1
}

const fn default_page_size() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryPage {
    pub items: Vec<ClipSummary>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub total_pages: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_detail_ipc_shape_is_stable() {
        let detail = ClipDetail {
            summary: ClipSummary {
                id: "clip-1".into(),
                content_type: ContentType::File,
                preview: "example.txt".into(),
                source_app: Some(SourceApp {
                    id: "com.example.app".into(),
                    name: "Example".into(),
                }),
                created_at: "2026-08-06T00:00:00Z".parse().unwrap(),
                byte_size: 12,
                metadata: ClipMetadata {
                    files: vec!["/tmp/example.txt".into()],
                    file_sizes: vec![Some(12)],
                    ..Default::default()
                },
            },
            plain_text: None,
            flavors: vec![FlavorInfo {
                format: "text/uri-list".into(),
                byte_size: 12,
            }],
        };

        assert_eq!(
            serde_json::to_value(detail).unwrap(),
            serde_json::json!({
                "id": "clip-1",
                "content_type": "file",
                "preview": "example.txt",
                "source_app": { "id": "com.example.app", "name": "Example" },
                "created_at": "2026-08-06T00:00:00Z",
                "byte_size": 12,
                "metadata": {
                    "files": ["/tmp/example.txt"],
                    "file_sizes": [12]
                },
                "plain_text": null,
                "flavors": [{ "format": "text/uri-list", "byte_size": 12 }]
            })
        );
    }
}
