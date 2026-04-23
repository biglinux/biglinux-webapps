mod browser;
mod webapp;

pub use browser::{Browser, BrowserCollection, BrowserKind};
pub use webapp::{
    AppCategory, AppMode, BrowserId, CategoryList, DesktopFileName, ProfileKind,
    UrlValidationError, WebApp, WebAppCollection, WebAppUrl,
};
