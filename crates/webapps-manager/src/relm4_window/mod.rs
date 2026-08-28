//! Relm4 surface for the webapps manager.
//!
//! Currently provides:
//! - [`list::WebAppListController`] — typed list controller backed by
//!   [`relm4::factory::FactoryVecDeque`]; supersedes the imperative
//!   `window::list::populate_list` flow.
//! - [`row::WebAppRowFactory`] — typed `FactoryComponent` for a single
//!   webapp row.
//! - [`section::WebAppSectionFactory`] — per-category preferences group.
//! - [`empty`] — empty-state surface.
//! - [`shell_spec`] — shell metadata for the manager window.

pub mod empty;
pub mod list;
pub mod row;
pub mod section;
pub mod shell_spec;
