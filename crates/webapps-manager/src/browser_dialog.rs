use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gettextrs::gettext;
use std::cell::RefCell;
use std::rc::Rc;
use webapps_core::models::{BrowserCollection, BrowserId};

use crate::platform::info_row::{InfoRow, InfoRowSpec};

/// User's browser selection plus viewer-only options.
pub struct BrowserSelection {
    pub browser_id: String,
    pub auto_hide_headerbar: bool,
}

type OnceCallback = Rc<RefCell<Option<Box<dyn FnOnce(BrowserSelection)>>>>;

/// Show browser selection dialog. Returns the selection via callback.
///
/// The callback fires at most once even if the user double-clicks "OK"
/// before the modal closes. When `allow_viewer` is false, the Internal
/// Browser (Built-in Viewer) option is omitted — used for DRM-required
/// templates where only external browsers can play the content.
///
/// `current_auto_hide` seeds the Auto-hide switch that is only visible
/// while Internal Browser is selected.
pub fn show(
    parent: &impl IsA<gtk::Widget>,
    browsers: &BrowserCollection,
    current_id: &str,
    current_auto_hide: bool,
    allow_viewer: bool,
    on_selected: impl FnOnce(BrowserSelection) + 'static,
) {
    let dialog = adw::Dialog::builder()
        .title(gettext("Select Browser"))
        .build();
    crate::geometry::bind_adw_dialog(&dialog, "browser-dialog.json", 400, 520);

    let selected_id: Rc<RefCell<String>> = Rc::new(RefCell::new(current_id.to_string()));
    let auto_hide: Rc<RefCell<bool>> = Rc::new(RefCell::new(current_auto_hide));

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let listbox = gtk::ListBox::new();
    listbox.set_selection_mode(gtk::SelectionMode::None);
    listbox.add_css_class("boxed-list");
    listbox.set_margin_top(6);
    listbox.set_margin_bottom(12);
    listbox.set_margin_start(12);
    listbox.set_margin_end(12);

    let check_group: Rc<RefCell<Vec<gtk::CheckButton>>> = Rc::new(RefCell::new(Vec::new()));

    // Viewer options group — only holds the auto-hide switch. Visibility
    // tracks the currently selected browser.
    let viewer_options = adw::PreferencesGroup::new();
    viewer_options.set_margin_top(12);
    viewer_options.set_margin_start(12);
    viewer_options.set_margin_end(12);

    let auto_hide_row = adw::SwitchRow::builder()
        .title(gettext("Auto-hide headerbar"))
        .subtitle(gettext(
            "Hide the titlebar; reveal it by moving the cursor to the top edge.",
        ))
        .active(current_auto_hide)
        .build();
    viewer_options.add(&auto_hide_row);
    viewer_options.set_visible(allow_viewer && current_id == BrowserId::VIEWER);

    {
        let auto_hide = auto_hide.clone();
        auto_hide_row.connect_active_notify(move |row| {
            *auto_hide.borrow_mut() = row.is_active();
        });
    }

    // Always show the internal browser; when the site needs DRM the row is
    // rendered disabled with an explanation instead of being silently omitted,
    // so the user can see the option exists and why it is unavailable.
    append_viewer_row(
        &listbox,
        &selected_id,
        &check_group,
        &viewer_options,
        current_id,
        allow_viewer,
    );

    for browser in &browsers.browsers {
        let row = InfoRow::new(InfoRowSpec::new(browser.display_name())).into_root();
        row.set_activatable(true);

        let icon = gtk::Image::new();
        icon.set_pixel_size(32);
        icon.set_accessible_role(gtk::AccessibleRole::Presentation);
        crate::webapp_row::load_icon(&icon, &browser.icon_name());
        row.add_prefix(&icon);

        let check = gtk::CheckButton::new();
        if browser.matches_id(current_id) {
            check.set_active(true);
        }
        {
            let group = check_group.borrow();
            if let Some(first) = group.first() {
                check.set_group(Some(first));
            }
        }
        check_group.borrow_mut().push(check.clone());

        let sel = selected_id.clone();
        let bid = browser.browser_id.clone();
        let viewer_options_ref = viewer_options.clone();
        check.connect_toggled(move |btn| {
            if btn.is_active() {
                *sel.borrow_mut() = bid.clone();
                viewer_options_ref.set_visible(false);
            }
        });

        row.add_suffix(&check);
        row.set_activatable_widget(Some(&check));
        listbox.append(&row);
    }

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    scroll.set_child(Some(&{
        let inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
        inner.append(&viewer_options);
        inner.append(&listbox);
        inner
    }));
    content.append(&scroll);

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);
    btn_box.set_margin_top(8);
    btn_box.set_margin_bottom(12);
    btn_box.set_margin_end(12);

    let cancel_btn = gtk::Button::with_label(&gettext("Cancel"));
    let ok_btn = gtk::Button::with_label(&gettext("OK"));
    ok_btn.add_css_class("suggested-action");

    {
        let d = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            d.close();
        });
    }
    {
        let d = dialog.clone();
        let sel = selected_id.clone();
        let auto_hide = auto_hide.clone();
        let on_selected_cell: OnceCallback = Rc::new(RefCell::new(Some(Box::new(on_selected))));
        ok_btn.connect_clicked(move |_| {
            let Some(cb) = on_selected_cell.borrow_mut().take() else {
                return;
            };
            let selection = BrowserSelection {
                browser_id: sel.borrow().clone(),
                auto_hide_headerbar: *auto_hide.borrow(),
            };
            cb(selection);
            let _ = d.close();
        });
    }
    dialog.set_default_widget(Some(&ok_btn));

    btn_box.append(&cancel_btn);
    btn_box.append(&ok_btn);
    content.append(&btn_box);

    toolbar.set_content(Some(&content));
    dialog.set_child(Some(&toolbar));
    dialog.present(Some(parent));
}

