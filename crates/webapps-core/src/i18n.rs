use gettextrs::{bindtextdomain, setlocale, textdomain, LocaleCategory};

const GETTEXT_DOMAIN: &str = "biglinux-webapps";

/// Initialize gettext at application startup.
///
/// # Safety
/// Call before starting other threads or installing signal handlers.
pub unsafe fn init() {
    // SAFETY: The caller guarantees single-threaded startup without signal handlers.
    unsafe { setlocale(LocaleCategory::LcAll, "") };
    bindtextdomain(GETTEXT_DOMAIN, crate::config::share_dir().join("locale")).ok();
    textdomain(GETTEXT_DOMAIN).ok();
}
