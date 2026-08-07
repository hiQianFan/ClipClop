#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyValidationError {
    InvalidFormat,
    MissingModifier,
    UnsupportedKey,
    DuplicateModifier,
    Reserved,
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
    use crate::settings::DEFAULT_HOTKEY;

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
