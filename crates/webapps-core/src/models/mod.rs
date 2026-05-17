//! Domain models for WebApps: `Browser` catalog plus the `WebApp` entity and
//! its value types.

mod browser;
mod webapp;

pub use browser::{Browser, BrowserCollection, BrowserKind};
pub use webapp::{
    AppCategory, AppMode, BrowserId, CategoryList, DesktopFileName, ProfileKind,
    UrlValidationError, WebApp, WebAppCollection, WebAppUrl,
};
