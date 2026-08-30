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
    match service.toggle(app, state, id, index) {
        Ok(true) => return Ok(PreviewOutcome::NativeOpened),
        Ok(false) | Err(AppError::NotFound | AppError::Validation(_)) => {}
        Err(error) => return Err(error),
    }

    Ok(PreviewOutcome::NotPreviewable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_have_stable_wire_values() {
        let values = [
            (PreviewOutcome::NativeOpened, "\"native_opened\""),
            (PreviewOutcome::NativeClosed, "\"native_closed\""),
            (PreviewOutcome::NotPreviewable, "\"not_previewable\""),
        ];
        for (outcome, expected) in values {
            assert_eq!(serde_json::to_string(&outcome).unwrap(), expected);
        }
    }
}
