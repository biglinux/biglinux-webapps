use gtk::prelude::*;
use gtk4 as gtk;

pub fn set(widget: &impl IsA<gtk::Widget>, text: &str) {
    widget.set_tooltip_text(Some(text));
}
