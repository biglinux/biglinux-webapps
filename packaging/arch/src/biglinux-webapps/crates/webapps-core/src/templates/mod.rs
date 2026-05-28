//! Built-in webapp template registry grouped by category (communication,
//! google, media, office365, productivity) and exposed via a single registry.

mod communication;
mod google;
mod media;
mod office365;
mod productivity;
mod registry;

pub use registry::{
    build_default_registry, default_registry, FileHandler, TemplateRegistry, WebAppTemplate,
};
