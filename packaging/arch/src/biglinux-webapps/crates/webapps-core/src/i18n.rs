use gettextrs::{bindtextdomain, setlocale, textdomain, LocaleCategory};

const GETTEXT_DOMAIN: &str = "biglinux-webapps";
const LOCALE_DIR: &str = "/usr/share/locale";

/// Init gettext i18n — call once at startup before any UI
pub fn init() {
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_DOMAIN, LOCALE_DIR).ok();
    textdomain(GETTEXT_DOMAIN).ok();
}
