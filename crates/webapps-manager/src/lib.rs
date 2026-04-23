//! Library facade for the manager binary.
//!
//! Re-exports the modules that the integration tests in `tests/` need to drive
//! the CRUD pipeline against a sandboxed XDG home. The binary entry point in
//! `main.rs` continues to own its own `mod` declarations.

pub mod browser_dialog;
pub mod favicon;
pub mod geometry;
pub mod service;
pub mod style;
pub mod template_gallery;
pub mod ui_async;
pub mod webapp_dialog;
pub mod webapp_row;
pub mod welcome_dialog;
pub mod window;
