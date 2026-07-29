use tauri::AppHandle;

use crate::{
    clipboard::SystemClipboard, error::AppResult, history::HistoryService, preview::PreviewService,
};

pub fn delete_clip(
    app: &AppHandle,
    history: &HistoryService,
    preview: &PreviewService,
    id: &str,
) -> AppResult<()> {
    persist_then_cleanup(
        || history.delete(id),
        || preview.delete_cached(app, id),
        "delete",
        id,
    )
}

pub fn clear_history(
    app: &AppHandle,
    history: &HistoryService,
    preview: &PreviewService,
) -> AppResult<u64> {
    persist_then_cleanup(
        || history.clear(),
        || preview.clear_cached(app),
        "clear",
        "history",
    )
}

pub fn copy_clip(history: &HistoryService, id: &str, plain_text_only: bool) -> AppResult<()> {
    SystemClipboard::write(history.flavors(id)?, plain_text_only)
}

fn persist_then_cleanup<T>(
    persist: impl FnOnce() -> AppResult<T>,
    cleanup: impl FnOnce() -> AppResult<()>,
    action: &str,
    target: &str,
) -> AppResult<T> {
    let value = persist()?;
    if let Err(error) = cleanup() {
        log::warn!("failed to {action} cached preview for {target}: {error}");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use std::cell::RefCell;

    #[test]
    fn persistence_precedes_best_effort_cache_cleanup() {
        let order = RefCell::new(Vec::new());
        let result = persist_then_cleanup(
            || {
                order.borrow_mut().push("persist");
                Ok(7)
            },
            || {
                order.borrow_mut().push("cleanup");
                Err(AppError::Platform("cache unavailable".into()))
            },
            "delete",
            "clip",
        );
        assert_eq!(result.unwrap(), 7);
        assert_eq!(*order.borrow(), ["persist", "cleanup"]);
    }
}
