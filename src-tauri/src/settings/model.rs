use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(target_os = "macos")]
pub const DEFAULT_HOTKEY: &str = "Control+Command+C";

#[cfg(not(target_os = "macos"))]
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+C";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(default = "default_retention_days")]
    pub retention_days: Option<u32>,
    #[serde(default = "default_history_limit")]
    pub history_limit: Option<u32>,
    #[serde(default = "default_move_used_to_top")]
    pub move_used_to_top: bool,
    #[serde(default)]
    pub restore_browse_position: bool,
    #[serde(default)]
    pub trim_whitespace: bool,
    #[serde(default)]
    pub file_preview_enabled: bool,
    pub launch_at_login: bool,
    pub hotkey: String,
    pub theme: Theme,
    pub language: LanguagePreference,
    #[serde(default)]
    pub tray_click_action: TrayClickAction,
    pub check_updates: bool,
    pub last_update_check: Option<String>,
    #[serde(default)]
    pub skipped_update_version: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            retention_days: default_retention_days(),
            history_limit: default_history_limit(),
            move_used_to_top: default_move_used_to_top(),
            restore_browse_position: false,
            trim_whitespace: false,
            file_preview_enabled: false,
            launch_at_login: false,
            hotkey: DEFAULT_HOTKEY.into(),
            theme: Theme::System,
            language: LanguagePreference::System,
            tray_click_action: TrayClickAction::Recent,
            check_updates: true,
            last_update_check: None,
            skipped_update_version: None,
            extra: BTreeMap::new(),
        }
    }
}

fn default_retention_days() -> Option<u32> {
    Some(30)
}

fn default_history_limit() -> Option<u32> {
    Some(500)
}

fn default_move_used_to_top() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LanguagePreference {
    #[serde(rename = "zh-CN")]
    ChineseSimplified,
    #[serde(rename = "en")]
    English,
    #[default]
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrayClickAction {
    #[default]
    Recent,
    History,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_settings_fields_survive_a_round_trip() {
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        value["future_setting"] = serde_json::json!({ "enabled": true });

        let settings: Settings = serde_json::from_value(value).unwrap();
        assert_eq!(
            serde_json::to_value(settings).unwrap()["future_setting"],
            serde_json::json!({ "enabled": true })
        );
    }

    #[test]
    fn language_is_required_and_strict() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert_eq!(value["language"], "system");
        let mut old = value.clone();
        old.as_object_mut().unwrap().remove("language");
        assert!(serde_json::from_value::<Settings>(old).is_err());
        let mut invalid = value;
        invalid["language"] = serde_json::json!("fr");
        assert!(serde_json::from_value::<Settings>(invalid).is_err());
    }

    #[test]
    fn released_settings_receive_new_defaults_and_null_round_trips() {
        let old = r#"{"retention_days":30,"launch_at_login":false,"hotkey":"Control+Command+C","theme":"system","language":"system","check_updates":true,"last_update_check":null}"#;
        let settings: Settings = serde_json::from_str(old).unwrap();
        assert_eq!(settings.retention_days, Some(30));
        assert_eq!(settings.history_limit, Some(500));
        assert!(settings.move_used_to_top);
        assert!(!settings.restore_browse_position);
        assert!(!settings.trim_whitespace);
        assert!(!settings.file_preview_enabled);
        assert_eq!(settings.tray_click_action, TrayClickAction::Recent);

        let unlimited = Settings {
            retention_days: None,
            history_limit: None,
            ..settings
        };
        let value = serde_json::to_value(unlimited).unwrap();
        assert!(value["retention_days"].is_null());
        assert!(value["history_limit"].is_null());
    }

    #[test]
    fn settings_ipc_shape_is_stable() {
        assert_eq!(
            serde_json::to_value(Settings::default()).unwrap(),
            serde_json::json!({
                "retention_days": 30,
                "history_limit": 500,
                "move_used_to_top": true,
                "restore_browse_position": false,
                "trim_whitespace": false,
                "file_preview_enabled": false,
                "launch_at_login": false,
                "hotkey": DEFAULT_HOTKEY,
                "theme": "system",
                "language": "system",
                "tray_click_action": "recent",
                "check_updates": true,
                "last_update_check": null,
                "skipped_update_version": null
            })
        );
    }
}
