use gtk::prelude::*;
use gtk4 as gtk;

pub fn set(widget: &impl IsA<gtk::Widget>, text: &str) {
    widget.set_tooltip_text(Some(text));
}

pub fn update(widget: &impl IsA<gtk::Widget>, text: &str) {
    set(widget, text);
}

pub fn clear(widget: &impl IsA<gtk::Widget>) {
    widget.set_tooltip_text(None);
}
