"""
WebAppDialog module containing the dialog for creating and editing webapps
"""

import gi
import time
import uuid

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, GObject, GdkPixbuf, Gdk

# Import BrowserDialog
from webapps.ui.browser_dialog import BrowserDialog

# Import our new WebsiteInfoFetcher
from webapps.utils.url_utils import WebsiteInfoFetcher

# Import the browser icon utilities
from webapps.utils.browser_icon_utils import set_image_from_browser_icon

# Import the centralized translation function
from webapps.utils.translation import _


class WebAppDialog(Adw.Window):
    """Dialog for creating and editing webapps"""

    # Define custom signals
    __gsignals__ = {"response": (GObject.SignalFlags.RUN_FIRST, None, (int,))}

    def __init__(
        self, parent, webapp, browser_collection, command_executor, is_new=False
    ):
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

        # Try to detect the system default browser
        self.system_default_browser_id = None
        if self.command_executor:
            self.system_default_browser_id = (
                self.command_executor.get_system_default_browser()
            )
            print(f"System default browser detected: {self.system_default_browser_id}")

        # For new webapps, always use the system default browser if available
        if self.is_new and self.system_default_browser_id:
            # Check if this browser exists in our collection
            system_browser = self.browser_collection.get_by_id(
                self.system_default_browser_id
            )
            if system_browser:
                # Override any previously selected browser with the system default
                original_browser = self.webapp.browser
                self.webapp.browser = self.system_default_browser_id
                print(
                    f"Overriding browser selection: {original_browser} → {self.system_default_browser_id}"
                )
            else:
                # Fallback to the app's default browser if the system browser isn't supported
                if not self.webapp.browser:  # Only if no browser is already selected
                    default_browser = self.browser_collection.get_default()
                    if default_browser:
                        self.webapp.browser = default_browser.browser_id
                        print(
                            f"System browser not supported, using app default: {default_browser.browser_id}"
                        )
        # If no browser is set at all and system detection failed, use the app's default browser
        elif self.is_new and not self.webapp.browser:
            default_browser = self.browser_collection.get_default()
            if default_browser:
                self.webapp.browser = default_browser.browser_id
                print(f"Using app default browser: {default_browser.browser_id}")

        # Create UI
        self.setup_ui()

    def _clone_webapp(self, webapp):
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
        }

        return WebApp(webapp_dict)

    def setup_ui(self):
        """Set up the UI components"""
        # Create main layout with content area
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
        header.set_show_end_title_buttons(True)  # Show window controls on the right

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

        # Create a central container for vertical centering
        central_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        central_box.set_vexpand(True)  # Allow vertical expansion
        central_box.set_valign(Gtk.Align.CENTER)  # Center vertically

        # Create scrollable content area
        scrolled = Gtk.ScrolledWindow()
        scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scrolled.set_propagate_natural_height(True)
        scrolled.set_min_content_height(300)

        # Main content using Adw.Clamp for proper width constraints
        clamp = Adw.Clamp()
        clamp.set_maximum_size(600)
        clamp.set_tightening_threshold(400)

        # Form content
        form_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        form_box.set_margin_top(0)
        form_box.set_margin_bottom(12)
        form_box.set_margin_start(24)  # Increased for better horizontal spacing
        form_box.set_margin_end(24)  # Increased for better horizontal spacing

        # Create single preferences group for form elements
        form_group = Adw.PreferencesGroup()

        # URL entry with detect button
        url_row = Adw.EntryRow()
        url_row.set_title(_("URL"))
        url_row.set_text(self.webapp.app_url)

        # Add detect button with consistent spacing
        detect_button = Gtk.Button(label=_("Detect"))
        detect_button.set_tooltip_text(_("Detect name and icon from website"))
        detect_button.set_valign(Gtk.Align.CENTER)
        detect_button.connect("clicked", self.on_detect_clicked)
        url_row.add_suffix(detect_button)
        url_row.connect("changed", self.on_url_changed)
        form_group.add(url_row)

        # Name entry
        name_row = Adw.EntryRow()
        name_row.set_title(_("Name"))
        name_row.set_text(self.webapp.app_name)
        name_row.connect("changed", self.on_name_changed)
        form_group.add(name_row)

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
        form_group.add(icon_row)

        # Icons row that will appear after icon selection when icons are available
        self.icon_selection_row = Adw.ActionRow(title=_("Available Icons"))
        self.icon_selection_row.set_visible(False)
        form_group.add(self.icon_selection_row)

        # Category selection
        main_category = self.webapp.get_main_category()
        self.category_dropdown = Gtk.DropDown()
        category_model = Gtk.StringList()

        # Store both system and display names for categories
        self.system_categories = [
            "Webapps",  # Our custom category - always first
            "Network",  # Common in most DEs
            "Office",  # Common in most DEs
            "Development",  # Common in most DEs
            "Graphics",  # Common in most DEs
            "AudioVideo",  # Common name in some DEs
            "Game",  # Common in most DEs
            "Utility",  # Common in GNOME
            "System",  # Common in most DEs
        ]

        # Display translated category names in UI
        for category in self.system_categories:
            category_model.append(_(category))

        self.category_dropdown.set_model(category_model)
        self.category_dropdown.set_valign(Gtk.Align.CENTER)

        # Set the current category - find the untranslated one that matches our current category
        for i, category in enumerate(self.system_categories):
            if category == main_category:
                self.category_dropdown.set_selected(i)
                break

        self.category_dropdown.connect("notify::selected", self.on_category_changed)

        category_row = Adw.ActionRow(title=_("Category"))
        category_row.add_suffix(self.category_dropdown)
        form_group.add(category_row)

        # Browser selection
        browser_row = Adw.ActionRow(title=_("Browser"))

        browser_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.browser_icon = Gtk.Image()
        self.browser_icon.set_pixel_size(24)
        self.set_browser_icon(self.webapp.browser)
        browser_box.append(self.browser_icon)

        self.browser_label = Gtk.Label()
        self.set_browser_label(self.webapp.browser)
        browser_box.append(self.browser_label)

        browser_row.add_prefix(browser_box)

        select_browser_button = Gtk.Button(label=_("Select"))
        select_browser_button.connect("clicked", self.on_select_browser_clicked)
        select_browser_button.set_valign(Gtk.Align.CENTER)
        browser_row.add_suffix(select_browser_button)
        form_group.add(browser_row)

        # Profile settings
        browser = self.browser_collection.get_by_id(self.webapp.browser)
        is_firefox = browser and browser.is_firefox_based()

        # Profile switch (only for non-Firefox browsers)
        self.profile_row = Adw.ActionRow(title=_("Use separate profile"))
        self.profile_row.set_subtitle(
            _("Using a separate profile allows you to log in to different accounts")
        )

        self.profile_switch = Gtk.Switch()
        self.profile_switch.set_valign(Gtk.Align.CENTER)

        # For new webapps, always set the switch to inactive by default
        # For existing webapps, set based on profile name
        if self.is_new:
            self.profile_switch.set_active(False)
        else:
            self.profile_switch.set_active(self.webapp.app_profile != "Browser")

        self.profile_switch.connect("notify::active", self.on_profile_switch_changed)
        self.profile_row.add_suffix(self.profile_switch)

        # Profile entry (only visible when switch is on)
        self.profile_entry_row = Adw.EntryRow()
        self.profile_entry_row.set_title(_("Profile Name"))
        self.profile_entry_row.set_text(self.webapp.app_profile)
        self.profile_entry_row.connect("changed", self.on_profile_entry_changed)
        self.profile_entry_row.set_visible(self.profile_switch.get_active())

        # Hide profile options for Firefox-based browsers
        if not is_firefox:
            form_group.add(self.profile_row)
            form_group.add(self.profile_entry_row)

        # Add form group to the layout
        form_box.append(form_group)

        # Favicons section (initially hidden)
        self.favicons_group = Adw.PreferencesGroup(title=_("Available Icons"))
        self.favicons_box = Gtk.FlowBox()
        self.favicons_box.set_selection_mode(Gtk.SelectionMode.SINGLE)
        self.favicons_box.set_max_children_per_line(5)
        self.favicons_box.set_homogeneous(True)
        self.favicons_box.connect("child-activated", self.on_favicon_selected)

        favicons_scroll = Gtk.ScrolledWindow()
        favicons_scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        favicons_scroll.set_min_content_height(100)
        favicons_scroll.set_max_content_height(200)
        favicons_scroll.set_child(self.favicons_box)

        self.favicons_group.add(favicons_scroll)
        self.favicons_group.set_visible(False)
        form_box.append(self.favicons_group)

        # Add bottom button bar with cancel and save buttons - reduce margins
        button_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        button_box.set_margin_top(12)  # Reduced from 24
        button_box.set_margin_bottom(6)  # Reduced from 12
        button_box.set_halign(Gtk.Align.END)
        button_box.set_valign(Gtk.Align.CENTER)  # Ensure vertical centering

        cancel_button = Gtk.Button(label=_("Cancel"))
        cancel_button.set_valign(Gtk.Align.CENTER)
        cancel_button.connect("clicked", self.on_cancel_clicked)

        save_button = Gtk.Button(label=_("Save"))
        save_button.set_valign(Gtk.Align.CENTER)
        save_button.add_css_class("suggested-action")
        save_button.connect("clicked", self.on_save_clicked)

        button_box.append(cancel_button)
        button_box.append(save_button)

        # Add button box to the form
        form_box.append(button_box)

        # Complete the content hierarchy with the central container
        clamp.set_child(form_box)
        scrolled.set_child(clamp)
        central_box.append(scrolled)
        content.append(central_box)

        # Add a loading spinner overlay (initially hidden)
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
        loading_label.set_halign(Gtk.Align.CENTER)  # Center the label horizontally
        self.loading_box.append(loading_label)

        # Set background color for loading overlay
        loading_overlay = Gtk.Box()
        loading_overlay.set_hexpand(True)
        loading_overlay.set_vexpand(True)
        css_provider = Gtk.CssProvider()
        css_provider.load_from_data(b"""
            box {
            background: rgba(0, 0, 0, 0.5);
            }
            label {
            color: white;
            }
        """)

        # Center the loading box inside the overlay
        self.loading_box.set_hexpand(True)
        self.loading_box.set_vexpand(True)
        loading_overlay.append(self.loading_box)

        style_context = loading_overlay.get_style_context()
        Gtk.StyleContext.add_provider(
            style_context, css_provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        loading_overlay.append(self.loading_box)

        self.loading_overlay = loading_overlay
        self.loading_overlay.set_visible(False)

        overlay.add_overlay(self.loading_overlay)

        # Use set_content() instead of set_child() for Adw.Window
        self.set_content(overlay)

    def on_key_pressed(self, controller, keyval, keycode, state):
        """Handle key press events"""
        if keyval == Gdk.KEY_Escape:
            self.close()
            self.emit("response", Gtk.ResponseType.CANCEL)
            return True
        return False

    def set_icon_from_path(self, icon_path):
        """
        Set the icon from a file path or icon name

        Parameters:
            icon_path (str): Path to the icon file or icon name
        """
        if not icon_path:
            self.icon_image.set_from_icon_name("webapp-generic")
            return

        try:
            if icon_path.startswith("/"):
                # Try to load from file
                pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_size(icon_path, 48, 48)
                self.icon_image.set_from_pixbuf(pixbuf)
            else:
                # Try to load as icon name
                self.icon_image.set_from_icon_name(icon_path)
        except Exception as e:
            print(f"Error loading icon {icon_path}: {e}")
            self.icon_image.set_from_icon_name("webapp-generic")

    def set_browser_icon(self, browser_id):
        """
        Set the browser icon

        Parameters:
            browser_id (str): Browser identifier
        """
        set_image_from_browser_icon(self.browser_icon, browser_id, pixel_size=24)

    def set_browser_label(self, browser_id):
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

    def on_url_changed(self, entry):
        """Handle URL entry changes"""
        self.webapp.app_url = entry.get_text()

    def on_name_changed(self, entry):
        """Handle name entry changes"""
        self.webapp.app_name = entry.get_text()

    def on_category_changed(self, dropdown, param):
        """Handle category dropdown changes"""
        selected = dropdown.get_selected()

        # Debug selected index and available categories
        print(f"Selected category index: {selected}")
        print(f"System categories: {self.system_categories}")

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
            print(f"Category set to: {system_category} (system name)")
        else:
            # Fallback to the display name if something goes wrong
            model = dropdown.get_model()
            display_category = model.get_string(selected)
            self.webapp.set_main_category(display_category)
            print(f"Fallback: using display category: {display_category}")

    def on_profile_switch_changed(self, switch, param):
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

    def on_profile_entry_changed(self, entry):
        """Handle profile entry changes"""
        if self.profile_switch.get_active():
            self.webapp.app_profile = entry.get_text()

    def on_detect_clicked(self, button):
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

    def on_website_info_fetched(self, title, icon_paths):
        """
        Handle fetched website information

        Parameters:
            title (str): Website title
            icon_paths (list): List of paths to downloaded icons
        """
        # Debug output to verify we're getting a title
        print(f"Website title detected: {title}")

        # Update name if title was found
        if title:
            self.webapp.app_name = title

            # Find all entry rows in the dialog content and update the one with title "Name"
            for row in self.find_all_widget_types(self.get_content(), Adw.EntryRow):
                if row.get_title() == _("Name"):
                    print(f"Found Name entry, setting text to: {title}")
                    row.set_text(title)
                    break

        # Derive profile name from URL
        profile_name = self.webapp.derive_profile_name()
        if self.profile_switch.get_active():
            self.webapp.app_profile = profile_name
            self.profile_entry_row.set_text(profile_name)

        if len(icon_paths) > 0:
            # Create a flowbox for the icons if it doesn't exist
            if not hasattr(self, "icons_flowbox"):
                self.icons_flowbox = Gtk.FlowBox()
                self.icons_flowbox.set_selection_mode(Gtk.SelectionMode.SINGLE)
                self.icons_flowbox.set_max_children_per_line(5)
                self.icons_flowbox.set_homogeneous(True)
                self.icons_flowbox.connect("child-activated", self.on_favicon_selected)
                self.icons_flowbox.set_margin_top(8)
                self.icons_flowbox.set_margin_bottom(8)
                self.icon_selection_row.set_child(self.icons_flowbox)
            else:
                # Clear existing icons
                while self.icons_flowbox.get_first_child():
                    self.icons_flowbox.remove(self.icons_flowbox.get_first_child())

            # Add icons to the flowbox in the row
            for icon_path in icon_paths:
                container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
                container.favicon_url = icon_path

                image = Gtk.Image()
                image.set_pixel_size(48)

                try:
                    pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_size(icon_path, 48, 48)
                    image.set_from_pixbuf(pixbuf)
                except Exception as e:
                    print(f"Error loading favicon {icon_path}: {e}")
                    image.set_from_icon_name("image-missing")

                container.append(image)
                self.icons_flowbox.append(container)

            # Show the icon selection row
            self.icon_selection_row.set_visible(True)

            # Hide the old favicons group since we're now using the row
            self.favicons_group.set_visible(False)
        else:
            # Hide the icon selection row if no icons
            self.icon_selection_row.set_visible(False)

        # Hide the loading overlay
        self.loading_overlay.set_visible(False)

    def on_favicon_selected(self, flowbox, child):
        """Handle favicon selection"""
        # Get the container box
        container = child.get_child()

        # Get the favicon URL from the container property
        favicon_url = getattr(container, "favicon_url", None)

        if favicon_url:
            self.webapp.app_icon_url = favicon_url
            self.set_icon_from_path(favicon_url)

    def on_select_icon_clicked(self, button):
        """Handle select icon button click"""
        icon_path = self.command_executor.select_icon()

        if icon_path:
            self.webapp.app_icon_url = icon_path
            self.set_icon_from_path(icon_path)

    def on_select_browser_clicked(self, button):
        """Handle select browser button click"""
        # Create the browser dialog with the same approach used in MainWindow
        dialog = BrowserDialog(
            self,  # Use self (WebAppDialog) as the parent
            self.webapp,
            self.browser_collection,
        )
        # Connect to response signal
        dialog.connect("response", self.on_browser_dialog_response)
        # Show the dialog
        dialog.present()

    def on_browser_dialog_response(self, dialog, response):
        """Handle browser dialog response"""
        if response == Gtk.ResponseType.OK:
            # Get the selected browser
            browser = dialog.get_selected_browser()

            # Store original properties before update (for debugging)
            original_browser = self.webapp.browser

            # Update only the local webapp browser property (don't save to disk yet)
            self.webapp.browser = browser.browser_id

            # Update the UI to show the new browser immediately
            self.set_browser_icon(browser.browser_id)
            self.set_browser_label(browser.browser_id)

            # Handle profile settings for Firefox-based browsers
            if browser.is_firefox_based():
                self.profile_row.set_visible(False)
                self.profile_entry_row.set_visible(False)
                self.webapp.app_profile = "Default"
            else:
                self.profile_row.set_visible(True)
                self.profile_entry_row.set_visible(self.profile_switch.get_active())

            # Don't update the webapp file yet - this will be done in on_save_clicked
            print(
                f"Browser selected: {original_browser} → {self.webapp.browser} (will be saved when clicking Save)"
            )

    def on_cancel_clicked(self, button):
        """Handle cancel button click"""
        self.close()
        self.emit("response", Gtk.ResponseType.CANCEL)

    def on_save_clicked(self, button):
        """Handle save button click"""
        # Validate required fields
        if not self.webapp.app_name:
            self.show_error_dialog(_("Please enter a name for the WebApp."))
            return

        if not self.webapp.app_url:
            self.show_error_dialog(_("Please enter a URL for the WebApp."))
            return

        if not self.webapp.browser:
            self.show_error_dialog(_("Please select a browser for the WebApp."))
            return

        # Generate a new app_file for new webapps
        if self.is_new:
            timestamp = int(time.time())
            random_suffix = uuid.uuid4().hex[:8]
            self.webapp.app_file = f"{timestamp}-{random_suffix}"

        # Everything is valid, close and emit response
        self.close()
        self.emit("response", Gtk.ResponseType.OK)

    def show_error_dialog(self, message):
        """
        Show an error dialog

        Parameters:
            message (str): Error message to display
        """
        dialog = Adw.MessageDialog(transient_for=self, heading=_("Error"), body=message)
        dialog.add_response("ok", _("OK"))
        dialog.present()

    def get_webapp(self):
        """
        Get the edited webapp

        Returns:
            WebApp: The edited WebApp object
        """
        return self.webapp

    def find_all_widget_types(self, widget, widget_type):
        """
        Recursively find all widgets of a specific type

        Parameters:
            widget (Gtk.Widget): Widget to search in
            widget_type (type): Type of widgets to find

        Returns:
            list: List of widgets of the specified type
        """
        result = []

        # If this widget is of the target type, add it
        if isinstance(widget, widget_type):
            result.append(widget)

        # If it's a container with children, search its children
        if hasattr(widget, "get_first_child"):
            child = widget.get_first_child()
            while child:
                result.extend(self.find_all_widget_types(child, widget_type))
                child = child.get_next_sibling()

        return result
