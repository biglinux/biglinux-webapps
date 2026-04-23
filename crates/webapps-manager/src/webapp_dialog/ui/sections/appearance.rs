use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::WebApp;

use super::super::CATEGORIES;

pub(crate) struct AppearanceSection {
    pub icon_row: adw::ActionRow,
    pub icon_preview: gtk::Image,
    pub icon_button: gtk::Button,
    pub favicon_flow: gtk::FlowBox,
    pub category_row: adw::ComboRow,
}

pub(crate) fn build_appearance_section(webapp: &WebApp) -> AppearanceSection {
    let icon_row = adw::ActionRow::builder()
        .title(gettext("Application Icon"))
        .build();
    let icon_preview = gtk::Image::new();
    icon_preview.set_pixel_size(32);
    icon_preview.set_accessible_role(gtk::AccessibleRole::Presentation);
    crate::webapp_row::load_icon(&icon_preview, &webapp.app_icon);
    icon_row.add_prefix(&icon_preview);
    let icon_button = gtk::Button::with_label(&gettext("Select"));
    icon_button.set_valign(gtk::Align::Center);
    icon_row.add_suffix(&icon_button);
    icon_row.set_activatable_widget(Some(&icon_button));

    let favicon_flow = gtk::FlowBox::new();
    favicon_flow.set_max_children_per_line(6);
    favicon_flow.set_min_children_per_line(3);
    favicon_flow.set_homogeneous(true);
    favicon_flow.set_selection_mode(gtk::SelectionMode::Single);
    favicon_flow.set_visible(false);
    favicon_flow.update_property(&[gtk::accessible::Property::Label(&gettext(
        "Detected icon candidates",
    ))]);

    let category_model = gtk::StringList::new(CATEGORIES);
    let category_row = adw::ComboRow::builder()
        .title(gettext("Category"))
        .model(&category_model)
        .build();
    let current_category = webapp.main_category().to_string();
    if let Some(position) = CATEGORIES
        .iter()
        .position(|category| *category == current_category)
    {
        category_row.set_selected(position as u32);
    }

    AppearanceSection {
        icon_row,
        icon_preview,
        icon_button,
        favicon_flow,
        category_row,
    }
}
