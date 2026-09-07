//! Per-category section factory: an [`adw::PreferencesGroup`] hosting a typed
//! [`FactoryVecDeque`] of [`WebAppRowFactory`] rows.
//!
//! The list controller owns one of these per visible category so rebuilds
//! happen via typed messages instead of manual `gtk::Box` clearing.

use libadwaita as adw;
use relm4::adw::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::gtk;

use webapps_core::models::WebApp;

use super::row::{WebAppRowFactory, WebAppRowInit, WebAppRowOutput};

/// Init payload for a section: the category title and its apps.
#[derive(Debug, Clone)]
pub struct WebAppSectionInit {
    /// Category title shown as the [`adw::PreferencesGroup`] heading.
    pub title: String,
    /// Apps that belong to this category, already sorted by name.
    pub apps: Vec<WebApp>,
}

/// Section factory model.
#[derive(Debug)]
pub struct WebAppSectionFactory {
    title: String,
    // caveman: pending apps installed into `rows` once init_widgets sees the root group.
    pending_apps: Vec<WebApp>,
    displayed_apps: Vec<WebApp>,
    rows: Option<FactoryVecDeque<WebAppRowFactory>>,
}

/// Inputs accepted by a section (unused; sections are immutable after build).
#[derive(Debug)]
pub enum WebAppSectionInput {
    Refresh(Vec<WebApp>),
}

/// Outputs bubbled up from any row in this section.
#[derive(Debug, Clone)]
pub enum WebAppSectionOutput {
    /// Row action: change browser.
    BrowserRequested(WebApp),
    /// Row action: edit webapp.
    EditRequested(WebApp),
    /// Row action: delete webapp.
    DeleteRequested(WebApp),
}

impl FactoryComponent for WebAppSectionFactory {
    type Init = WebAppSectionInit;
    type Input = WebAppSectionInput;
    type Output = WebAppSectionOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type Root = adw::PreferencesGroup;
    type Widgets = ();
    type Index = DynamicIndex;

    fn init_root(&self) -> Self::Root {
        let group = adw::PreferencesGroup::new();
        group.set_title(&self.title);
        group
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            title: init.title,
            displayed_apps: init.apps.clone(),
            pending_apps: init.apps,
            rows: None,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let rows = FactoryVecDeque::<WebAppRowFactory>::builder()
            .launch(root)
            .forward(sender.output_sender(), |out| match out {
                WebAppRowOutput::BrowserRequested(app) => {
                    WebAppSectionOutput::BrowserRequested(app)
                }
                WebAppRowOutput::EditRequested(app) => WebAppSectionOutput::EditRequested(app),
                WebAppRowOutput::DeleteRequested(app) => WebAppSectionOutput::DeleteRequested(app),
            });

        self.rows = Some(rows);
        let rows = self.rows.as_mut().expect("rows just initialized");
        let mut guard = rows.guard();
        for app in std::mem::take(&mut self.pending_apps) {
            guard.push_back(WebAppRowInit { webapp: app });
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        let WebAppSectionInput::Refresh(apps) = msg;
        let Some(rows) = self.rows.as_mut() else {
            return;
        };
        let mut guard = rows.guard();
        for (index, app) in apps.iter().enumerate() {
            if self.displayed_apps.get(index) == Some(app) {
                continue;
            }
            if index < guard.len() {
                guard.remove(index);
            }
            guard.insert(
                index,
                WebAppRowInit {
                    webapp: app.clone(),
                },
            );
        }
        while guard.len() > apps.len() {
            guard.pop_back();
        }
        self.displayed_apps = apps;
    }
}
