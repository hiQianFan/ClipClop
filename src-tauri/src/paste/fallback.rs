use super::{PasteOutcome, PasteTarget};

pub(super) fn can_inject() -> bool {
    true
}

pub(super) fn capture_target() -> Option<PasteTarget> {
    None
}

pub(super) fn paste(_: PasteTarget) -> PasteOutcome {
    PasteOutcome::CopiedUnsupportedPlatform
}
