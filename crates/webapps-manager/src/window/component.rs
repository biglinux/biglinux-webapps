//! `ManagerWindow` — the webapps manager window as a Relm4 [`Component`].
//!
//! The window content (header, search, list, toast overlay) is mounted as a
//! Relm4 component. The launcher [`super::build`] creates the
//! `adw::ApplicationWindow`, mounts `controller.widget()`, and owns geometry
//! plus presentation.
//!
//! Model owns the window-scoped [`WindowContext`]; the header's Add button and
//! the search entry emit typed [`ManagerInput`] messages handled in
//! [`Component::update`]. Row actions (edit/browser/delete/add) arrive as
//! `WebAppListController` outputs and are routed to handler functions.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::platform::tooltip;
use adw::prelude::*;
use gettextrs::gettext;
use gtk4 as gtk;
use libadwaita as adw;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender};

use webapps_core::models::{BrowserCollection, WebAppCollection};

use crate::relm4_window::list::{WebAppListController, WebAppListOutput};
use crate::{service, ui_async};

use super::context::WindowContext;
use super::{actions, list, shortcuts, state, ui};

/// Domain handle the launcher hands the component: the host application (for
/// shortcut scope) and the window it mounts into (for dialogs, actions).
pub(super) struct ManagerInit {
    pub app: adw::Application,
    pub window: adw::ApplicationWindow,
}

/// Header-driven user actions the window owns.
#[derive(Debug)]
pub(super) enum ManagerInput {
    /// The "Add" button was clicked.
    AddRequested,
    /// The "Templates" button was clicked.
    TemplatesRequested,
    /// The search entry text changed.
    SearchChanged(String),
}

/// The manager window content as a Relm4 component (model owns `WindowContext`).
pub(super) struct ManagerWindow {
    context: WindowContext,
}

impl Component for ManagerWindow {
    type Init = ManagerInit;
    type Input = ManagerInput;
    type Output = ();
    type CommandOutput = ();
    /// Mountable content (ADR-D14): the toast overlay, never the window.
    type Root = adw::ToastOverlay;
    type Widgets = ();

