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
}

#[derive(Debug, Serialize)]
pub struct ErrorDto {
    pub code: &'static str,
    pub message: String,
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
        };
        ErrorDto {
            code,
            message: self.to_string(),
        }
        .serialize(serializer)
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
