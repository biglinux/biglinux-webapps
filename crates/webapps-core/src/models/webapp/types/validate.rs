use anyhow::{bail, Result};
use std::path::{Component, Path};

pub(super) fn validate_optional_single_path_component(value: &str, field_name: &str) -> Result<()> {
    if value.is_empty() {
        return Ok(());
    }

    validate_single_path_component(value, field_name)
}

pub(super) fn validate_single_path_component(value: &str, field_name: &str) -> Result<()> {
    let path = Path::new(value);
    let mut components = path.components();

    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => bail!("{field_name} must be a single relative path component"),
    }
}