    fn init_root() -> Self::Root {
        adw::ToastOverlay::new()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ManagerInit { app, window } = init;

        // migrate + load run on a worker thread so a slow home (NFS) can't delay
        // first paint; the window opens empty and data is swapped in below.
        let state = state::new_empty_state();
        let browsers = Rc::new(RefCell::new(BrowserCollection::default()));

        // Relm4 list controller mounted inside the scrolled clamp.
        let list_connector = WebAppListController::builder().launch(());
        let list_widget: gtk::Widget = list_connector.widget().clone().upcast();
        let controls = ui::build_content(&root, &list_widget);

        // Block "Add"/"Templates" until browser detection completes — creating a
        // webapp with browser="" would produce a broken .desktop file.
        controls.add_btn.set_sensitive(false);
        controls.templates_btn.set_sensitive(false);
        tooltip::set(&controls.add_btn, &gettext("Detecting installed browsers…"));
        {
            let browsers = browsers.clone();
            let add_btn = controls.add_btn.clone();
            let templates_btn = controls.templates_btn.clone();
            ui_async::run_with_result(service::detect_browsers, move |detected| {
                let has_any = !detected.browsers.is_empty();
                *browsers.borrow_mut() = detected;
                add_btn.set_sensitive(has_any);
                templates_btn.set_sensitive(has_any);
                tooltip::update(
                    &add_btn,
                    &gettext(if has_any {
                        "Add WebApp"
                    } else {
                        "No supported browser is installed"
                    }),
                );
            });
        }

        // Deferred context so `connect_receiver` can fire row outputs back into
        // the existing handlers (the context needs the list controller, which
        // needs this receiver — break the cycle with a slot).
        let context_slot: Rc<RefCell<Option<WindowContext>>> = Rc::new(RefCell::new(None));
        let list_controller = {
            let slot = context_slot.clone();
            list_connector.connect_receiver(move |_, out| {
                let Some(ctx) = slot.borrow().as_ref().cloned() else {
                    return;
                };
                match out {
                    WebAppListOutput::EditRequested(app) => list::handle_edit(ctx, &app),
                    WebAppListOutput::BrowserRequested(app) => {
                        list::handle_browser_change(ctx, &app)
                    }
                    WebAppListOutput::DeleteRequested(app) => list::handle_delete(ctx, &app),
                    WebAppListOutput::AddRequested => list::open_add_dialog(&ctx),
                }
            })
        };

        let context = WindowContext {
            state,
            browsers,
            window: Rc::new(window.clone()),
            toast: Rc::new(root.clone()),
            list: Rc::new(list_controller),
            reload_generation: Rc::new(Cell::new(0)),
        };
        *context_slot.borrow_mut() = Some(context.clone());

        list::populate_list(&context);
        actions::install_window_actions(&context);

        // Kick off migration + initial load on a worker thread.
        {
            let context_for_load = context.clone();
            // Snapshot the reload generation: a user create/edit before this slow
            // startup load finishes leaves a newer list — don't clobber it.
            let load_generation = context.reload_generation.get();
            ui_async::run_with_result(
                || {
                    let migrated = service::migrate_legacy_desktops();
                    let regenerated_app = service::regenerate_app_mode_desktops();
                    let regenerated_browser = service::regenerate_browser_mode_desktops();
                    let renamed_browser = service::migrate_browser_desktop_filenames();
                    let persisted_icons = service::persist_existing_icons();
                    let webapps = service::load_webapps();
                    (
                        migrated,
                        regenerated_app,
                        regenerated_browser,
                        renamed_browser,
                        persisted_icons,
                        webapps,
                    )
                },
                move |(
                    migrated,
                    regenerated_app,
                    regenerated_browser,
                    renamed_browser,
                    persisted_icons,
                    webapps,
                ): (usize, usize, usize, usize, usize, WebAppCollection)| {
                    if migrated > 0 {
                        log::info!("Migrated {migrated} legacy webapps from .desktop files");
                    }
                    if regenerated_app > 0 {
                        log::info!("Regenerated {regenerated_app} viewer-mode .desktop entries (StartupWMClass alignment)");
                    }
                    if regenerated_browser > 0 {
                        log::info!("Regenerated {regenerated_browser} browser-mode .desktop entries (StartupWMClass + --class alignment)");
                    }
                    if renamed_browser > 0 {
                        log::info!("Renamed {renamed_browser} browser-mode .desktop entries to the Chromium app_id scheme");
                    }
                    if persisted_icons > 0 {
                        log::info!("Persisted {persisted_icons} webapp icons into the data directory (Icon= now uses a stable absolute path)");
                    }
                    if context_for_load.reload_generation.get() != load_generation {
                        return;
                    }
                    state::apply_webapps(&context_for_load.state, webapps);
                    list::populate_list(&context_for_load);
                },
            );
        }

        // Header interactions → typed messages.
        {
            let sender = sender.clone();
            controls.search_entry.connect_search_changed(move |entry| {
                sender.input(ManagerInput::SearchChanged(entry.text().to_string()));
            });
        }
        {
            let sender = sender.clone();
            controls
                .add_btn
                .connect_clicked(move |_| sender.input(ManagerInput::AddRequested));
        }
        {
            let sender = sender.clone();
            controls
                .templates_btn
                .connect_clicked(move |_| sender.input(ManagerInput::TemplatesRequested));
        }

        shortcuts::install_shortcuts(&app, &window, &controls.add_btn, &controls.search_btn);

        ComponentParts {
            model: ManagerWindow { context },
            widgets: (),
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            ManagerInput::AddRequested => list::open_add_dialog(&self.context),
            ManagerInput::TemplatesRequested => list::open_template_gallery(&self.context),
            ManagerInput::SearchChanged(text) => {
                state::set_filter_text(&self.context.state, text);
                list::populate_list(&self.context);
            }
        }
    }
}
