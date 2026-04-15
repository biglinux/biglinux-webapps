use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;
use webapps_core::models::BrowserCollection;

/// Show browser selection dialog. Returns selected browser_id via callback.
pub fn show(
    parent: &impl IsA<gtk::Window>,
    browsers: &BrowserCollection,
    current_id: &str,
    on_selected: impl Fn(String) + 'static,
) {
    let win = adw::Window::builder()
        .title(&gettext("Select Browser"))
        .default_width(400)
        .default_height(450)
        .modal(true)
        .transient_for(parent)
        .build();

    let selected: Rc<RefCell<String>> = Rc::new(RefCell::new(current_id.to_string()));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // headerbar
    let header = adw::HeaderBar::new();
    header.set_show_end_title_buttons(true);
    content.append(&header);

    // scrollable list
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let listbox = gtk::ListBox::new();
    listbox.set_selection_mode(gtk::SelectionMode::None);
    listbox.add_css_class("boxed-list");
    listbox.set_margin_top(12);
    listbox.set_margin_bottom(12);
    listbox.set_margin_start(12);
    listbox.set_margin_end(12);

    let check_group: Rc<RefCell<Vec<gtk::CheckButton>>> = Rc::new(RefCell::new(Vec::new()));

    for browser in &browsers.browsers {
        let row = adw::ActionRow::builder()
            .title(browser.display_name())
            .activatable(true)
            .build();

        // browser icon
        let icon = gtk::Image::from_icon_name(&browser.icon_name());
        icon.set_pixel_size(32);
        row.add_prefix(&icon);

        // radio check button
        let check = gtk::CheckButton::new();
        if browser.browser_id == current_id {
            check.set_active(true);
        }

        // link to first in group for radio behavior
        {
            let group = check_group.borrow();
            if let Some(first) = group.first() {
                check.set_group(Some(first));
            }
        }
        check_group.borrow_mut().push(check.clone());

        let sel = selected.clone();
        let bid = browser.browser_id.clone();
        check.connect_toggled(move |btn| {
            if btn.is_active() {
                *sel.borrow_mut() = bid.clone();
            }
        });

        row.add_suffix(&check);
        row.set_activatable_widget(Some(&check));
        listbox.append(&row);
    }

    scroll.set_child(Some(&listbox));
    content.append(&scroll);

    // buttons
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_top(8);
    btn_box.set_margin_bottom(12);
    btn_box.set_margin_end(12);

    let cancel_btn = gtk::Button::with_label(&gettext("Cancel"));
    let ok_btn = gtk::Button::with_label(&gettext("OK"));
    ok_btn.add_css_class("suggested-action");

    {
        let w = win.clone();
        cancel_btn.connect_clicked(move |_| w.close());
    }
    {
        let w = win.clone();
        let sel = selected.clone();
        ok_btn.connect_clicked(move |_| {
            let id = sel.borrow().clone();
            on_selected(id);
            w.close();
        });
    }

    btn_box.append(&cancel_btn);
    btn_box.append(&ok_btn);
    content.append(&btn_box);

    // ESC to close
    let esc = gtk::EventControllerKey::new();
    {
        let w = win.clone();
        esc.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                w.close();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
    }
    win.add_controller(esc);

    win.set_content(Some(&content));
    win.present();
}
