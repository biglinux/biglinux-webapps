use gtk4 as gtk;
use gtk4::gdk as gdk4;

const CSS: &str = r#"
/* App icon in list rows — soft rounded corners for a polished look */
.webapp-icon {
    border-radius: 10px;
}

/* Empty-state icon is decorative; soften its presence */
.empty-state-icon {
    opacity: 0.72;
}

/* Segmented action bar on webapp rows — three buttons share a rounded
   container divided by thin separators. */
.webapp-actions {
    padding: 0;
}
.webapp-actions > button {
    min-width: 34px;
    min-height: 32px;
    padding: 4px 10px;
    background-color: alpha(currentColor, 0.07);
    box-shadow: none;
    border: none;
    border-radius: 0;
    transition: background-color 140ms ease-out;
}
.webapp-actions > button:hover {
    background-color: alpha(currentColor, 0.14);
}
.webapp-actions > button:active {
    background-color: alpha(currentColor, 0.20);
}
.webapp-actions > button:not(:last-child) {
    border-right: 1px solid alpha(currentColor, 0.10);
}
.webapp-actions > button:first-child {
    border-top-left-radius: 8px;
    border-bottom-left-radius: 8px;
}
.webapp-actions > button:last-child {
    border-top-right-radius: 8px;
    border-bottom-right-radius: 8px;
}
.webapp-actions > button.destructive {
    color: @error_color;
}
.webapp-actions > button.destructive:hover {
    background-color: alpha(@error_color, 0.15);
}
.webapp-actions > button.destructive:active {
    background-color: alpha(@error_color, 0.25);
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
