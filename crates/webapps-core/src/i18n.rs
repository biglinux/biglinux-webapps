use gettextrs::{bindtextdomain, setlocale, textdomain, LocaleCategory};

const GETTEXT_DOMAIN: &str = "biglinux-webapps";

/// Init gettext i18n — call once at startup before any UI
pub fn init() {
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_DOMAIN, crate::config::share_dir().join("locale")).ok();
    textdomain(GETTEXT_DOMAIN).ok();
}
