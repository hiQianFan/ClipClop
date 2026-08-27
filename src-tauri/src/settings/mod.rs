mod hotkey;
mod model;
mod service;

pub use hotkey::{validate_hotkey, HotkeyValidationError};
pub use model::{LanguagePreference, Settings, Theme, TrayClickAction, DEFAULT_HOTKEY};
pub use service::SettingsService;

pub const SETTINGS_KEY: &str = "app";
