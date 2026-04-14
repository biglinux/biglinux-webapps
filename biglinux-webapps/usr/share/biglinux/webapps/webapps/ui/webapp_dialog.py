"""
WebAppDialog module containing the dialog for creating and editing webapps
"""

import gi
import threading
import time
import uuid

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, GObject, GdkPixbuf, Gdk, GLib

# Import BrowserDialog
from webapps.ui.browser_dialog import BrowserDialog
from webapps.ui.favicon_picker import FaviconPicker

# Import our new WebsiteInfoFetcher
from webapps.utils.url_utils import WebsiteInfoFetcher

# Import the browser icon utilities
from webapps.utils.browser_icon_utils import set_image_from_browser_icon

# Import the centralized translation function
from webapps.utils.translation import _
from webapps.models.webapp_model import WebApp
from webapps.models.browser_model import BrowserCollection
from webapps.utils.command_executor import CommandExecutor

import logging

logger = logging.getLogger(__name__)


class WebAppDialog(Adw.Window):
    """Dialog for creating and editing webapps"""

    # Define custom signals
    __gsignals__ = {"response": (GObject.SignalFlags.RUN_FIRST, None, (int,))}

    def __init__(
        self,
        parent: Gtk.Window,
        webapp: WebApp,
        browser_collection: BrowserCollection,
        command_executor: CommandExecutor,
        is_new: bool = False,
    ) -> None:
        """
        Initialize the WebAppDialog

        Parameters:
            parent (Gtk.Window): Parent window
            webapp (WebApp): WebApp object to edit or create
            browser_collection (BrowserCollection): BrowserCollection object
            command_executor (CommandExecutor): CommandExecutor object
            is_new (bool): Whether this is a new webapp or an existing one
        """
        super().__init__(
            title=_("Add WebApp") if is_new else _("Edit WebApp"),
            transient_for=parent,
            modal=True,
            destroy_with_parent=True,
            default_width=700,  # Increased width
            default_height=650,  # Increased height
        )

        # Use a safer approach to sizing the dialog - set fixed size directly
        # without trying to determine parent size which appears to be causing errors
        self.set_default_size(700, 650)

        self.webapp = webapp
        self.browser_collection = browser_collection
        self.command_executor = command_executor
        self.is_new = is_new

        # Clone the webapp to avoid modifying the original
        self.webapp = self._clone_webapp(webapp)

        # Detect system default browser + assign for new webapps
        self.system_default_browser_id = None
        if self.command_executor:
            self.system_default_browser_id = (
                self.command_executor.get_system_default_browser()
            )
            logger.debug(
                "System default browser detected: %s", self.system_default_browser_id
            )

        if self.is_new:
            self._assign_default_browser()

        # Create UI
        self.setup_ui()

    def _assign_default_browser(self) -> None:
        """Assign browser for a new webapp: system default → app default → noop."""
        if self.system_default_browser_id:
            system_browser = self.browser_collection.get_by_id(
                self.system_default_browser_id
            )
            if system_browser:
                self.webapp.browser = self.system_default_browser_id
                return
        # fallback to app default if no browser set
        if not self.webapp.browser:
            default_browser = self.browser_collection.get_default()
            if default_browser:
                self.webapp.browser = default_browser.browser_id

    def _clone_webapp(self, webapp: WebApp) -> WebApp:
        """
        Create a copy of a webapp

        Parameters:
            webapp (WebApp): WebApp object to clone

        Returns:
            WebApp: A new WebApp object with the same properties
        """
        from webapps.models.webapp_model import WebApp

        # Convert to dict and create a new instance
        webapp_dict = {
            "browser": webapp.browser,
            "app_file": webapp.app_file,
            "app_name": webapp.app_name,
            "app_url": webapp.app_url,
            "app_icon": webapp.app_icon,
            "app_profile": webapp.app_profile,
            "app_categories": webapp.app_categories,
            "app_icon_url": webapp.app_icon_url,
            "app_mode": webapp.app_mode,
        }

        return WebApp(webapp_dict)

    def setup_ui(self) -> None:
        """Set up the UI components."""
        content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)

        # Add key event controller to handle ESC key to close dialog
        key_controller = Gtk.EventControllerKey.new()
        key_controller.connect("key-pressed", self.on_key_pressed)
        self.add_controller(key_controller)

        # Define the window title based on whether we're adding or editing
        title = self.is_new and _("Add WebApp") or _("Edit WebApp")

        # Header with title - reduce padding in the header bar
        header = Adw.HeaderBar()
        header.set_title_widget(Gtk.Label(label=title))
        header.add_css_class("flat")
        header.set_show_end_title_buttons(True)

        # Apply custom CSS to reduce header padding
        css_provider = Gtk.CssProvider()
        css_provider.load_from_data(b"""
            headerbar {
                min-height: 38px;
                padding: 2px 6px;
            }
        """)
        style_context = header.get_style_context()
        Gtk.StyleContext.add_provider(
            style_context, css_provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        content.append(header)

        # Scrollable form area
        central_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        central_box.set_vexpand(True)
        central_box.set_valign(Gtk.Align.CENTER)

        scrolled = Gtk.ScrolledWindow()
        scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scrolled.set_propagate_natural_height(True)
        scrolled.set_min_content_height(300)

        clamp = Adw.Clamp()
        clamp.set_maximum_size(600)
        clamp.set_tightening_threshold(400)

        form_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        form_box.set_margin_top(0)
        form_box.set_margin_bottom(12)
        form_box.set_margin_start(24)
        form_box.set_margin_end(24)

        form_box.append(self._build_form_group())
        form_box.append(self._build_buttons())

        clamp.set_child(form_box)
        scrolled.set_child(clamp)
        central_box.append(scrolled)
        content.append(central_box)

        self.set_content(self._build_loading_overlay(content))

        # focus: URL for new webapps (need to type URL first);
        # name for editing (URL already filled, name is more actionable)
        target = self.url_row if self.is_new else self.name_row
        self.connect("map", lambda *_: target.grab_focus())

    def _build_form_group(self) -> Adw.PreferencesGroup:
        """Build all form rows: URL, Name, Icon, Category, Mode, Browser, Profile."""
        group = Adw.PreferencesGroup()

        # URL entry with detect button + real-time validation
        self.url_row = Adw.EntryRow()
        self.url_row.set_title(_("URL"))
        self.url_row.set_text(self.webapp.app_url)

        self.url_valid_icon = Gtk.Image()
        self.url_valid_icon.set_pixel_size(16)
        self.url_valid_icon.set_visible(False)
        self.url_row.add_suffix(self.url_valid_icon)

        detect_button = Gtk.Button(label=_("Detect"))
        detect_button.set_tooltip_text(_("Detect name and icon from website"))
        detect_button.set_valign(Gtk.Align.CENTER)
        detect_button.connect("clicked", self.on_detect_clicked)
        self.url_row.add_suffix(detect_button)
        self.url_row.connect("changed", self.on_url_changed)
        group.add(self.url_row)

        # Name entry
        self.name_row = Adw.EntryRow()
        self.name_row.set_title(_("Name"))
        self.name_row.set_text(self.webapp.app_name)
        self.name_row.connect("changed", self.on_name_changed)
        group.add(self.name_row)

        # Icon selection
        icon_row = Adw.ActionRow(title=_("App Icon"))
        self.icon_image = Gtk.Image()
        self.icon_image.set_pixel_size(48)
        self.set_icon_from_path(self.webapp.app_icon_url)
        icon_row.add_prefix(self.icon_image)

        select_icon_button = Gtk.Button(label=_("Select"))
        select_icon_button.set_tooltip_text(_("Select icon for the WebApp"))
        select_icon_button.set_valign(Gtk.Align.CENTER)
        select_icon_button.connect("clicked", self.on_select_icon_clicked)
        icon_row.add_suffix(select_icon_button)
        group.add(icon_row)

        # Row for favicon picker (shown after website detection)
        self.icon_selection_row = Adw.ActionRow(title=_("Available Icons"))
        self.icon_selection_row.set_visible(False)
        group.add(self.icon_selection_row)

        # Category
        self._build_category_row(group)

        # App mode + Browser + Profile
        self._build_mode_browser_profile(group)

        return group

    def _build_category_row(self, group: Adw.PreferencesGroup) -> None:
        """Add category dropdown to *group*."""
        main_category = self.webapp.get_main_category()
        self.category_dropdown = Gtk.DropDown()
        category_model = Gtk.StringList()

        self.system_categories = [
            "Webapps",
            "Network",
            "Office",
            "Development",
            "Graphics",
            "AudioVideo",
            "Game",
            "Utility",
            "System",
        ]
        for category in self.system_categories:
            category_model.append(_(category))

        self.category_dropdown.set_model(category_model)
        self.category_dropdown.set_valign(Gtk.Align.CENTER)
        self.category_dropdown.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [_("Category")],
        )

        for i, category in enumerate(self.system_categories):
            if category == main_category:
                self.category_dropdown.set_selected(i)
                break

        self.category_dropdown.connect("notify::selected", self.on_category_changed)
        category_row = Adw.ActionRow(title=_("Category"))
        category_row.add_suffix(self.category_dropdown)
        group.add(category_row)

    def _build_mode_browser_profile(self, group: Adw.PreferencesGroup) -> None:
        """Add app-mode switch, browser selector & profile rows to *group*."""
        # App mode toggle
        self.app_mode_row = Adw.ActionRow(title=_("Application Mode"))
        self.app_mode_row.set_subtitle(
            _("Opens as a native window without browser interface")
        )
        self.app_mode_switch = Gtk.Switch()
        self.app_mode_switch.set_valign(Gtk.Align.CENTER)
        self.app_mode_switch.set_active(self.webapp.app_mode == "app")
        self.app_mode_switch.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [_("Application Mode")],
        )
        self.app_mode_switch.connect("notify::active", self.on_app_mode_switch_changed)
        self.app_mode_row.add_suffix(self.app_mode_switch)
        group.add(self.app_mode_row)

        # Browser selection
        self.browser_row = Adw.ActionRow(title=_("Browser"))
        browser_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.browser_icon = Gtk.Image()
        self.browser_icon.set_pixel_size(24)
        self.set_browser_icon(self.webapp.browser)
        browser_box.append(self.browser_icon)
        self.browser_label = Gtk.Label()
        self.set_browser_label(self.webapp.browser)
        browser_box.append(self.browser_label)
        self.browser_row.add_prefix(browser_box)

        select_browser_button = Gtk.Button(label=_("Select"))
        select_browser_button.connect("clicked", self.on_select_browser_clicked)
        select_browser_button.set_valign(Gtk.Align.CENTER)
        self.browser_row.add_suffix(select_browser_button)
        group.add(self.browser_row)

        # Profile settings — progressive disclosure via expander
        browser = self.browser_collection.get_by_id(self.webapp.browser)
        is_firefox = browser and browser.is_firefox_based()

        self.profile_expander = Adw.ExpanderRow(title=_("Profile Settings"))
        self.profile_expander.set_subtitle(
            _("Configure a separate browser profile for this webapp")
        )

        profile_switch_row = Adw.ActionRow(title=_("Use separate profile"))
        profile_switch_row.set_subtitle(_("Allows independent cookies and sessions"))
        self.profile_switch = Gtk.Switch()
        self.profile_switch.set_valign(Gtk.Align.CENTER)
        self.profile_switch.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [_("Use separate profile")],
        )
        if self.is_new:
            self.profile_switch.set_active(False)
        else:
            self.profile_switch.set_active(self.webapp.app_profile != "Browser")
        self.profile_switch.connect("notify::active", self.on_profile_switch_changed)
        profile_switch_row.add_suffix(self.profile_switch)
        self.profile_expander.add_row(profile_switch_row)

        self.profile_entry_row = Adw.EntryRow()
        self.profile_entry_row.set_title(_("Profile Name"))
        self.profile_entry_row.set_text(self.webapp.app_profile)
        self.profile_entry_row.connect("changed", self.on_profile_entry_changed)
        self.profile_entry_row.set_visible(self.profile_switch.get_active())
        self.profile_expander.add_row(self.profile_entry_row)

        if not is_firefox:
            group.add(self.profile_expander)

        # hide browser/profile in app mode
        if self.webapp.app_mode == "app":
            self.browser_row.set_visible(False)
            self.profile_expander.set_visible(False)

    def _build_buttons(self) -> Gtk.Box:
        """Build Cancel / Save button bar."""
        box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        box.set_margin_top(12)
        box.set_margin_bottom(6)
        box.set_halign(Gtk.Align.END)
        box.set_valign(Gtk.Align.CENTER)

        cancel_button = Gtk.Button(label=_("Cancel"))
        cancel_button.set_valign(Gtk.Align.CENTER)
        cancel_button.connect("clicked", self.on_cancel_clicked)

        save_button = Gtk.Button(label=_("Save"))
        save_button.set_valign(Gtk.Align.CENTER)
        save_button.add_css_class("suggested-action")
        save_button.connect("clicked", self.on_save_clicked)

        box.append(cancel_button)
        box.append(save_button)
        return box

    def _build_loading_overlay(self, content: Gtk.Widget) -> Gtk.Overlay:
        """Wrap *content* in an overlay with a loading spinner."""
        overlay = Gtk.Overlay()
        overlay.set_child(content)

        self.loading_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        self.loading_box.set_valign(Gtk.Align.CENTER)
        self.loading_box.set_halign(Gtk.Align.CENTER)

        spinner = Gtk.Spinner()
        spinner.set_size_request(32, 32)
        spinner.start()
        self.loading_box.append(spinner)

        loading_label = Gtk.Label(label=_("Loading..."))
        loading_label.set_halign(Gtk.Align.CENTER)
        loading_label.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [_("Detecting website information, please wait")],
        )
        self.loading_box.append(loading_label)

        # semi-transparent backdrop
        loading_box_wrapper = Gtk.Box()
        loading_box_wrapper.set_hexpand(True)
        loading_box_wrapper.set_vexpand(True)
        css_provider = Gtk.CssProvider()
        css_provider.load_from_data(b"""
            box { background: rgba(0, 0, 0, 0.5); }
            label { color: white; }
        """)

        self.loading_box.set_hexpand(True)
        self.loading_box.set_vexpand(True)
        loading_box_wrapper.append(self.loading_box)

        style_context = loading_box_wrapper.get_style_context()
        Gtk.StyleContext.add_provider(
            style_context, css_provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        self.loading_overlay = loading_box_wrapper
        self.loading_overlay.set_visible(False)
        overlay.add_overlay(self.loading_overlay)
        return overlay

    def on_key_pressed(
        self,
        _controller: Gtk.EventControllerKey,
        keyval: int,
        _keycode: int,
        _state: Gdk.ModifierType,
    ) -> bool:
        """Handle key press events"""
        if keyval == Gdk.KEY_Escape:
            self.close()
            self.emit("response", Gtk.ResponseType.CANCEL)
            return True
        return False

    def set_icon_from_path(self, icon_path: str) -> None:

        try:
            if icon_path.startswith("/"):
                # Try to load from file
                pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_size(icon_path, 48, 48)
                self.icon_image.set_from_pixbuf(pixbuf)
            else:
                # Try to load as icon name
                self.icon_image.set_from_icon_name(icon_path)
        except Exception as e:
            logger.error("Error loading icon %s: %s", icon_path, e)
            self.icon_image.set_from_icon_name("webapp-generic")

    def set_browser_icon(self, browser_id: str) -> None:
        """Set the browser icon for the given browser ID."""
        set_image_from_browser_icon(self.browser_icon, browser_id, pixel_size=24)

    def set_browser_label(self, browser_id: str) -> None:
        """
        Set the browser label text

        Parameters:
            browser_id (str): Browser identifier
        """
        browser = self.browser_collection.get_by_id(browser_id)
        if browser:
            label_text = browser.get_friendly_name()
            self.browser_label.set_text(label_text)
        else:
            self.browser_label.set_text(browser_id)

    def on_url_changed(self, entry: Adw.EntryRow) -> None:
        """Handle URL entry changes with real-time validation."""
        url = entry.get_text()
        self.webapp.app_url = url
        self._validate_url(url)

    def _validate_url(self, url: str) -> None:
        """Update URL row visual feedback based on format validity."""
        from urllib.parse import urlparse

        self.url_row.remove_css_class("error")
        self.url_row.remove_css_class("success")

        if not url:
            self.url_valid_icon.set_visible(False)
            return

        parsed = urlparse(url)
        valid = parsed.scheme in ("http", "https") and bool(parsed.netloc)

        if valid:
            self.url_row.add_css_class("success")
            self.url_valid_icon.set_from_icon_name("emblem-ok-symbolic")
        else:
            self.url_row.add_css_class("error")
            self.url_valid_icon.set_from_icon_name("dialog-warning-symbolic")
        self.url_valid_icon.set_visible(True)

    def on_name_changed(self, entry: Adw.EntryRow) -> None:
        """Handle name entry changes"""
        self.webapp.app_name = entry.get_text()

    def on_category_changed(
        self, dropdown: Gtk.DropDown, _param: GObject.ParamSpec
    ) -> None:
        """Handle category dropdown changes"""
        selected = dropdown.get_selected()

        # Debug selected index and available categories
        logger.debug("Selected category index: %s", selected)
        logger.debug("System categories: %s", self.system_categories)

        # Use the system categories list to get the untranslated category name
        if hasattr(self, "system_categories") and 0 <= selected < len(
            self.system_categories
        ):
            # Get the original untranslated system category name
            system_category = self.system_categories[selected]

            # Make sure we're using the exact correct system category name
            # Force Development to exactly match the desktop entry standard
            if system_category.lower() == "development":
                system_category = "Development"

            self.webapp.set_main_category(system_category)
            logger.debug("Category set to: %s (system name)", system_category)
        else:
            # Fallback to the display name if something goes wrong
            model = dropdown.get_model()
            display_category = model.get_string(selected)
            self.webapp.set_main_category(display_category)
            logger.warning("Fallback: using display category: %s", display_category)

    def on_app_mode_switch_changed(
        self, switch: Gtk.Switch, _param: GObject.ParamSpec
    ) -> None:
        """Toggle between browser mode and application mode."""
        is_app = switch.get_active()
        self.webapp.app_mode = "app" if is_app else "browser"
        self.browser_row.set_visible(not is_app)
        self.profile_expander.set_visible(not is_app)

    def on_profile_switch_changed(
        self, switch: Gtk.Switch, _param: GObject.ParamSpec
    ) -> None:
        """Handle profile switch changes"""
        active = switch.get_active()
        self.profile_entry_row.set_visible(active)

        if not active:
            # Set to "Browser" when switch is off
            self.webapp.app_profile = "Browser"
            self.profile_entry_row.set_text("Browser")
        else:
            # Set to "Default" when switch is on and profile was "Browser"
            if self.webapp.app_profile == "Browser":
                self.webapp.app_profile = "Default"
                self.profile_entry_row.set_text("Default")

    def on_profile_entry_changed(self, entry: Adw.EntryRow) -> None:
        """Handle profile entry changes"""
        if self.profile_switch.get_active():
            self.webapp.app_profile = entry.get_text()

    def on_detect_clicked(self, button: Gtk.Button) -> None:
        """Handle detect button click"""
        url = self.webapp.app_url

        if not url:
            self.show_error_dialog(_("Please enter a URL first."))
            return

        # Show loading overlay
        self.loading_overlay.set_visible(True)

        # Create a WebsiteInfoFetcher and fetch info
        fetcher = WebsiteInfoFetcher()
        fetcher.fetch_info(url, self.on_website_info_fetched)

    def on_website_info_fetched(self, title: str, icon_paths: list[str]) -> None:
        """
        Handle fetched website information

        Parameters:
            title (str): Website title
            icon_paths (list): List of paths to downloaded icons
        """
        # Debug output to verify we're getting a title
        logger.debug("Website title detected: %s", title)

        # Update name if title was found
        if title:
            self.webapp.app_name = title
            self.name_row.set_text(title)

        # Derive profile name from URL
        profile_name = self.webapp.derive_profile_name()
        if self.profile_switch.get_active():
            self.webapp.app_profile = profile_name
            self.profile_entry_row.set_text(profile_name)

        if len(icon_paths) > 0:
            # Create FaviconPicker if it doesn't exist
            if not hasattr(self, "favicon_picker"):
                self.favicon_picker = FaviconPicker()
                self.favicon_picker.connect("icon-selected", self._on_icon_selected)
                self.icon_selection_row.set_child(self.favicon_picker)

            self.favicon_picker.load_icons(icon_paths)

            # Show the icon selection row
            self.icon_selection_row.set_visible(True)
        else:
            # Hide the icon selection row if no icons
            self.icon_selection_row.set_visible(False)

        # Hide the loading overlay
        self.loading_overlay.set_visible(False)

        # move focus to name entry → screen reader announces result
        self.name_row.grab_focus()

    def _on_icon_selected(self, _picker: FaviconPicker, icon_path: str) -> None:
        """Handle icon selection from FaviconPicker."""
        self.webapp.app_icon_url = icon_path
        self.set_icon_from_path(icon_path)

    def on_select_icon_clicked(self, button: Gtk.Button) -> None:
        """Handle select icon button click"""
        icon_path = self.command_executor.select_icon()

        if icon_path:
            self.webapp.app_icon_url = icon_path
            self.set_icon_from_path(icon_path)

    def on_select_browser_clicked(self, button: Gtk.Button) -> None:
        """Handle browser selection button click."""
        dialog = BrowserDialog(
            self,  # Use self (WebAppDialog) as the parent
            self.webapp,
            self.browser_collection,
        )
        # Connect to response signal
        dialog.connect("response", self.on_browser_dialog_response)
        # Show the dialog
        dialog.present()

    def on_browser_dialog_response(self, dialog: BrowserDialog, response: int) -> None:
        """Handle browser dialog response."""
        if response == Gtk.ResponseType.OK:
            browser = dialog.get_selected_browser()
            if browser:
                # Store original properties before update (for debugging)
                original_browser = self.webapp.browser

                # Update only the local webapp browser property (don't save to disk yet)
                self.webapp.browser = browser.browser_id

                # Update the UI to show the new browser immediately
                self.set_browser_icon(browser.browser_id)
                self.set_browser_label(browser.browser_id)

                # Handle profile settings for Firefox-based browsers
                if browser.is_firefox_based():
                    self.profile_expander.set_visible(False)
                    self.webapp.app_profile = "Default"
                else:
                    self.profile_expander.set_visible(True)

                # Don't update the webapp file yet - this will be done in on_save_clicked
                logger.debug(
                    "Browser selected: %s → %s (will be saved when clicking Save)",
                    original_browser,
                    self.webapp.browser,
                )

    def on_cancel_clicked(self, button: Gtk.Button) -> None:
        """Handle cancel button click."""
        self.close()
        self.emit("response", Gtk.ResponseType.CANCEL)

    def on_save_clicked(self, button: Gtk.Button) -> None:
        """Handle save button click"""
        # Validate required fields
        if not self.webapp.app_name:
            self.show_error_dialog(_("Please enter a name for the WebApp."))
            return

        if not self.webapp.app_url:
            self.show_error_dialog(_("Please enter a URL for the WebApp."))
            return

        if not self.webapp.browser and self.webapp.app_mode != "app":
            self.show_error_dialog(_("Please select a browser for the WebApp."))
            return

        # Generate a new app_file for new webapps
        if self.is_new:
            timestamp = int(time.time())
            random_suffix = uuid.uuid4().hex[:8]
            self.webapp.app_file = f"{timestamp}-{random_suffix}"

        # show saving indicator
        self.loading_overlay.set_visible(True)

        def _do_save() -> None:
            if self.is_new:
                ok = self.command_executor.create_webapp(self.webapp)
            else:
                ok = self.command_executor.update_webapp(self.webapp)
            GLib.idle_add(self._on_save_finished, ok)

        threading.Thread(target=_do_save, daemon=True).start()

    def _on_save_finished(self, success: bool) -> None:
        """Called on main thread after save completes."""
        self.loading_overlay.set_visible(False)
        self.close()
        self.emit(
            "response", Gtk.ResponseType.OK if success else Gtk.ResponseType.CANCEL
        )

    def show_error_dialog(self, message: str) -> None:
        """
        Show an error dialog

        Parameters:
            message (str): Error message to display
        """
        dialog = Adw.MessageDialog(transient_for=self, heading=_("Error"), body=message)
        dialog.add_response("ok", _("OK"))
        dialog.present()

    def get_webapp(self) -> WebApp:
        """
        Get the edited webapp

        Returns:
            WebApp: The edited WebApp object
        """
        return self.webapp
