use std::sync::Once;

use gtk::gdk;
use gtk4 as gtk;

pub fn load_app_css(css: &str) {
    let Some(display) = gdk::Display::default() else {
        return;
    };
    let provider = gtk::CssProvider::new();
    provider.load_from_string(css);
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn load_app_css_once(css: &'static str, once: &'static Once) {
    once.call_once(|| load_app_css(css));
}
