//! Manager shell metadata used by `window::build`.

pub struct ShellSpec {
    pub title: String,
    pub default_width: i32,
    pub default_height: i32,
}

#[must_use]
pub fn build() -> ShellSpec {
    ShellSpec {
        title: gettextrs::gettext("WebApps Manager"),
        default_width: 820,
        default_height: 680,
    }
}
