use std::{cell::Cell, rc::Rc, sync::Once, time::Duration};

use big_relm4_components::theme;
use glib::clone;
use gtk4 as gtk;
use gtk4::prelude::*;
use webkit6 as webkit;
use webkit6::prelude::*;

const CONTENT_REVEAL_DELAY_MS: u64 = 250;

const CSS: &str = r#"
.viewer-loading {
    background-color: #03040a;
}

.viewer-loading-spinner {
    color: #d4d5df;
    min-width: 72px;
    min-height: 72px;
}
"#;

static CSS_LOADED: Once = Once::new();

pub(super) fn build_loading_overlay(webview: &webkit::WebView) -> gtk::Overlay {
    theme::load_app_css_once(CSS, &CSS_LOADED);

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(webview));

    let veil = gtk::Box::new(gtk::Orientation::Vertical, 0);
    veil.add_css_class("viewer-loading");
    veil.set_can_target(false);
    veil.set_halign(gtk::Align::Fill);
    veil.set_valign(gtk::Align::Fill);
    veil.set_hexpand(true);
    veil.set_vexpand(true);

    let spinner = gtk::Spinner::new();
    spinner.add_css_class("viewer-loading-spinner");
    spinner.set_can_target(false);
    spinner.set_halign(gtk::Align::Center);
    spinner.set_valign(gtk::Align::Center);
    spinner.start();

    overlay.add_overlay(&veil);
    overlay.add_overlay(&spinner);

    let generation = Rc::new(Cell::new(0_u64));
    connect_loading_state(webview, &veil, &spinner, &generation);
    sync_initial_state(webview, &veil, &spinner, &generation);

    overlay
}

fn connect_loading_state(
    webview: &webkit::WebView,
    veil: &gtk::Box,
    spinner: &gtk::Spinner,
    generation: &Rc<Cell<u64>>,
) {
    webview.connect_load_changed(clone!(
        #[weak]
        veil,
        #[weak]
        spinner,
        #[strong]
        generation,
        move |_, event| match event {
            webkit::LoadEvent::Started | webkit::LoadEvent::Redirected => {
                show_loading(&veil, &spinner, &generation);
            }
            webkit::LoadEvent::Committed => schedule_hide(&veil, &spinner, &generation),
            webkit::LoadEvent::Finished => {
                hide_loading(&veil, &spinner);
            }
            _ => {}
        }
    ));

    webview.connect_load_failed(clone!(
        #[weak]
        veil,
        #[weak]
        spinner,
        #[strong]
        generation,
        #[upgrade_or]
        false,
        move |_, _, _, _| {
            generation.set(generation.get().wrapping_add(1));
            hide_loading(&veil, &spinner);
            false
        }
    ));

    webview.connect_estimated_load_progress_notify(clone!(
        #[weak]
        veil,
        #[weak]
        spinner,
        #[strong]
        generation,
        move |wv| {
            if wv.estimated_load_progress() >= 0.35 {
                schedule_hide(&veil, &spinner, &generation);
            }
        }
    ));
}

fn sync_initial_state(
    webview: &webkit::WebView,
    veil: &gtk::Box,
    spinner: &gtk::Spinner,
    generation: &Rc<Cell<u64>>,
) {
    if webview.is_loading() || webview.estimated_load_progress() < 1.0 {
        show_loading(veil, spinner, generation);
    } else {
        hide_loading(veil, spinner);
    }
}

fn show_loading(veil: &gtk::Box, spinner: &gtk::Spinner, generation: &Rc<Cell<u64>>) {
    generation.set(generation.get().wrapping_add(1));
    veil.set_visible(true);
    spinner.set_visible(true);
    spinner.start();
}

fn schedule_hide(veil: &gtk::Box, spinner: &gtk::Spinner, generation: &Rc<Cell<u64>>) {
    let expected_generation = generation.get();
    glib::timeout_add_local_once(
        Duration::from_millis(CONTENT_REVEAL_DELAY_MS),
        clone!(
            #[weak]
            veil,
            #[weak]
            spinner,
            #[strong]
            generation,
            move || {
                if generation.get() == expected_generation {
                    hide_loading(&veil, &spinner);
                }
            }
        ),
    );
}

fn hide_loading(veil: &gtk::Box, spinner: &gtk::Spinner) {
    spinner.stop();
    spinner.set_visible(false);
    veil.set_visible(false);
}
