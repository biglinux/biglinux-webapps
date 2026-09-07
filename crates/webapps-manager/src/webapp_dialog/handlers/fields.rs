use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::models::{AppMode, BrowserCollection, BrowserId, WebApp};
use webapps_core::templates::default_registry;

use crate::browser_dialog;

use super::super::ui::{DialogWidgets, CATEGORIES};
use super::super::validation;

pub(crate) fn setup_url_handler(
    widgets: &DialogWidgets,
    webapp_cell: Rc<RefCell<WebApp>>,
    browsers: Rc<RefCell<BrowserCollection>>,
    drm_required: Rc<Cell<bool>>,
    skip_auto_detect: Rc<Cell<bool>>,
    debounce_handle: Rc<RefCell<Option<glib::SourceId>>>,
) {
    let detect_button = widgets.detect_button.clone();
    let browser_row = widgets.browser_row.clone();
    let browser_icon = widgets.browser_icon.clone();
    let profile_row = widgets.profile_row.clone();
    widgets.url_row.connect_changed(move |row| {
        let text = row.text().to_string();
        {
            let mut webapp = webapp_cell.borrow_mut();
            webapp.app_url = text.clone();
            let requires_drm = default_registry().requires_drm(&webapp.template_id, &text);
            drm_required.set(requires_drm);
            if requires_drm && webapp.browser_id().is_viewer() {
                if let Some(browser_id) = preferred_external_browser(&browsers.borrow()) {
                    webapp.browser = browser_id;
                    webapp.app_mode = AppMode::Browser;
                }
            }
        }
        let selected_browser = webapp_cell.borrow().browser.clone();
        update_browser_row_subtitle(&browser_row, &browsers.borrow(), &selected_browser);
        super::super::ui::update_browser_icon(&browser_icon, &webapp_cell.borrow());
        profile_row.set_visible(!webapp_cell.borrow().browser_id().is_viewer());

        if let Some(id) = debounce_handle.borrow_mut().take() {
            id.remove();
        }

        if skip_auto_detect.replace(false) {
            return;
        }

        let detect_button = detect_button.clone();
        let scheduled_handle = debounce_handle.clone();
        let source =
            glib::timeout_add_local_once(std::time::Duration::from_millis(800), move || {
                scheduled_handle.borrow_mut().take();
                if validation::should_auto_detect(&text) {
                    detect_button.emit_clicked();
                }
            });
        *debounce_handle.borrow_mut() = Some(source);
    });
}

pub(crate) fn setup_name_handler(widgets: &DialogWidgets, webapp_cell: Rc<RefCell<WebApp>>) {
    widgets.name_row.connect_changed(move |row| {
        webapp_cell.borrow_mut().app_name = row.text().to_string();
    });
}

pub(crate) fn setup_category_handler(widgets: &DialogWidgets, webapp_cell: Rc<RefCell<WebApp>>) {
    widgets.category_row.connect_selected_notify(move |row| {
        let index = row.selected() as usize;
        if index < CATEGORIES.len() {
            webapp_cell
                .borrow_mut()
                .set_main_category(CATEGORIES[index]);
        }
    });
}

pub(crate) fn setup_browser_handler(
    widgets: &DialogWidgets,
    webapp_cell: Rc<RefCell<WebApp>>,
    browsers: Rc<RefCell<BrowserCollection>>,
    drm_required: Rc<Cell<bool>>,
) {
    let browser_row = widgets.browser_row.clone();
    let profile_row = widgets.profile_row.clone();
    let browser_icon = widgets.browser_icon.clone();
    let dialog = widgets.dialog.clone();
    widgets.browser_button.connect_clicked(move |_| {
        let (current_id, current_auto_hide) = {
            let app = webapp_cell.borrow();
            (app.browser.clone(), app.auto_hide_headerbar)
        };
        let browsers_snapshot = browsers.borrow().clone();
        let browser_row = browser_row.clone();
        let profile_row = profile_row.clone();
        let browser_icon = browser_icon.clone();
        let webapp_cell = webapp_cell.clone();
        let browsers = browsers.clone();
        let allow_viewer = !drm_required.get();
        browser_dialog::show(
            &dialog,
            &browsers_snapshot,
            &current_id,
            current_auto_hide,
            allow_viewer,
            move |selection| {
                apply_browser_selection(
                    &webapp_cell,
                    &browsers.borrow(),
                    &selection.browser_id,
                    selection.auto_hide_headerbar,
                );
                update_browser_row_subtitle(
                    &browser_row,
                    &browsers.borrow(),
                    &selection.browser_id,
                );
                super::super::ui::update_browser_icon(&browser_icon, &webapp_cell.borrow());
                profile_row.set_visible(selection.browser_id != BrowserId::VIEWER);
            },
        );
    });
}

