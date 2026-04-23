use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppCategory {
    Webapps,
    Network,
    Office,
    Development,
    Graphics,
    AudioVideo,
    Game,
    Utility,
    System,
    Education,
    Science,
    Custom(String),
}

impl AppCategory {
    pub fn parse(value: &str) -> Self {
        match value {
            "Webapps" => Self::Webapps,
            "Network" => Self::Network,
            "Office" => Self::Office,
            "Development" => Self::Development,
            "Graphics" => Self::Graphics,
            "AudioVideo" => Self::AudioVideo,
            "Game" => Self::Game,
            "Utility" => Self::Utility,
            "System" => Self::System,
            "Education" => Self::Education,
            "Science" => Self::Science,
            _ => Self::Custom(value.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Webapps => "Webapps",
            Self::Network => "Network",
            Self::Office => "Office",
            Self::Development => "Development",
            Self::Graphics => "Graphics",
            Self::AudioVideo => "AudioVideo",
            Self::Game => "Game",
            Self::Utility => "Utility",
            Self::System => "System",
            Self::Education => "Education",
            Self::Science => "Science",
            Self::Custom(value) => value,
        }
    }

    pub fn validate(&self) -> Result<()> {
        let value = self.as_str();
        if value.is_empty() {
            bail!("category must not be empty");
        }
        if value.contains(';') {
            bail!("category must not contain ';'");
        }
        if value.chars().any(char::is_control) {
            bail!("category must not contain control characters");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CategoryList {
    categories: Vec<AppCategory>,
}

impl CategoryList {
    pub fn parse(serialized: &str) -> Self {
        Self {
            categories: serialized
                .split(';')
                .filter(|category| !category.is_empty())
                .map(AppCategory::parse)
                .collect(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.categories.iter().map(AppCategory::as_str)
    }

    pub fn main(&self) -> &str {
        self.categories
            .first()
            .map(AppCategory::as_str)
            .unwrap_or("Webapps")
    }

    pub fn with_main(&self, category: &str) -> Self {
        if category.is_empty() {
            return self.clone();
        }

        let main = AppCategory::parse(category);
        let mut categories = vec![main.clone()];
        for existing in &self.categories {
            if existing != &main {
                categories.push(existing.clone());
            }
        }

        Self { categories }
    }

    pub fn to_serialized(&self) -> String {
        self.categories
            .iter()
            .map(AppCategory::as_str)
            .collect::<Vec<_>>()
            .join(";")
    }

    pub fn to_desktop_string(&self) -> String {
        if self.categories.is_empty() {
            "Webapps;".to_string()
        } else {
            format!("{};", self.to_serialized())
        }
    }

    pub fn validate(&self) -> Result<()> {
        for category in &self.categories {
            category.validate()?;
        }
        Ok(())
    }
}
