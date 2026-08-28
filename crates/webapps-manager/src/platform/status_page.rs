use gtk4 as gtk;
use libadwaita as adw;
use relm4::adw::prelude::*;

#[derive(Debug, Clone)]
pub struct EmptyStateSpec {
    icon_name: String,
    title: String,
    body: Option<String>,
    action_label: Option<String>,
    action_name: Option<String>,
}

impl EmptyStateSpec {
    pub fn new(icon_name: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            icon_name: icon_name.into(),
            title: title.into(),
            body: None,
            action_label: None,
            action_name: None,
        }
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_action(mut self, label: impl Into<String>, action_name: impl Into<String>) -> Self {
        self.action_label = Some(label.into());
        self.action_name = Some(action_name.into());
        self
    }
}

pub fn build_empty_state(spec: &EmptyStateSpec) -> (adw::StatusPage, Option<gtk::Button>) {
    let page = adw::StatusPage::builder()
        .icon_name(&spec.icon_name)
        .title(&spec.title)
        .build();
    if let Some(body) = &spec.body {
        page.set_description(Some(body));
    }

    let button = spec.action_label.as_ref().map(|label| {
        let button = gtk::Button::with_label(label);
        button.add_css_class("suggested-action");
        if let Some(action_name) = &spec.action_name {
            button.set_action_name(Some(action_name));
        }
        button
    });
    if let Some(button) = &button {
        page.set_child(Some(button));
    }
    (page, button)
}