pub(crate) fn setup_profile_handlers(widgets: &DialogWidgets, webapp_cell: Rc<RefCell<WebApp>>) {
    let profile_entry = widgets.profile_entry.clone();
    let expansion_webapp = webapp_cell.clone();
    widgets
        .profile_row
        .connect_enable_expansion_notify(move |row| {
            if row.enables_expansion() {
                let profile_name = expansion_webapp.borrow().derive_profile_name();
                expansion_webapp.borrow_mut().app_profile = profile_name.clone();
                profile_entry.set_text(&profile_name);
            } else {
                expansion_webapp.borrow_mut().app_profile = "Default".into();
            }
        });

    widgets.profile_entry.connect_changed(move |row| {
        let text = row.text().to_string();
        if !text.is_empty() {
            webapp_cell.borrow_mut().app_profile = text;
        }
    });
}

/// Persist a browser selection into the webapp, keeping `app_mode` in sync:
/// the built-in viewer implies `App` mode, any other browser implies `Browser`.
/// The `auto_hide_headerbar` flag is stored verbatim (only acted upon by the
/// viewer, so it's harmless to keep while an external browser is selected).
fn apply_browser_selection(
    webapp_cell: &Rc<RefCell<WebApp>>,
    browsers: &BrowserCollection,
    id: &str,
    auto_hide_headerbar: bool,
) {
    let mut webapp = webapp_cell.borrow_mut();
    webapp.auto_hide_headerbar = auto_hide_headerbar;
    if id == BrowserId::VIEWER {
        webapp.browser = BrowserId::VIEWER.to_string();
        webapp.app_mode = AppMode::App;
    } else {
        let resolved = if id.is_empty() || browsers.get_by_id(id).is_none() {
            browsers
                .default_browser()
                .map(|b| b.browser_id.clone())
                .unwrap_or_else(|| id.to_string())
        } else {
            id.to_string()
        };
        webapp.browser = resolved;
        webapp.app_mode = AppMode::Browser;
    }
}

fn update_browser_row_subtitle(
    browser_row: &adw::ActionRow,
    browsers: &BrowserCollection,
    id: &str,
) {
    let label = if id == BrowserId::VIEWER {
        gettextrs::gettext("Internal Browser")
    } else {
        browsers
            .get_by_id(id)
            .map(|b| b.display_name().to_string())
            .unwrap_or_else(|| id.to_string())
    };
    browser_row.set_subtitle(&label);
}

fn preferred_external_browser(browsers: &BrowserCollection) -> Option<String> {
    browsers
        .default_browser()
        .filter(|browser| browser.browser_id != BrowserId::VIEWER)
        .or_else(|| {
            browsers
                .browsers
                .iter()
                .find(|browser| browser.browser_id != BrowserId::VIEWER)
        })
        .map(|browser| browser.browser_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use webapps_core::models::Browser;

    #[test]
    fn confirming_a_saved_alias_preserves_its_profile_identity() {
        let browsers = BrowserCollection {
            browsers: vec![Browser {
                browser_id: "flatpak-brave-browser".into(),
                is_default: false,
            }],
            default_id: None,
        };
        let webapp = Rc::new(RefCell::new(WebApp {
            browser: "flatpak-brave".into(),
            app_profile: "Work".into(),
            ..WebApp::default()
        }));
        apply_browser_selection(&webapp, &browsers, "flatpak-brave", false);
        assert_eq!(webapp.borrow().browser, "flatpak-brave");
        assert_eq!(webapp.borrow().app_profile, "Work");
    }

    #[test]
    fn preferred_external_browser_skips_viewer_fallback() {
        let browsers = BrowserCollection {
            browsers: vec![
                Browser {
                    browser_id: BrowserId::VIEWER.to_string(),
                    is_default: true,
                },
                Browser {
                    browser_id: "brave".to_string(),
                    is_default: false,
                },
            ],
            default_id: Some(BrowserId::VIEWER.to_string()),
        };

        assert_eq!(
            preferred_external_browser(&browsers),
            Some("brave".to_string())
        );
    }
}
