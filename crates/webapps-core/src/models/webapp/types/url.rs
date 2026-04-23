use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlValidationError {
    Empty,
    Invalid,
}

impl fmt::Display for UrlValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "url must not be empty"),
            Self::Invalid => write!(f, "url is invalid"),
        }
    }
}

impl std::error::Error for UrlValidationError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WebAppUrl(String);

impl WebAppUrl {
    pub fn parse(raw: &str) -> Result<Self, UrlValidationError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(UrlValidationError::Empty);
        }

        let normalized = if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("file://")
        {
            trimmed.to_string()
        } else {
            format!("https://{trimmed}")
        };

        url::Url::parse(&normalized).map_err(|_| UrlValidationError::Invalid)?;
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}
