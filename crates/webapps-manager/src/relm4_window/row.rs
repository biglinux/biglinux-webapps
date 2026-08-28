//! Typed Relm4 factory row for a single webapp entry.
//!
//! Replaces the imperative `webapp_row::build_row` callback wiring with a
//! `FactoryComponent`. Each row owns a clone of the [`WebApp`] model; user
//! actions become typed `WebAppRowOutput` messages routed to the parent
//! list component.

use libadwaita as adw;
use relm4::adw::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::gtk;

use webapps_core::models::{AppMode, Browser, WebApp};

use crate::platform::info_row::{InfoRow, InfoRowSpec};
use crate::platform::tooltip;
use crate::webapp_row::load_icon;

/// Init payload for [`WebAppRowFactory`].
#[derive(Debug, Clone)]
pub struct WebAppRowInit {
    /// The webapp model that backs this row.
    pub webapp: WebApp,
}

/// Outputs emitted by a webapp row (bubble up to list/window controller).
#[derive(Debug, Clone)]
pub enum WebAppRowOutput {
    /// User pressed the "change browser" segmented button.
    BrowserRequested(WebApp),
    /// User pressed the edit button.
    EditRequested(WebApp),
    /// User pressed the delete button.
    DeleteRequested(WebApp),
}

/// Factory model: holds the webapp clone the row was constructed from.
#[derive(Debug)]
pub struct WebAppRowFactory {
    webapp: WebApp,
}

// caveman: row inputs unused — all interactions emit outputs directly.
#[derive(Debug)]
pub enum WebAppRowInput {}

impl FactoryComponent for WebAppRowFactory {
    type Init = WebAppRowInit;
    type Input = WebAppRowInput;
    type Output = WebAppRowOutput;
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;
    type Root = adw::ActionRow;
    type Widgets = ();
    type Index = DynamicIndex;

    fn init_root(&self) -> Self::Root {
        let row = InfoRow::new(
            InfoRowSpec::new(glib::markup_escape_text(&self.webapp.app_name).to_string())
                .subtitle(
                    glib::markup_escape_text(&crate::service::display_url(&self.webapp.app_url))
                        .to_string(),
                )
                .allow_markup(),
        )
        .into_root();
        row.set_activatable(false);
        row
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            webapp: init.webapp,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        // prefix icon
        let icon = gtk::Image::new();
        icon.set_pixel_size(40);
        icon.add_css_class("webapp-icon");
        icon.set_accessible_role(gtk::AccessibleRole::Presentation);
        load_icon(&icon, &self.webapp.app_icon);
        root.add_prefix(&icon);

        // segmented action group
        let group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        group.set_valign(gtk::Align::Center);
        group.add_css_class("linked");
        group.add_css_class("webapp-actions");

        group.append(&build_browser_button(&self.webapp, &sender));
        group.append(&build_edit_button(&self.webapp, &sender));
        group.append(&build_delete_button(&self.webapp, &sender));
        root.add_suffix(&group);
    }

    fn update(&mut self, _msg: Self::Input, _sender: FactorySender<Self>) {}
}

fn build_browser_button(webapp: &WebApp, sender: &FactorySender<WebAppRowFactory>) -> gtk::Button {
    // caveman: viewer mode → bundled paintable. else browser icon.
    let icon_name = if webapp.app_mode == AppMode::App {
        "big-webapps".to_string()
    } else {
        Browser {
            browser_id: webapp.browser.clone(),
            is_default: false,
        }
        .icon_name()
    };
    let label = gettextrs::gettext("Change browser");
    let button = make_button_with_loaded_icon(&icon_name, &label);
    let app = webapp.clone();
    let sender = sender.clone();
    button.connect_clicked(move |_| {
        let _ = sender.output(WebAppRowOutput::BrowserRequested(app.clone()));
    });
    button
}

fn build_edit_button(webapp: &WebApp, sender: &FactorySender<WebAppRowFactory>) -> gtk::Button {
    let label = gettextrs::gettext("Edit");
    let button = make_button("document-edit-symbolic", &label);
    let app = webapp.clone();
    let sender = sender.clone();
    button.connect_clicked(move |_| {
        let _ = sender.output(WebAppRowOutput::EditRequested(app.clone()));
    });
    button
}

fn build_delete_button(webapp: &WebApp, sender: &FactorySender<WebAppRowFactory>) -> gtk::Button {
    let label = gettextrs::gettext("Remove");
    let button = make_button("user-trash-symbolic", &label);
    button.add_css_class("destructive-action");
    let app = webapp.clone();
    let sender = sender.clone();
    button.connect_clicked(move |_| {
        let _ = sender.output(WebAppRowOutput::DeleteRequested(app.clone()));
    });
    button
}

fn make_button(icon_name: &str, label: &str) -> gtk::Button {
    let image = gtk::Image::from_icon_name(icon_name);
    image.set_pixel_size(22);
    finalize_button(image, label)
}

fn make_button_with_loaded_icon(icon_name: &str, label: &str) -> gtk::Button {
    let image = gtk::Image::new();
    image.set_pixel_size(24);
    load_icon(&image, icon_name);
    finalize_button(image, label)
}

fn finalize_button(image: gtk::Image, label: &str) -> gtk::Button {
    image.set_accessible_role(gtk::AccessibleRole::Presentation);
    let button = gtk::Button::builder().child(&image).build();
    button.set_valign(gtk::Align::Center);
    tooltip::set(&button, label);
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
}

// pull glib markup escape through relm4 re-exports
use relm4::gtk::glib;
