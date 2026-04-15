"""
MainWindow module containing the main application window
"""

import gi
import time
import uuid

from typing import Any

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, Gio, GObject

from webapps.ui.webapp_row import WebAppRow
from webapps.ui.webapp_dialog import WebAppDialog
from webapps.ui.browser_dialog import BrowserDialog
from webapps.ui.welcome_dialog import WelcomeDialog
from webapps.models.webapp_model import WebApp
from webapps.utils.translation import _

import logging

logger = logging.getLogger(__name__)


class MainWindow(Adw.ApplicationWindow):
    """Main application window for WebApps Manager"""

    def __init__(self, **kwargs: Any) -> None:
        """Initialize the main window"""
        super().__init__(
            title=_("WebApps Manager"), default_width=800, default_height=650, **kwargs
        )

        self.app = kwargs.get("application")
        self.filter_text = ""
        self.current_webapp = None
        self.setup_ui()

        # Create action to show welcome dialog
        show_welcome_action = Gio.SimpleAction.new("show-welcome", None)
        show_welcome_action.connect(
            "activate", lambda *_args: self.show_welcome_dialog()
        )
        self.app.add_action(show_welcome_action)

        # Show welcome dialog if it should be shown
        if WelcomeDialog.should_show_welcome():
            self.show_welcome_dialog()

    def show_welcome_dialog(self) -> None:
        """Show the welcome dialog"""
        welcome = WelcomeDialog(self)
        welcome.present()

    def setup_ui(self) -> None:
        """Set up the UI components."""
        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)

        # Header bar
        header = Adw.HeaderBar()

        # Search button on the left
        search_button = Gtk.ToggleButton(icon_name="system-search-symbolic")
        search_button.connect("toggled", self.on_search_toggled)
        search_button.set_tooltip_text(_("Search WebApps"))
        search_button.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [_("Search WebApps")],
        )
        header.pack_start(search_button)

        # Add menu first so it appears to the right of the Add button
        menu_button = Gtk.MenuButton()
        menu_button.set_icon_name("open-menu-symbolic")
        menu_button.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [_("Main Menu")],
        )

        # Create menu model with items
        menu = Gio.Menu()
        # Use translation function for menu items
        menu.append(_("Refresh"), "app.refresh")
        menu.append(_("Export WebApps"), "app.export")
        menu.append(_("Import WebApps"), "app.import")

        # Add new menu items for browsing folders
        menu.append(_("Browse Applications Folder"), "app.browse-apps")
        menu.append(_("Browse Profiles Folder"), "app.browse-profiles")

        # Add a separator before help-related items
        menu_section = Gio.Menu()
        menu_section.append(_("Show Welcome Screen"), "app.show-welcome")
        menu_section.append(_("Remove All WebApps"), "app.remove-all")
        menu_section.append(_("About"), "app.about")
        menu.append_section(None, menu_section)

        # Set the menu
        menu_button.set_menu_model(menu)
        header.pack_end(menu_button)

        # Add new webapp button on the right with text instead of icon
        add_button = Gtk.Button(label=_("Add"))
        add_button.connect("clicked", self.on_add_clicked)
        add_button.add_css_class("suggested-action")
        header.pack_end(add_button)

        main_box.append(header)

        # Search bar
        self.search_bar = Gtk.SearchBar()
        self.search_bar.set_key_capture_widget(self)

        search_entry = Gtk.SearchEntry()
        search_entry.set_hexpand(True)
        search_entry.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [_("Search WebApps")],
        )
        search_entry.connect("search-changed", self.on_search_changed)
        self.search_bar.set_child(search_entry)

        main_box.append(self.search_bar)

        # Create scrolled window for content
        scrolled = Gtk.ScrolledWindow()
        scrolled.set_hexpand(True)
        scrolled.set_vexpand(True)

        # Create box for categorized webapps
        self.content_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        self.content_box.set_margin_start(40)
        self.content_box.set_margin_end(40)
        self.content_box.set_margin_top(12)
        self.content_box.set_margin_bottom(12)

        # Add an empty state for when there are no webapps
        self.empty_state = Adw.StatusPage()
        self.empty_state.set_icon_name("big-webapps")
        self.empty_state.set_title(_("No WebApps Found"))
        self.empty_state.set_description(_("Add a new webapp to get started"))
        self.empty_state.set_focusable(True)

        # Add button in empty state
        empty_add_button = Gtk.Button(label=_("Add WebApp"))
        empty_add_button.add_css_class("suggested-action")
        empty_add_button.connect("clicked", self.on_add_clicked)
        self.empty_state.set_child(empty_add_button)

        # Start with empty state
        self.content_box.append(self.empty_state)

        scrolled.set_child(self.content_box)
        main_box.append(scrolled)

        # Add a toast overlay for notifications
        self.toast_overlay = Adw.ToastOverlay()
        self.toast_overlay.set_child(main_box)

        # Set the content
        self.set_content(self.toast_overlay)

        # Load webapps
        self.refresh_ui()

    def on_search_toggled(self, button: Gtk.ToggleButton) -> None:
        """Handle search button toggle"""
        self.search_bar.set_search_mode(button.get_active())

    def on_search_changed(self, entry: Gtk.SearchEntry) -> None:
        """Handle search text changes"""
        self.filter_text = entry.get_text()
        self.refresh_ui()

    def on_add_clicked(self, button: Gtk.Button) -> None:
        """Handle add button click"""
        # Create a new webapp with default values
        default_browser = self.app.browser_collection.get_default()
        browser_id = default_browser.browser_id if default_browser else ""

        new_webapp = WebApp({
            "browser": browser_id,
            "app_file": f"{int(time.time())}-{uuid.uuid4().hex[:8]}",
            "app_name": "",
            "app_url": "",
            "app_icon": "",
            "app_profile": "Browser",
            "app_categories": "Webapps",
            "app_icon_url": "/usr/share/icons/hicolor/scalable/apps/webapp-generic.svg",
        })

        dialog = WebAppDialog(
            self,
            new_webapp,
            self.app.browser_collection,
            self.app.command_executor,
            is_new=True,
        )
        dialog.connect("response", self.on_webapp_dialog_response)
        dialog.present()

    def on_webapp_clicked(self, row: WebAppRow, webapp: WebApp) -> None:
        """Handle webapp row click"""
        self.current_webapp = webapp
        dialog = WebAppDialog(
            self,
            webapp,
            self.app.browser_collection,
            self.app.command_executor,
            is_new=False,
        )
        dialog.connect("response", self.on_webapp_dialog_response)
        dialog.present()

    def _find_webapp_after_reload(
        self, url: str, name: str, app_file: str = ""
    ) -> WebApp | None:
        """Find webapp after data reload — prefers app_file (stable ID)."""
        return self.app.service.find_webapp(url, name, app_file=app_file)

    def on_webapp_dialog_response(self, dialog: WebAppDialog, response: int) -> None:
        """Handle webapp dialog response — save already executed by dialog."""
        if response == Gtk.ResponseType.OK:
            webapp = dialog.get_webapp()

            # reload collections (command already ran in dialog thread)
            self.app.service.load_data()

            found = self._find_webapp_after_reload(
                webapp.app_url, webapp.app_name, app_file=webapp.app_file
            )
            if found:
                self.current_webapp = found
            elif dialog.is_new:
                self.app.webapp_collection.add(webapp)

            msg = (
                _("WebApp created successfully")
                if dialog.is_new
                else _("WebApp updated successfully")
            )
            self.show_toast(msg)
            self.refresh_ui()

    def on_browser_selected(self, row: WebAppRow, webapp: WebApp) -> None:
        """Handle browser selection button click"""
        self.current_webapp = webapp

        dialog = BrowserDialog(
            self,
            webapp,
            self.app.browser_collection,
        )
        dialog.connect("response", self.on_browser_dialog_response)
        dialog.present()

    def on_browser_dialog_response(self, dialog: BrowserDialog, response: int) -> None:
        """Handle browser dialog response"""
        if response == Gtk.ResponseType.OK:
            browser = dialog.get_selected_browser()
            original_url = self.current_webapp.app_url
            original_name = self.current_webapp.app_name
            original_file = self.current_webapp.app_file
            self.current_webapp.browser = browser.browser_id

            success = self.app.service.update_webapp(self.current_webapp)
            if success:
                found = self._find_webapp_after_reload(
                    original_url, original_name, app_file=original_file
                )
                if found:
                    self.current_webapp = found

                self.show_toast(
                    _("Browser changed to {0}").format(browser.get_friendly_name())
                )
                self.refresh_ui()

    def on_delete_clicked(self, row: WebAppRow, webapp: WebApp) -> None:
        """Handle delete button click"""
        # Make sure we use the webapp from the row (most up-to-date)
        # rather than potentially stale current_webapp
        self.current_webapp = webapp
        logger.debug("Delete clicked on webapp file: %s", webapp.app_file)

        # Create a delete confirmation dialog
        dialog = Adw.MessageDialog(
            transient_for=self,
            body=_(
                "Are you sure you want to delete {0}?\n\nURL: {1}\nBrowser: {2}"
            ).format(webapp.app_name, webapp.app_url, webapp.browser),
        )

        # Add checkbox for deleting the profile folder too
        check_button = None

        # Only show delete folder option if profile is not Browser and no other webapp uses it
        show_delete_folder = webapp.app_profile != "Browser" and not any(
            w.app_profile == webapp.app_profile and w.browser == webapp.browser
            for w in self.app.webapp_collection.get_all()
            if w.app_file != webapp.app_file
        )

        if show_delete_folder:
            content_area = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
            content_area.set_margin_top(12)
            check_button = Gtk.CheckButton(label=_("Also delete configuration folder"))
            content_area.append(check_button)

            dialog.set_extra_child(content_area)

        # Add buttons
        dialog.add_response("cancel", _("Cancel"))
        dialog.add_response("delete", _("Delete"))
        dialog.set_response_appearance("delete", Adw.ResponseAppearance.DESTRUCTIVE)
        dialog.set_default_response("cancel")

        dialog.connect("response", self.on_delete_dialog_response, check_button)
        dialog.present()

    def on_delete_dialog_response(
        self,
        dialog: Adw.MessageDialog,
        response: str,
        check_button: Gtk.CheckButton | None,
    ) -> None:
        """Handle delete dialog response"""
        if response == "delete":
            delete_folder = check_button.get_active() if check_button else False
            success = self.app.service.delete_webapp(self.current_webapp, delete_folder)
            if success:
                self.show_toast(
                    _("WebApp deleted successfully"), Adw.ToastPriority.HIGH
                )
                self.refresh_ui()

    def on_remove_all_clicked(self) -> None:
        """Handle remove all webapps action with text confirmation"""
        confirm_phrase = _("REMOVE ALL")
        dialog = Adw.MessageDialog(
            transient_for=self,
            heading=_("Remove All WebApps"),
            body=_(
                "Are you sure you want to remove all your WebApps? "
                "This action cannot be undone.\n\n"
                'Type "{0}" to confirm.'
            ).format(confirm_phrase),
        )

        dialog.add_response("cancel", _("Cancel"))
        dialog.add_response("confirm", _("Remove All"))
        dialog.set_response_appearance("confirm", Adw.ResponseAppearance.DESTRUCTIVE)
        dialog.set_default_response("cancel")
        dialog.set_response_enabled("confirm", False)

        entry = Gtk.Entry()
        entry.set_placeholder_text(confirm_phrase)
        entry.connect(
            "changed",
            lambda e: dialog.set_response_enabled(
                "confirm", e.get_text().strip() == confirm_phrase
            ),
        )
        dialog.set_extra_child(entry)
        dialog.connect("response", self._on_remove_all_confirmed)
        dialog.present()

    def _on_remove_all_confirmed(
        self, dialog: Adw.MessageDialog, response: str
    ) -> None:
        """Execute remove-all after confirmed."""
        if response != "confirm":
            return

        success = self.app.service.delete_all_webapps()
        if success:
            self.show_toast(_("All WebApps have been removed"), Adw.ToastPriority.HIGH)
        else:
            self.show_toast(_("Failed to remove all WebApps"), Adw.ToastPriority.HIGH)
        self.refresh_ui()

    def show_toast(
        self, message: str, priority: Adw.ToastPriority = Adw.ToastPriority.NORMAL
    ) -> None:
        """Show a toast notification. Use HIGH priority for errors/destructive actions."""
        toast = Adw.Toast.new(message)
        toast.set_timeout(3)
        toast.set_priority(priority)
        self.toast_overlay.add_toast(toast)

    def refresh_ui(self) -> None:
        """Refresh the UI with current webapps"""
        # Clear the content box
        while self.content_box.get_first_child():
            self.content_box.remove(self.content_box.get_first_child())

        # Get categorized webapps
        categorized = self.app.webapp_collection.get_categorized(self.filter_text)

        if not categorized:
            # Show empty state if no webapps
            self.content_box.append(self.empty_state)
            self.empty_state.grab_focus()
            return

        # Sort categories alphabetically
        categories = sorted(categorized.keys())

        # Add webapps by category
        for category in categories:
            # heading for Orca "h" key navigation
            header = GObject.new(
                Gtk.Label,
                accessible_role=Gtk.AccessibleRole.HEADING,
                label=f"<b>{category}</b>",
                use_markup=True,
            )
            header.set_halign(Gtk.Align.START)
            header.set_margin_top(12)
            header.set_margin_bottom(6)
            header.set_focusable(True)
            self.content_box.append(header)

            # Create a list box for the category
            list_box = Gtk.ListBox()
            list_box.set_selection_mode(Gtk.SelectionMode.NONE)
            list_box.add_css_class("boxed-list")

            # Sort webapps alphabetically by name
            webapps = sorted(categorized[category], key=lambda app: app.app_name)

            # Add webapps to the list box
            for webapp in webapps:
                row = WebAppRow(webapp, self.app.browser_collection)
                row.connect("edit-clicked", self.on_webapp_clicked)
                row.connect("browser-clicked", self.on_browser_selected)
                row.connect("delete-clicked", self.on_delete_clicked)
                list_box.append(row)

            self.content_box.append(list_box)
