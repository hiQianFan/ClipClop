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
    let _guard = preview.lock_lifecycle()?;
    cleanup_then_persist(|| preview.delete_cached(app, id), || history.delete(id))
}

pub fn clear_history(
    app: &AppHandle,
    history: &HistoryService,
    preview: &PreviewService,
) -> AppResult<u64> {
    let _guard = preview.lock_lifecycle()?;
    cleanup_then_persist(|| preview.clear_cached(app), || history.clear())
}

pub fn copy_clip(history: &HistoryService, id: &str, plain_text_only: bool) -> AppResult<()> {
    SystemClipboard::write(history.flavors(id)?, plain_text_only)
}

fn cleanup_then_persist<T>(
    cleanup: impl FnOnce() -> AppResult<()>,
    persist: impl FnOnce() -> AppResult<T>,
) -> AppResult<T> {
    cleanup()?;
    persist()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use std::cell::RefCell;

    #[test]
    fn cleanup_failure_preserves_persisted_history() {
        let order = RefCell::new(Vec::new());
        let result = cleanup_then_persist(
            || {
                order.borrow_mut().push("cleanup");
                Err(AppError::Platform("cache unavailable".into()))
            },
            || {
                order.borrow_mut().push("persist");
                Ok(7)
            },
        );
        assert!(result.is_err());
        assert_eq!(*order.borrow(), ["cleanup"]);
    }
}
