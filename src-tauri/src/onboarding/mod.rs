mod model;
mod service;

pub use model::*;
pub use service::OnboardingService;

pub fn auto_paste_readiness() -> AutoPasteReadiness {
    #[cfg(target_os = "macos")]
    {
        if macos::preflight() {
            AutoPasteReadiness::Available
        } else {
            AutoPasteReadiness::PermissionRequired
        }
    }
    #[cfg(target_os = "windows")]
    {
        AutoPasteReadiness::AvailableWithElevatedTargetLimit
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        AutoPasteReadiness::Unsupported
    }
}

pub fn request_auto_paste_access() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::request()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[cfg(target_os = "macos")]
mod macos {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGPreflightPostEventAccess() -> bool;
        fn CGRequestPostEventAccess() -> bool;
    }

    pub fn preflight() -> bool {
        unsafe { CGPreflightPostEventAccess() }
    }

    pub fn request() -> bool {
        unsafe { CGRequestPostEventAccess() }
    }
}
