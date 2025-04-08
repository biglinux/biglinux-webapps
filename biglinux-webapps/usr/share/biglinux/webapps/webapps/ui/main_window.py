"""
MainWindow module containing the main application window
"""

import gi
import time
from datetime import datetime

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, Gio

from webapps.ui.webapp_row import WebAppRow
from webapps.ui.webapp_dialog import WebAppDialog
from webapps.ui.browser_dialog import BrowserDialog
from webapps.ui.welcome_dialog import WelcomeDialog
from webapps.models.webapp_model import WebApp
from webapps.utils.translation import _


class MainWindow(Adw.ApplicationWindow):
    """Main application window for WebApps Manager"""

    def __init__(self, **kwargs):
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
            "activate", lambda *args: self.show_welcome_dialog()
        )
        self.app.add_action(show_welcome_action)

        # Show welcome dialog if it should be shown
        if WelcomeDialog.should_show_welcome():
            self.show_welcome_dialog()

    def show_welcome_dialog(self):
        """Show the welcome dialog"""
        welcome = WelcomeDialog(self)
        welcome.present()

    def setup_ui(self):
        """Set up the UI components"""
        # Main box
        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)

        # Header bar
        header = Adw.HeaderBar()

        # App icon on the left
        app_icon = Gtk.Image.new_from_icon_name("big-webapps")
        app_icon.set_pixel_size(24)
        header.pack_start(app_icon)

        # Search button on the left
        search_button = Gtk.ToggleButton(icon_name="system-search-symbolic")
        search_button.connect("toggled", self.on_search_toggled)
        search_button.set_tooltip_text(_("Search WebApps"))
        header.pack_start(search_button)

        # Add menu first so it appears to the right of the Add button
        menu_button = Gtk.MenuButton()
        menu_button.set_icon_name("open-menu-symbolic")

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

    def on_search_toggled(self, button):
        """Handle search button toggle"""
        self.search_bar.set_search_mode(button.get_active())

    def on_search_changed(self, entry):
        """Handle search text changes"""
        self.filter_text = entry.get_text()
        self.refresh_ui()

    def on_add_clicked(self, button):
        """Handle add button click"""
        # Create a new webapp with default values
        default_browser = self.app.browser_collection.get_default()
        browser_id = default_browser.browser_id if default_browser else ""

        new_webapp = WebApp({
            "browser": browser_id,
            "app_file": f"{int(time.time())}-{hash(datetime.now()) % 10000}",
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

    def on_webapp_clicked(self, row, webapp):
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

    def on_webapp_dialog_response(self, dialog, response):
        """Handle webapp dialog response"""
        if response == Gtk.ResponseType.OK:
            # Get the updated webapp from the dialog
            webapp = dialog.get_webapp()

            # Make a copy of the original URL and name for reference
            original_url = webapp.app_url
            original_name = webapp.app_name

            if dialog.is_new:
                # Create a new webapp
                success = self.app.command_executor.create_webapp(webapp)
                if success:
                    # Force a reload of all data to get the latest state
                    self.app.load_data()

                    # Find the newly created webapp
                    found = False
                    for new_webapp in self.app.webapp_collection.get_all():
                        if (
                            new_webapp.app_url == original_url
                            and new_webapp.app_name == original_name
                        ):
                            # Update our local reference with the one from the collection
                            webapp = new_webapp
                            self.current_webapp = new_webapp
                            found = True
                            print(f"Updated new webapp reference: {webapp.app_file}")
                            break

                    # Add it to the collection if not already found
                    if not found:
                        self.app.webapp_collection.add(webapp)

                    self.show_toast(_("WebApp created successfully"))
            else:
                # Update an existing webapp
                # Store the original file name before update
                original_file = webapp.app_file

                success = self.app.command_executor.update_webapp(webapp)
                if success:
                    # Force a full data reload to get the fresh data
                    self.app.load_data()

                    # Look for the updated webapp with the potentially new filename
                    found = False
                    for updated_webapp in self.app.webapp_collection.get_all():
                        if (
                            updated_webapp.app_url == original_url
                            and updated_webapp.app_name == original_name
                        ):
                            # Use the updated webapp
                            webapp = updated_webapp
                            # Update the current webapp reference too
                            if self.current_webapp:
                                self.current_webapp = updated_webapp
                                print(
                                    f"Updated current_webapp reference to: {self.current_webapp.app_file}"
                                )
                            found = True
                            break

                    if not found and self.current_webapp:
                        # If we couldn't find the updated webapp, do a manual search
                        # based on the original file name pattern
                        print(
                            f"Webapp not found by URL/name. Looking for file pattern similar to {original_file}"
                        )

                        for updated_webapp in self.app.webapp_collection.get_all():
                            # See if the new file follows a similar pattern but with different browser
                            if updated_webapp.app_url == original_url:
                                self.current_webapp = updated_webapp
                                print(f"Found by URL: {self.current_webapp.app_file}")
                                break

                    self.show_toast(_("WebApp updated successfully"))

            # Refresh the UI
            self.refresh_ui()

    def on_browser_selected(self, row, webapp):
        """Handle browser selection button click"""
        self.current_webapp = webapp

        # Make sure command_executor is accessible
        self.command_executor = self.app.command_executor

        dialog = BrowserDialog(
            self,
            webapp,
            self.app.browser_collection,
        )
        dialog.connect("response", self.on_browser_dialog_response)
        dialog.present()

    def on_browser_dialog_response(self, dialog, response):
        """Handle browser dialog response"""
        if response == Gtk.ResponseType.OK:
            # Get the selected browser from the dialog
            browser = dialog.get_selected_browser()

            # Make a copy of the original webapp's properties before modifications
            original_url = self.current_webapp.app_url
            original_name = self.current_webapp.app_name
            original_file = self.current_webapp.app_file

            # Update the webapp browser
            self.current_webapp.browser = browser.browser_id

            # Update the webapp
            success = self.app.command_executor.update_webapp(self.current_webapp)
            if success:
                # Force a reload of all data to get the latest webapp information
                self.app.load_data()

                # After reload, find the updated webapp by URL and name
                found_webapp = None
                for webapp in self.app.webapp_collection.get_all():
                    if (
                        webapp.app_url == original_url
                        and webapp.app_name == original_name
                    ):
                        found_webapp = webapp
                        break

                # If found, update our reference
                if found_webapp:
                    self.current_webapp = found_webapp

                    # Debug information
                    print(
                        f"Original file: {original_file}, New file: {self.current_webapp.app_file}"
                    )

                self.show_toast(
                    _("Browser changed to {0}").format(browser.get_friendly_name())
                )

                # Refresh the UI
                self.refresh_ui()

    def on_delete_clicked(self, row, webapp):
        """Handle delete button click"""
        # Make sure we use the webapp from the row (most up-to-date)
        # rather than potentially stale current_webapp
        self.current_webapp = webapp
        print(f"Delete clicked on webapp file: {webapp.app_file}")

        # Create a delete confirmation dialog
        dialog = Adw.MessageDialog(
            transient_for=self,
            body=_("Are you sure you want to delete {0}?").format(webapp.app_name),
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

    def on_delete_dialog_response(self, dialog, response, check_button):
        """Handle delete dialog response"""
        if response == "delete":
            # Get delete folder option if available
            delete_folder = check_button.get_active() if check_button else False

            # Delete the webapp
            success = self.app.command_executor.remove_webapp(
                self.current_webapp, delete_folder
            )
            if success:
                self.app.webapp_collection.remove(self.current_webapp)
                self.show_toast(_("WebApp deleted successfully"))

                # Refresh the UI
                self.refresh_ui()

    def on_remove_all_clicked(self):
        """Handle remove all webapps action with double confirmation"""
        # First confirmation dialog
        first_dialog = Adw.MessageDialog(
            transient_for=self,
            heading=_("Remove All WebApps"),
            body=_(
                "Are you sure you want to remove all your WebApps? This action cannot be undone."
            ),
        )

        # Add buttons
        first_dialog.add_response("cancel", _("Cancel"))
        first_dialog.add_response("continue", _("Continue"))
        first_dialog.set_response_appearance(
            "continue", Adw.ResponseAppearance.DESTRUCTIVE
        )
        first_dialog.set_default_response("cancel")

        first_dialog.connect("response", self.on_first_remove_all_response)
        first_dialog.present()

    def on_first_remove_all_response(self, dialog, response):
        """Handle first confirmation dialog response"""
        if response == "continue":
            # Second confirmation dialog
            second_dialog = Adw.MessageDialog(
                transient_for=self,
                heading=_("Final Confirmation"),
                body=_("Are you ABSOLUTELY sure you want to remove ALL your WebApps?"),
            )

            # Add buttons
            second_dialog.add_response("cancel", _("No, Cancel"))
            second_dialog.add_response("confirm", _("Yes, Remove All"))
            second_dialog.set_response_appearance(
                "confirm", Adw.ResponseAppearance.DESTRUCTIVE
            )
            second_dialog.set_default_response("cancel")

            second_dialog.connect("response", self.on_second_remove_all_response)
            second_dialog.present()

    def on_second_remove_all_response(self, dialog, response):
        """Handle second confirmation dialog response"""
        if response == "confirm":
            # Get all webapps
            webapps = self.app.webapp_collection.get_all()

            # Remove each webapp
            success = True
            for webapp in webapps:
                # Delete the webapp without deleting profile folders
                if not self.app.command_executor.remove_webapp(webapp, False):
                    success = False

            if success:
                # Instead of calling clear(), reload the data to get a fresh state
                self.app.load_data()
                self.show_toast(_("All WebApps have been removed"))
            else:
                self.show_toast(_("Failed to remove all WebApps"))

            # Refresh the UI
            self.refresh_ui()

    def show_toast(self, message):
        """Show a toast notification"""
        toast = Adw.Toast.new(message)
        toast.set_timeout(3)
        self.toast_overlay.add_toast(toast)

    def refresh_ui(self):
        """Refresh the UI with current webapps"""
        # Clear the content box
        while self.content_box.get_first_child():
            self.content_box.remove(self.content_box.get_first_child())

        # Get categorized webapps
        categorized = self.app.webapp_collection.get_categorized(self.filter_text)

        if not categorized:
            # Show empty state if no webapps
            self.content_box.append(self.empty_state)
            return

        # Sort categories alphabetically
        categories = sorted(categorized.keys())

        # Add webapps by category
        for category in categories:
            # Create a category header
            header = Gtk.Label()
            header.set_markup(f"<b>{category}</b>")
            header.set_halign(Gtk.Align.START)
            header.set_margin_top(12)
            header.set_margin_bottom(6)
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
