use anyhow::{bail, Result};

use super::validate::{validate_optional_single_path_component, validate_single_path_component};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BrowserId(String);

impl BrowserId {
    /// Sentinel browser id used by webapps that open in the built-in WebKit viewer.
    pub const VIEWER: &'static str = "__viewer__";

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_viewer(&self) -> bool {
        self.0 == Self::VIEWER
    }

    pub fn validate(&self) -> Result<()> {
        validate_optional_single_path_component(&self.0, "browser id")
    }
}

impl From<&str> for BrowserId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DesktopFileName(String);

impl DesktopFileName {
    pub fn parse(value: &str) -> Option<Self> {
        (!value.is_empty()).then(|| Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate(&self) -> Result<()> {
        validate_single_path_component(&self.0, "desktop filename")?;
        if !self.0.ends_with(".desktop") {
            bail!("desktop filename must end with .desktop");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileKind {
    Default,
    Browser,
    Custom(String),
}

impl ProfileKind {
    pub fn parse(value: &str) -> Self {
        match value {
            "Default" => Self::Default,
            "Browser" => Self::Browser,
            _ => Self::Custom(value.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Default => "Default",
            Self::Browser => "Browser",
            Self::Custom(value) => value,
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Default | Self::Browser => Ok(()),
            Self::Custom(value) => validate_single_path_component(value, "profile name"),
        }
    }
}
