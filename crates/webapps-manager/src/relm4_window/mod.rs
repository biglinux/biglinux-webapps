//! Relm4 surface for the webapps manager (onda6 migration).
//!
//! Currently provides:
//! - [`list::WebAppListController`] — typed list controller backed by
//!   [`relm4::factory::FactoryVecDeque`]; supersedes the imperative
//!   `window::list::populate_list` flow.
//! - [`row::WebAppRowFactory`] — typed `FactoryComponent` for a single
//!   webapp row (replaces `webapp_row::build_row`).
//! - [`section::WebAppSectionFactory`] — per-category preferences group.
//! - [`empty`] — `BigEmptyStateSpec`-driven empty surface.
//! - [`shell_spec`] — canonical `BigShellSpec` for the manager window.
//!
//! TODO(onda6): full `BigShell`-based main window + `BigDialogSpec` dialogs.

pub mod empty;
pub mod list;
pub mod row;
pub mod section;
pub mod shell_spec;
