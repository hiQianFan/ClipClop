use serde::{Deserialize, Serialize};

mod service;

pub use service::SettingsService;

pub const SETTINGS_KEY: &str = "app";

#[cfg(target_os = "macos")]
pub const DEFAULT_HOTKEY: &str = "Control+Command+C";

#[cfg(not(target_os = "macos"))]
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+C";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default = "default_retention_days")]
    pub retention_days: Option<u32>,
    #[serde(default = "default_history_limit")]
    pub history_limit: Option<u32>,
    #[serde(default = "default_move_used_to_top")]
    pub move_used_to_top: bool,
    pub launch_at_login: bool,
    pub hotkey: String,
    pub theme: Theme,
    pub language: LanguagePreference,
    pub check_updates: bool,
    pub last_update_check: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            retention_days: default_retention_days(),
            history_limit: default_history_limit(),
            move_used_to_top: default_move_used_to_top(),
            launch_at_login: false,
            hotkey: DEFAULT_HOTKEY.into(),
            theme: Theme::System,
            language: LanguagePreference::System,
            check_updates: true,
            last_update_check: None,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyValidationError {
    InvalidFormat,
    MissingModifier,
    UnsupportedKey,
    DuplicateModifier,
    Reserved,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

pub fn validate_hotkey(hotkey: &str) -> Result<(), HotkeyValidationError> {
    if hotkey.is_empty() || hotkey.chars().count() > 80 || hotkey.trim() != hotkey {
        return Err(HotkeyValidationError::InvalidFormat);
    }

    let parts: Vec<_> = hotkey.split('+').collect();
    let Some((key, modifiers)) = parts.split_last() else {
        return Err(HotkeyValidationError::MissingModifier);
    };
    if modifiers.is_empty() || key.is_empty() || !supported_key(key) {
        return Err(if modifiers.is_empty() {
            HotkeyValidationError::MissingModifier
        } else {
            HotkeyValidationError::UnsupportedKey
        });
    }

    #[cfg(target_os = "macos")]
    let allowed = ["Control", "Alt", "Shift", "Command"];
    #[cfg(not(target_os = "macos"))]
    let allowed = ["Ctrl", "Alt", "Shift", "Super"];

    if modifiers.iter().enumerate().any(|(index, modifier)| {
        !allowed.contains(modifier) || modifiers[..index].contains(modifier)
    }) {
        return Err(HotkeyValidationError::DuplicateModifier);
    }

    if is_reserved_hotkey(modifiers, key) {
        return Err(HotkeyValidationError::Reserved);
    }

    Ok(())
}

fn modifiers_match(actual: &[&str], expected: &[&str]) -> bool {
    actual.len() == expected.len() && expected.iter().all(|item| actual.contains(item))
}

#[cfg(target_os = "macos")]
fn is_reserved_hotkey(modifiers: &[&str], key: &str) -> bool {
    (modifiers_match(modifiers, &["Command"])
        && matches!(
            key,
            "A" | "C" | "F" | "H" | "M" | "Q" | "S" | "Tab" | "V" | "W" | "X" | "Z" | "Space"
        ))
        || (modifiers_match(modifiers, &["Control"]) && key == "Space")
        || (modifiers_match(modifiers, &["Alt"]) && key == "Space")
        || (modifiers_match(modifiers, &["Command", "Shift"]) && key == "W")
        || (modifiers_match(modifiers, &["Control", "Command"]) && key == "Q")
        || (modifiers_match(modifiers, &["Alt", "Command"]) && key == "Escape")
        || (modifiers_match(modifiers, &["Command", "Shift"]) && matches!(key, "3" | "4" | "5"))
}

#[cfg(not(target_os = "macos"))]
fn is_reserved_hotkey(modifiers: &[&str], key: &str) -> bool {
    (modifiers_match(modifiers, &["Ctrl"])
        && matches!(key, "A" | "C" | "F" | "S" | "V" | "W" | "X" | "Z" | "Space"))
        || (modifiers_match(modifiers, &["Alt"]) && matches!(key, "F4" | "Space" | "Tab"))
        || (modifiers_match(modifiers, &["Super"])
            && matches!(key, "D" | "E" | "L" | "R" | "S" | "Tab" | "V"))
        || (modifiers_match(modifiers, &["Ctrl", "Alt"]) && key == "Delete")
}

fn supported_key(key: &str) -> bool {
    (key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric())
        || matches!(
            key,
            "F1" | "F2"
                | "F3"
                | "F4"
                | "F5"
                | "F6"
                | "F7"
                | "F8"
                | "F9"
                | "F10"
                | "F11"
                | "F12"
                | "ArrowUp"
                | "ArrowDown"
                | "ArrowLeft"
                | "ArrowRight"
                | "Backspace"
                | "Delete"
                | "Home"
                | "End"
                | "PageUp"
                | "PageDown"
                | "Enter"
                | "Escape"
                | "Tab"
                | "Space"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_outdated_settings_fields() {
        let json = r#"{"retention_days":30,"launch_at_login":false,"hotkey":"test","ignored_apps":[],"theme":"system","check_updates":true,"last_update_check":null}"#;
        assert!(serde_json::from_str::<Settings>(json).is_err());
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
    fn old_settings_receive_new_history_defaults_and_null_round_trips() {
        let old = r#"{"retention_days":30,"launch_at_login":false,"hotkey":"Control+Command+C","theme":"system","language":"system","check_updates":true,"last_update_check":null}"#;
        let settings: Settings = serde_json::from_str(old).unwrap();
        assert_eq!(settings.retention_days, Some(30));
        assert_eq!(settings.history_limit, Some(500));
        assert!(settings.move_used_to_top);

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
    fn validates_default_and_rejects_incomplete_hotkeys() {
        assert_eq!(validate_hotkey(DEFAULT_HOTKEY), Ok(()));
        assert_eq!(
            validate_hotkey("Ctrl"),
            Err(HotkeyValidationError::MissingModifier)
        );
        assert_eq!(
            validate_hotkey("C"),
            Err(HotkeyValidationError::MissingModifier)
        );
        assert_eq!(
            validate_hotkey("Ctrl+Ctrl+C"),
            Err(HotkeyValidationError::DuplicateModifier)
        );
    }

    #[test]
    fn rejects_reserved_system_hotkeys() {
        #[cfg(target_os = "macos")]
        {
            assert!(validate_hotkey("Command+C").is_err());
            assert!(validate_hotkey("Command+Q").is_err());
            assert!(validate_hotkey("Command+V").is_err());
            assert!(validate_hotkey("Command+Tab").is_err());
            assert!(validate_hotkey("Control+Space").is_err());
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(validate_hotkey("Ctrl+C").is_err());
            assert!(validate_hotkey("Super+V").is_err());
            assert!(validate_hotkey("Alt+Tab").is_err());
            assert!(validate_hotkey("Super+L").is_err());
        }
        assert!(validate_hotkey("Alt+Space").is_err());
    }
}