/// Append the "Internal Browser" choice.
///
/// `enabled` is false for DRM sites: the row is shown but rendered insensitive
/// with an explanatory subtitle (the WebKit viewer ships no Widevine, so DRM
/// content cannot play there). Showing it disabled — rather than omitting it —
/// tells the user the option exists and why it is unavailable.
fn append_viewer_row(
    listbox: &gtk::ListBox,
    selected: &Rc<RefCell<String>>,
    check_group: &Rc<RefCell<Vec<gtk::CheckButton>>>,
    viewer_options: &adw::PreferencesGroup,
    current_id: &str,
    enabled: bool,
) {
    let subtitle = if enabled {
        gettext("Embedded browser with better system integration. May not work on all websites.")
    } else {
        gettext("Unavailable: this site requires DRM, which the internal browser does not support.")
    };
    let row =
        InfoRow::new(InfoRowSpec::new(gettext("Internal Browser")).subtitle(subtitle)).into_root();
    row.set_activatable(enabled);
    row.set_sensitive(enabled);

    let icon = gtk::Image::new();
    icon.set_pixel_size(32);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    crate::webapp_row::load_icon(&icon, "big-webapps");
    row.add_prefix(&icon);

    let check = gtk::CheckButton::new();
    check.set_sensitive(enabled);
    if enabled && current_id == BrowserId::VIEWER {
        check.set_active(true);
    }
    check_group.borrow_mut().push(check.clone());

    if enabled {
        let sel = selected.clone();
        let viewer_options = viewer_options.clone();
        check.connect_toggled(move |btn| {
            if btn.is_active() {
                *sel.borrow_mut() = BrowserId::VIEWER.to_string();
                viewer_options.set_visible(true);
            }
        });
        row.set_activatable_widget(Some(&check));
    }

    row.add_suffix(&check);
    listbox.append(&row);
}
