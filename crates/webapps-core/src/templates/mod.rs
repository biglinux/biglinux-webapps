mod registry;
mod office365;
mod google;
mod communication;
mod media;
mod productivity;

pub use registry::{WebAppTemplate, TemplateRegistry, FileHandler, build_default_registry};
