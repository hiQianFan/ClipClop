#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyValidationError {
    InvalidFormat,
    MissingModifier,
    UnsupportedKey,
    DuplicateModifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ShortcutSpec {
    Combination(String),
    Single(String),
}

impl ShortcutSpec {
    pub fn parse(value: &str) -> Result<Self, HotkeyValidationError> {
        validate_hotkey(value)?;
        let parts: Vec<_> = value.split('+').collect();
        if parts.len() == 1 {
            Ok(Self::Single(value.into()))
        } else {
            Ok(Self::Combination(value.into()))
        }
    }
}

pub fn validate_hotkey(hotkey: &str) -> Result<(), HotkeyValidationError> {
    if hotkey.is_empty() || hotkey.chars().count() > 80 || hotkey.trim() != hotkey {
        return Err(HotkeyValidationError::InvalidFormat);
    }

    let parts: Vec<_> = hotkey.split('+').collect();
    let Some((key, modifiers)) = parts.split_last() else {
        return Err(HotkeyValidationError::MissingModifier);
    };
    if key.is_empty() || !supported_key(key) {
        return Err(if modifiers.is_empty() && allowed_modifier(key) {
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

    Ok(())
}

fn allowed_modifier(key: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        matches!(
            key,
            "Control" | "Ctrl" | "Alt" | "Shift" | "Command" | "Super"
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        matches!(key, "Ctrl" | "Alt" | "Shift" | "Super")
    }
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
    use crate::settings::DEFAULT_HOTKEY;

    #[test]
    fn validates_default_and_rejects_incomplete_hotkeys() {
        assert_eq!(validate_hotkey(DEFAULT_HOTKEY), Ok(()));
        assert_eq!(
            validate_hotkey("Ctrl"),
            Err(HotkeyValidationError::MissingModifier)
        );
        assert_eq!(validate_hotkey("C"), Ok(()));
        assert_eq!(
            validate_hotkey("Ctrl+Ctrl+C"),
            Err(HotkeyValidationError::DuplicateModifier)
        );
    }

    #[test]
    fn leaves_system_availability_to_registration() {
        assert!(validate_hotkey("Command+Q").is_ok());
    }

    #[test]
    fn parses_single_and_combination_specs() {
        assert_eq!(
            ShortcutSpec::parse("C"),
            Ok(ShortcutSpec::Single("C".into()))
        );
        assert_eq!(
            ShortcutSpec::parse(DEFAULT_HOTKEY),
            Ok(ShortcutSpec::Combination(DEFAULT_HOTKEY.into()))
        );
    }
}
use serde::{Deserialize, Serialize};
