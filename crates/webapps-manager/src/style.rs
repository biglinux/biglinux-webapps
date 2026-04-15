use gtk4 as gtk;
use gtk4::gdk as gdk4;

const CSS: &str = r#"
/* webapp icon — subtle rounding for modern look */
.webapp-icon {
    border-radius: 10px;
}

/* webapp row — refined spacing */
.webapp-row {
    padding: 6px 12px;
    min-height: 56px;
}

/* category header */
.category-header {
    padding-top: 18px;
    padding-bottom: 6px;
}

/* action button — circular, subtle */
.action-btn {
    border-radius: 50%;
    min-width: 36px;
    min-height: 36px;
    padding: 6px;
}

/* app mode badge */
.app-mode-badge {
    font-size: 0.7em;
    font-weight: bold;
    padding: 2px 8px;
    border-radius: 12px;
    background: alpha(@accent_bg_color, 0.15);
    color: @accent_fg_color;
}

/* empty state refinement */
.empty-state-icon {
    opacity: 0.6;
}

/* delete button hover emphasis */
.action-btn.error:hover {
    background: alpha(@error_bg_color, 0.15);
}
"#;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS);

    if let Some(display) = gdk4::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
