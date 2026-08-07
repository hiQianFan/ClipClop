use serde::Serialize;
use tauri::AppHandle;

use crate::{
    error::{AppError, AppResult},
    preview::ExternalPreviewService,
    window::PreviewState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewOutcome {
    NativeOpened,
    NativeClosed,
    FallbackOpened,
    NotPreviewable,
}

pub fn preview(
    app: &AppHandle,
    state: &PreviewState,
    service: &ExternalPreviewService,
    id: &str,
    index: usize,
) -> AppResult<PreviewOutcome> {
    if state.is_active() {
        service.close_native(app, state)?;
        return Ok(PreviewOutcome::NativeClosed);
    }
    if service.toggle(app, state, id, index)? {
        return Ok(PreviewOutcome::NativeOpened);
    }

    let fallback = service.open_clip_file(app, id, index);
    match fallback {
        Ok(()) => Ok(PreviewOutcome::FallbackOpened),
        Err(AppError::Validation(_)) if index == 0 => match service.open_clip(app, id) {
            Ok(()) => Ok(PreviewOutcome::FallbackOpened),
            Err(AppError::NotFound) => Ok(PreviewOutcome::NotPreviewable),
            Err(error) => Err(error),
        },
        Err(AppError::NotFound) => Ok(PreviewOutcome::NotPreviewable),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_have_stable_wire_values() {
        let values = [
            (PreviewOutcome::NativeOpened, "\"native_opened\""),
            (PreviewOutcome::NativeClosed, "\"native_closed\""),
            (PreviewOutcome::FallbackOpened, "\"fallback_opened\""),
            (PreviewOutcome::NotPreviewable, "\"not_previewable\""),
        ];
        for (outcome, expected) in values {
            assert_eq!(serde_json::to_string(&outcome).unwrap(), expected);
        }
    }
}
