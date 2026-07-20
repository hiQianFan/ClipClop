use serde::{Deserialize, Serialize};

pub const SETTINGS_KEY: &str = "app";

#[cfg(target_os = "macos")]
pub const DEFAULT_HOTKEY: &str = "Control+Command+C";

#[cfg(not(target_os = "macos"))]
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+C";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub retention_days: u32,
    pub launch_at_login: bool,
    pub hotkey: String,
    pub theme: Theme,
    pub check_updates: bool,
    pub last_update_check: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            retention_days: 30,
            launch_at_login: false,
            hotkey: DEFAULT_HOTKEY.into(),
            theme: Theme::System,
            check_updates: true,
            last_update_check: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_outdated_settings_fields() {
        let json = r#"{"retention_days":30,"launch_at_login":false,"hotkey":"test","ignored_apps":[],"theme":"system","check_updates":true,"last_update_check":null}"#;
        assert!(serde_json::from_str::<Settings>(json).is_err());
    }
}
