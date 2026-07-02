use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk4 as gtk;

pub fn install_action<T, F>(target: &T, name: &str, callback: F) -> gio::SimpleAction
where
    T: IsA<gio::ActionMap>,
    F: Fn() + 'static,
{
    let action = gio::SimpleAction::new(name, None);
    action.connect_activate(move |_, _| callback());
    target.add_action(&action);
    action
}

pub fn install_string_action<T, F>(target: &T, name: &str, callback: F) -> gio::SimpleAction
where
    T: IsA<gio::ActionMap>,
    F: Fn(String) + 'static,
{
    let action = gio::SimpleAction::new(name, Some(&String::static_variant_type()));
    action.connect_activate(move |_, parameter| {
        let Some(value) = parameter.and_then(glib::Variant::str) else {
            return;
        };
        callback(value.to_string());
    });
    target.add_action(&action);
    action
}
