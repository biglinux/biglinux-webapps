//! Value types for the `WebApp` model: mode, categories, identifiers, URL
//! type, and shared validators.

mod app_mode;
mod categories;
mod identifiers;
mod url;
mod validate;

pub use app_mode::AppMode;
pub use categories::{AppCategory, CategoryList};
pub use identifiers::{BrowserId, DesktopFileName, ProfileKind};
pub use url::{UrlValidationError, WebAppUrl};
