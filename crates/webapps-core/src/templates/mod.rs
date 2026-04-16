mod communication;
mod google;
mod media;
mod office365;
mod productivity;
mod registry;

pub use registry::{build_default_registry, FileHandler, TemplateRegistry, WebAppTemplate};
