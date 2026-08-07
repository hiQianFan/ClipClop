use super::{PasteOutcome, PasteTarget};

pub(super) fn capture_target() -> Option<PasteTarget> {
    None
}

pub(super) fn paste(_: PasteTarget) -> PasteOutcome {
    PasteOutcome::CopiedUnsupportedPlatform
}
