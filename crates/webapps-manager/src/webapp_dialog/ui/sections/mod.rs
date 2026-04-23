mod appearance;
mod behavior;
mod website;

pub(crate) use appearance::{build_appearance_section, AppearanceSection};
pub(crate) use behavior::{build_behavior_section, update_browser_icon, BehaviorSection};
pub(crate) use website::{build_website_section, WebsiteSection};
