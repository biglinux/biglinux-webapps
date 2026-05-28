//! Typed list controller for the webapps manager.
//!
//! Owns the content `gtk::Box`, a `FactoryVecDeque<WebAppSectionFactory>` for
//! the categorised list, and the empty-state surface. Inputs reset the list
//! from a fresh `Vec<WebAppSection>`; outputs propagate row actions to the
//! window controller.
//!
//! This replaces the imperative `window::list::populate_list` cycle and the
//! `Rc<webapp_row::RowCallbacks>` closure jungle with a single Relm4
//! component.

use libadwaita as adw;
use relm4::adw::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::gtk;
use relm4::prelude::*;

use webapps_core::models::WebApp;

use super::empty;
use super::section::{WebAppSectionFactory, WebAppSectionInit, WebAppSectionOutput};

/// One category section: title plus pre-sorted apps.
#[derive(Debug, Clone)]
pub struct WebAppSection {
    /// Category title.
    pub title: String,
    /// Apps in this category, sorted by name.
    pub apps: Vec<WebApp>,
}

/// Inputs accepted by [`WebAppListController`].
#[derive(Debug, Clone)]
pub enum WebAppListInput {
    /// Replace all sections atomically.
    Refresh {
        /// Computed sections (already sorted).
        sections: Vec<WebAppSection>,
        /// Whether a search filter is currently active.
        has_filter: bool,
        /// Total visible result count (used for the "{N} results" status line).
        result_count: usize,
    },
}

/// Outputs emitted by [`WebAppListController`].
#[derive(Debug, Clone)]
pub enum WebAppListOutput {
    /// Row "edit" pressed.
    EditRequested(WebApp),
    /// Row "change browser" pressed.
    BrowserRequested(WebApp),
    /// Row "delete" pressed.
    DeleteRequested(WebApp),
    /// Empty-state CTA pressed.
    AddRequested,
}

/// List controller model.
pub struct WebAppListController {
    sections: FactoryVecDeque<WebAppSectionFactory>,
    empty_page: adw::StatusPage,
    status_label: gtk::Label,
}

#[relm4::component(pub)]
impl SimpleComponent for WebAppListController {
    type Init = ();
    type Input = WebAppListInput;
    type Output = WebAppListOutput;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 18,
            set_margin_start: 12,
            set_margin_end: 12,
            set_margin_top: 18,
            set_margin_bottom: 24,
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let sections = FactoryVecDeque::<WebAppSectionFactory>::builder()
            .launch(root.clone())
            .forward(sender.output_sender(), |out| match out {
                WebAppSectionOutput::BrowserRequested(app) => {
                    WebAppListOutput::BrowserRequested(app)
                }
                WebAppSectionOutput::EditRequested(app) => WebAppListOutput::EditRequested(app),
                WebAppSectionOutput::DeleteRequested(app) => WebAppListOutput::DeleteRequested(app),
            });

        // Empty state via BigEmptyStateSpec.
        let spec = empty::build_spec();
        let (empty_page, empty_button) = empty::build_page(&spec);
        if let Some(btn) = empty_button {
            let out = sender.output_sender().clone();
            btn.connect_clicked(move |_| {
                let _ = out.send(WebAppListOutput::AddRequested);
            });
        }
        empty_page.set_visible(false);
        root.append(&empty_page);

        // Status label ("{N} results") — shown only while filtering.
        let status_label = gtk::Label::new(None);
        status_label.set_visible(false);
        status_label.set_accessible_role(gtk::AccessibleRole::Status);
        status_label.add_css_class("dim-label");
        status_label.add_css_class("caption");
        status_label.set_margin_bottom(6);
        root.append(&status_label);

        let model = Self {
            sections,
            empty_page,
            status_label,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            WebAppListInput::Refresh {
                sections,
                has_filter,
                result_count,
            } => {
                let mut guard = self.sections.guard();
                guard.clear();
                for section in &sections {
                    guard.push_back(WebAppSectionInit {
                        title: section.title.clone(),
                        apps: section.apps.clone(),
                    });
                }
                drop(guard);

                let is_empty = sections.is_empty();
                self.empty_page.set_visible(is_empty);

                if has_filter {
                    let label =
                        gettextrs::gettext("{} results").replace("{}", &result_count.to_string());
                    self.status_label.set_label(&label);
                    self.status_label.set_visible(true);
                } else {
                    self.status_label.set_label("");
                    self.status_label.set_visible(false);
                }
            }
        }
    }
}
