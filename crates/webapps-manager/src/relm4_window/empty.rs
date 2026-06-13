//! Empty-state surface built from `BigEmptyStateSpec`.
//!
//! The spec lives in `big-relm4-components::state::empty`; this module turns
//! the spec into the concrete `adw::StatusPage` used by the webapps manager
//! when the catalog is empty.

use big_relm4_components::display::status_page::build_empty_state;
use big_relm4_components::state::empty::BigEmptyStateSpec;
use gettextrs::gettext;
use libadwaita as adw;
use relm4::adw::prelude::*;
use relm4::gtk;

/// Build the manager's empty-state spec.
///
/// The CTA is left unbound — the consumer wires `connect_clicked` on the
/// returned button to whatever "add webapp" path is appropriate.
#[must_use]
pub fn build_spec() -> BigEmptyStateSpec {
    BigEmptyStateSpec::new("big-webapps", gettext("No WebApps yet"))
        .with_body(gettext(
            "Turn any website into a desktop app. Press Add to get started.",
        ))
        .with_action(gettext("Add WebApp"), "win.add-webapp")
}

/// Realise a spec into an [`adw::StatusPage`] + optional CTA button.
///
/// Thin wrapper over the catalog renderer
/// [`big_relm4_components::display::status_page::build_empty_state`]; adds the
/// webapps-specific `empty-state-icon` style. Returns the page (already
/// containing the button as child) and the button itself so callers can wire
/// signals without re-walking the widget tree.
#[must_use]
pub fn build_page(spec: &BigEmptyStateSpec) -> (adw::StatusPage, Option<gtk::Button>) {
    let (page, button) = build_empty_state(spec);
    page.add_css_class("empty-state-icon");
    (page, button)
}
