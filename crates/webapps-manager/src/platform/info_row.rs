use libadwaita as adw;
use relm4::adw::prelude::*;

#[derive(Debug, Clone)]
pub struct InfoRowSpec {
    title: String,
    subtitle: Option<String>,
    allow_markup: bool,
}

pub struct InfoRow {
    row: adw::ActionRow,
}

impl InfoRowSpec {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            allow_markup: false,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn allow_markup(mut self) -> Self {
        self.allow_markup = true;
        self
    }
}

impl InfoRow {
    pub fn new(spec: InfoRowSpec) -> Self {
        let row = adw::ActionRow::builder().title(spec.title).build();
        if let Some(subtitle) = spec.subtitle {
            row.set_subtitle(&subtitle);
        }
        row.set_use_markup(spec.allow_markup);
        Self { row }
    }

    pub fn into_root(self) -> adw::ActionRow {
        self.row
    }
}
