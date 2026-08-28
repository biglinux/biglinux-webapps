use libadwaita as adw;
#[derive(Debug, Clone)]
pub struct InfoRowSpec {
    title: String,
}

pub struct InfoRow {
    row: adw::ActionRow,
}

impl InfoRowSpec {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }
}

impl InfoRow {
    pub fn new(spec: InfoRowSpec) -> Self {
        Self {
            row: adw::ActionRow::builder().title(spec.title).build(),
        }
    }

    pub fn into_root(self) -> adw::ActionRow {
        self.row
    }
}
