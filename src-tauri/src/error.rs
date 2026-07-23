use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("The requested item was not found")]
    NotFound,
    #[error("The request is invalid: {0}")]
    Validation(String),
    #[error("Storage operation failed: {0}")]
    Storage(String),
    #[error("Clipboard operation failed: {0}")]
    Clipboard(String),
    #[error("Platform integration failed: {0}")]
    Platform(String),
    #[error("Hotkey validation failed")]
    Hotkey(&'static str),
    #[error("Hotkey unavailable: {0}")]
    HotkeyUnavailable(String),
}

#[derive(Debug, Serialize)]
pub struct ErrorDto {
    pub code: &'static str,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let code = match self {
            Self::NotFound => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::Clipboard(_) => "CLIPBOARD_ERROR",
            Self::Platform(_) => "PLATFORM_ERROR",
            Self::Hotkey(code) => code,
            Self::HotkeyUnavailable(_) => "HOTKEY_UNAVAILABLE",
        };
        // Log the diagnostic locally for troubleshooting; only the code crosses IPC.
        match self {
            Self::HotkeyUnavailable(diagnostic) => {
                log::error!("IPC error {code}: global-shortcut plugin: {diagnostic}");
            }
            _ => log::error!("IPC error {code}"),
        }
        ErrorDto { code }.serialize(serializer)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Storage(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Storage(value.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_errors_expose_codes_without_diagnostics() {
        let value = serde_json::to_value(AppError::Storage("database path secret".into())).unwrap();
        assert_eq!(value, serde_json::json!({ "code": "STORAGE_ERROR" }));
        let hotkey = serde_json::to_value(AppError::HotkeyUnavailable("os detail".into())).unwrap();
        assert_eq!(hotkey, serde_json::json!({ "code": "HOTKEY_UNAVAILABLE" }));
    }
}
