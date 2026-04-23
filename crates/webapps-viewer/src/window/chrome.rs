use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;
use webkit6 as webkit;

use super::settings;

pub(super) struct ViewerChrome {
    pub title_widget: adw::WindowTitle,
    pub back_btn: gtk::Button,
    pub forward_btn: gtk::Button,
    pub reload_btn: gtk::Button,
    pub fullscreen_btn: gtk::Button,
    pub url_entry: gtk::Entry,
    pub url_bar: gtk::Revealer,
    pub toolbar: adw::ToolbarView,
}

pub(super) fn build_chrome(name: &str, url: &str, webview: &webkit::WebView) -> ViewerChrome {
    let title_widget = adw::WindowTitle::new(name, url);

    let back_btn = navigation_button("go-previous-symbolic", &gettext("Back"), false);
    let forward_btn = navigation_button("go-next-symbolic", &gettext("Forward"), false);
    let reload_btn = navigation_button("view-refresh-symbolic", &gettext("Reload"), true);
    let fullscreen_btn =
        navigation_button("view-fullscreen-symbolic", &gettext("Fullscreen"), true);

    let menu_btn = build_menu_button();

    let header = adw::HeaderBar::builder()
        .title_widget(&title_widget)
        .build();
    header.pack_start(&back_btn);
    header.pack_start(&forward_btn);
    header.pack_start(&reload_btn);
    header.pack_end(&menu_btn);
    header.pack_end(&fullscreen_btn);

    let url_entry = gtk::Entry::builder()
        .placeholder_text(gettext("Enter URL…"))
        .hexpand(true)
        .build();
    url_entry.set_text(url);

    let url_bar = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideDown)
        .reveal_child(false)
        .child(&url_entry)
        .build();

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.add_top_bar(&url_bar);
    toolbar.set_content(Some(webview));
    toolbar.set_reveal_top_bars(true);

    ViewerChrome {
        title_widget,
        back_btn,
        forward_btn,
        reload_btn,
        fullscreen_btn,
        url_entry,
        url_bar,
        toolbar,
    }
}

fn navigation_button(icon_name: &str, label: &str, is_sensitive: bool) -> gtk::Button {
    let button = gtk::Button::from_icon_name(icon_name);
    button.set_sensitive(is_sensitive);
    button.set_tooltip_text(Some(label));
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
}

fn build_menu_button() -> gtk::MenuButton {
    use gtk::gio;

    let menu = gio::Menu::new();
    menu.append(Some(&gettext("Zoom In")), Some("win.zoom-in"));
    menu.append(Some(&gettext("Zoom Out")), Some("win.zoom-out"));
    menu.append(Some(&gettext("Reset Zoom")), Some("win.zoom-reset"));
    if settings::DEVELOPER_TOOLS_ENABLED {
        menu.append(Some(&gettext("Developer Tools")), Some("win.devtools"));
    }

    let help = gio::Menu::new();
    help.append(Some(&gettext("Keyboard Shortcuts")), Some("win.shortcuts"));
    menu.append_section(None, &help);

    let button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .tooltip_text(gettext("Menu"))
        .build();
    button.update_property(&[gtk::accessible::Property::Label(&gettext("Menu"))]);
    button
}
