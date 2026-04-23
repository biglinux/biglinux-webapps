use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    #[default]
    Browser,
    App,
}

impl AppMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::App => "app",
        }
    }
}
