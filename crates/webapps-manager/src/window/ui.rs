use crate::platform::tooltip;
use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;

/// The curated template gallery is wired but not launched yet. Flip to `true`
/// to surface the header "Templates" button (the gallery + its handler stay
/// fully wired behind it). Keep `false` until the feature ships.
const TEMPLATES_FEATURE_ENABLED: bool = false;

/// Header controls the [`super::component::ManagerWindow`] wires to messages.
pub(super) struct WindowControls {
    pub search_btn: gtk::ToggleButton,
    pub add_btn: gtk::Button,
    pub templates_btn: gtk::Button,
    pub search_entry: gtk::SearchEntry,
}

/// Populate `toast_overlay` (the component `Root`) with the manager chrome and
/// mount the Relm4 list widget inside the scrolled clamp. The window itself is
/// created by the launcher ([`super::build`]) and owns this content — keeping
/// the chrome window-agnostic (ADR-D14 mountable content).
///
/// `list_widget` is the root widget of the Relm4 `WebAppListController`.
pub(super) fn build_content(
    toast_overlay: &adw::ToastOverlay,
    list_widget: &gtk::Widget,
) -> WindowControls {
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let header = adw::HeaderBar::new();

    let search_btn = gtk::ToggleButton::new();
    search_btn.set_icon_name("system-search-symbolic");
    tooltip::set(&search_btn, &gettext("Search WebApps"));
    search_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Search WebApps"))]);
    header.pack_start(&search_btn);

    let add_btn = gtk::Button::with_label(&gettext("Add"));
    tooltip::set(&add_btn, &gettext("Add WebApp"));
    add_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Add WebApp"))]);
    add_btn.add_css_class("suggested-action");
    header.pack_start(&add_btn);

    let templates_btn = gtk::Button::from_icon_name("view-grid-symbolic");
    tooltip::set(&templates_btn, &gettext("Choose from templates"));
    templates_btn.update_property(&[gtk::accessible::Property::Label(&gettext(
        "Choose from templates",
    ))]);
    if TEMPLATES_FEATURE_ENABLED {
        header.pack_start(&templates_btn);
    }

    let menu_btn = gtk::MenuButton::new();
    menu_btn.set_icon_name("open-menu-symbolic");
    menu_btn.set_menu_model(Some(&build_main_menu()));
    menu_btn.update_property(&[gtk::accessible::Property::Label(&gettext("Main Menu"))]);
    header.pack_end(&menu_btn);

    main_box.append(&header);

    let search_bar = gtk::SearchBar::new();
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_hexpand(true);
    search_bar.set_child(Some(&search_entry));
    search_bar.connect_entry(&search_entry);
    search_btn
        .bind_property("active", &search_bar, "search-mode-enabled")
        .bidirectional()
        .build();
    main_box.append(&search_bar);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(900);
    clamp.set_tightening_threshold(720);
    clamp.set_child(Some(list_widget));
    scroll.set_child(Some(&clamp));
    main_box.append(&scroll);

    let status_label = gtk::Label::new(None);
    status_label.set_visible(false);
    status_label.set_accessible_role(gtk::AccessibleRole::Status);
    status_label.add_css_class("dim-label");
    status_label.add_css_class("caption");
    status_label.set_margin_bottom(6);
    main_box.append(&status_label);

    toast_overlay.set_child(Some(&main_box));

    WindowControls {
        search_btn,
        add_btn,
        templates_btn,
        search_entry,
    }
}

fn build_main_menu() -> gtk::gio::Menu {
    let menu = gtk::gio::Menu::new();
    menu.append(Some(&gettext("Import WebApps")), Some("win.import"));
    menu.append(Some(&gettext("Export WebApps")), Some("win.export"));
    menu.append(
        Some(&gettext("Browse Applications Folder")),
        Some("win.browse-apps"),
    );
    menu.append(
        Some(&gettext("Browse Profiles Folder")),
        Some("win.browse-profiles"),
    );

    let danger = gtk::gio::Menu::new();
    danger.append(
        Some(&gettext("Remove WebApps in Bulk")),
        Some("win.remove-multiple"),
    );
    danger.append(Some(&gettext("Remove All WebApps")), Some("win.remove-all"));
    menu.append_section(None, &danger);

    let help = gtk::gio::Menu::new();
    help.append(Some(&gettext("Keyboard Shortcuts")), Some("win.shortcuts"));
    help.append(Some(&gettext("About")), Some("win.about"));
    menu.append_section(None, &help);

    menu
}
