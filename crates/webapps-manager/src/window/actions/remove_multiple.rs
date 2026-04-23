use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::WebApp;

use crate::{geometry, service, ui_async, webapp_row};

use super::super::context::WindowContext;
use super::super::list;
use super::super::state;

pub(super) fn install(context: &WindowContext) {
    let action = gtk::gio::SimpleAction::new("remove-multiple", None);
    let context_ref = context.clone();
    action.connect_activate(move |_, _| open_dialog(&context_ref));
    context.window.add_action(&action);
}

fn open_dialog(context: &WindowContext) {
    let webapps = state::webapps_snapshot(&context.state);
    if webapps.is_empty() {
        context.show_toast(&gettext("No WebApps to remove"));
        return;
    }

    let dialog = adw::Dialog::builder()
        .title(gettext("Remove WebApps"))
        .build();
    geometry::bind_adw_dialog(&dialog, "remove-multiple-dialog.json", 480, 560);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let hint = gtk::Label::new(Some(&gettext(
        "Select the WebApps you want to remove. This cannot be undone.",
    )));
    hint.set_wrap(true);
    hint.set_xalign(0.0);
    hint.add_css_class("dim-label");
    hint.set_margin_start(12);
    hint.set_margin_end(12);
    hint.set_margin_top(12);
    hint.set_margin_bottom(6);
    root.append(&hint);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let group = adw::PreferencesGroup::new();
    group.set_margin_start(12);
    group.set_margin_end(12);
    group.set_margin_top(6);
    group.set_margin_bottom(12);

    let select_all = gtk::CheckButton::with_label(&gettext("Select all"));
    let select_all_row = adw::ActionRow::builder().activatable(true).build();
    select_all_row.set_title(&gettext("Select all"));
    select_all_row.add_prefix(&select_all);
    select_all_row.set_activatable_widget(Some(&select_all));
    group.add(&select_all_row);

    let selected: Rc<RefCell<Vec<(WebApp, gtk::CheckButton)>>> = Rc::new(RefCell::new(Vec::new()));

    for app in &webapps {
        let row = adw::ActionRow::builder()
            .title(glib_markup_escape(&app.app_name))
            .subtitle(glib_markup_escape(&app.app_url))
            .activatable(true)
            .build();

        let icon = gtk::Image::new();
        icon.set_pixel_size(32);
        icon.set_accessible_role(gtk::AccessibleRole::Presentation);
        webapp_row::load_icon(&icon, &app.app_icon);
        row.add_prefix(&icon);

        let check = gtk::CheckButton::new();
        row.add_suffix(&check);
        row.set_activatable_widget(Some(&check));
        group.add(&row);

        selected.borrow_mut().push((app.clone(), check));
    }

    {
        let selected = selected.clone();
        select_all.connect_toggled(move |button| {
            let active = button.is_active();
            for (_, check) in selected.borrow().iter() {
                if check.is_active() != active {
                    check.set_active(active);
                }
            }
        });
    }

    {
        let select_all_ref = select_all.clone();
        let selected = selected.clone();
        for (_, check) in selected.borrow().iter() {
            let select_all_ref = select_all_ref.clone();
            let selected = selected.clone();
            check.connect_toggled(move |_| {
                let all_on = selected.borrow().iter().all(|(_, c)| c.is_active());
                let any_on = selected.borrow().iter().any(|(_, c)| c.is_active());
                select_all_ref.set_inconsistent(any_on && !all_on);
                if !select_all_ref.is_inconsistent() && select_all_ref.is_active() != all_on {
                    select_all_ref.set_active(all_on);
                }
            });
        }
    }

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.append(&group);
    scroll.set_child(Some(&container));
    root.append(&scroll);

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_top(8);
    btn_box.set_margin_bottom(12);
    btn_box.set_margin_end(12);

    let cancel_btn = gtk::Button::with_label(&gettext("Cancel"));
    let remove_btn = gtk::Button::with_label(&gettext("Remove Selected"));
    remove_btn.add_css_class("destructive-action");

    {
        let d = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            d.close();
        });
    }

    {
        let d = dialog.clone();
        let selected = selected.clone();
        let context = context.clone();
        remove_btn.connect_clicked(move |_| {
            let to_delete: Vec<WebApp> = selected
                .borrow()
                .iter()
                .filter(|(_, check)| check.is_active())
                .map(|(app, _)| app.clone())
                .collect();

            if to_delete.is_empty() {
                context.show_toast(&gettext("Select at least one WebApp"));
                return;
            }

            confirm_and_delete(&context, to_delete, d.clone());
        });
    }

    btn_box.append(&cancel_btn);
    btn_box.append(&remove_btn);
    root.append(&btn_box);

    toolbar.set_content(Some(&root));
    dialog.set_child(Some(&toolbar));
    dialog.present(Some(&*context.window));
}

fn confirm_and_delete(context: &WindowContext, to_delete: Vec<WebApp>, parent: adw::Dialog) {
    let count = to_delete.len();
    let heading = gettext("Remove Selected WebApps?");
    let body_template = gettext(
        "This will remove {count} WebApps and their desktop entries. This cannot be undone.",
    );
    let body = body_template.replace("{count}", &count.to_string());

    let confirm = adw::AlertDialog::builder()
        .heading(heading)
        .body(body)
        .build();
    confirm.add_response("cancel", &gettext("Cancel"));
    confirm.add_response("delete", &gettext("Remove"));
    confirm.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    confirm.set_default_response(Some("cancel"));
    confirm.set_close_response("cancel");

    let context_cb = context.clone();
    confirm.connect_response(None, move |_, response| {
        if response != "delete" {
            return;
        }
        let apps = to_delete.clone();
        let context_done = context_cb.clone();
        let parent_close = parent.clone();
        ui_async::run_with_result(
            move || {
                let mut succeeded = 0usize;
                let mut failed = 0usize;
                for app in &apps {
                    match service::delete_webapp(app, false) {
                        Ok(()) => succeeded += 1,
                        Err(err) => {
                            failed += 1;
                            log::error!("Failed to remove {}: {err}", app.app_name);
                        }
                    }
                }
                (succeeded, failed)
            },
            move |(succeeded, failed)| {
                list::refresh_and_render(&context_done);
                let message = if failed == 0 {
                    gettext("{count} WebApps removed").replace("{count}", &succeeded.to_string())
                } else {
                    gettext("{ok} removed, {fail} failed")
                        .replace("{ok}", &succeeded.to_string())
                        .replace("{fail}", &failed.to_string())
                };
                context_done.show_toast(&message);
                parent_close.close();
            },
        );
    });
    confirm.present(Some(&*context.window));
}

fn glib_markup_escape(value: &str) -> String {
    gtk::glib::markup_escape_text(value).to_string()
}
