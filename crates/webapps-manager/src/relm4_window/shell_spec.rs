//! Canonical [`BigShellSpec`] description for the webapps manager.
//!
//! `window::build` consumes this spec for the window title and default
//! geometry (single source of truth). The chrome widget tree is still built
//! imperatively in `window/ui.rs`; the remaining fields (header policy, toast
//! usage) document the intended contract so a future migration to a
//! `BigShell`-based builder can derive the rest of the GTK tree from the spec.

use big_app_kit::shell::{BigHeaderPolicy, BigShellKind, BigShellSpec};
use webapps_core::config;

/// Webapps manager shell spec.
///
/// - `kind = SingleWindow`: one window, no sidebar, no tabs.
/// - `header_policy = Unified`: single header bar.
/// - `uses_toasts = true`: toast overlay drives success/error feedback.
#[must_use]
pub fn build() -> BigShellSpec {
    BigShellSpec {
        app_id: config::APP_ID.to_string(),
        title: gettextrs::gettext("WebApps Manager"),
        kind: BigShellKind::SingleWindow,
        header_policy: BigHeaderPolicy::Unified,
        default_width: 820,
        default_height: 680,
        uses_toasts: true,
        uses_sidebar: false,
        uses_tabs: false,
        uses_footer: false,
    }
}
