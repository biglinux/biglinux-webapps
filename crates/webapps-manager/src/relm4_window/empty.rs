//! Empty-state surface for the webapps list.

use gettextrs::gettext;
use libadwaita as adw;
use relm4::adw::prelude::*;
use relm4::gtk;

use crate::platform::status_page::{build_empty_state, EmptyStateSpec};

/// Build the manager's empty-state spec.
///
/// The CTA is left unbound — the consumer wires `connect_clicked` on the
/// returned button to whatever "add webapp" path is appropriate.
#[must_use]
pub fn build_spec() -> EmptyStateSpec {
    EmptyStateSpec::new("big-webapps", gettext("No WebApps yet"))
        .with_body(gettext(
            "Turn any website into a desktop app. Press Add to get started.",
        ))
        .with_action(gettext("Add WebApp"), "win.add-webapp")
}

/// Realise a spec into an [`adw::StatusPage`] + optional CTA button.
///
/// Returns the page and the button itself so callers can wire signals without
/// re-walking the widget tree.
#[must_use]
pub fn build_page(spec: &EmptyStateSpec) -> (adw::StatusPage, Option<gtk::Button>) {
    let (page, button) = build_empty_state(spec);
    page.add_css_class("empty-state-icon");
    (page, button)
}
