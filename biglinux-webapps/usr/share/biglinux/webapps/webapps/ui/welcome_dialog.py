"""
Welcome dialog shown on application startup
"""

import os
import gi
import json

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, Gdk

from webapps.utils.translation import _


class WelcomeDialog(Adw.Window):
    """Welcome dialog explaining what webapps are and their benefits"""

    def __init__(self, parent_window):
        """Initialize the welcome dialog"""
        super().__init__(
            title=_("Welcome to WebApps Manager"),
            transient_for=parent_window,
            modal=True,
            destroy_with_parent=True,
            width_request=640,
            height_request=400,
        )

        self.parent_window = parent_window
        self.config_file = os.path.expanduser(
            "~/.config/biglinux-webapps/welcome_shown.json"
        )

        # Set up key event controller for ESC key
        key_controller = Gtk.EventControllerKey()
        key_controller.connect("key-pressed", self.on_key_pressed)
        self.add_controller(key_controller)

        self.setup_ui()

    def on_key_pressed(self, controller, keyval, keycode, state):
        """Handle key press events"""
        # Check if ESC key was pressed
        if keyval == Gdk.KEY_Escape:
            self.destroy()
            return True
        return False

    def setup_ui(self):
        """Set up the UI components"""
        # Main container - similar to browser_dialog.py approach
        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)

        # Add a headerbar for window movement with custom styling
        headerbar = Adw.HeaderBar()
        headerbar.set_show_title(False)  # No title for cleaner look
        headerbar.add_css_class("flat")  # Make it less prominent

        # Apply custom CSS to reduce header padding
        css_provider = Gtk.CssProvider()
        css_provider.load_from_data(b"""
            headerbar {
                min-height: 38px;
                padding: 2px 6px;
            }
        """)
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(),
            css_provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
        )

        main_box.append(headerbar)

        # Content container
        content_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        content_box.set_margin_top(12)  # Reduced from 24 to match browser_dialog style
        content_box.set_margin_bottom(24)
        content_box.set_margin_start(24)
        content_box.set_margin_end(24)

        # Header with icon
        header_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        header_box.set_halign(Gtk.Align.CENTER)

        app_icon = Gtk.Image.new_from_icon_name("big-webapps")
        app_icon.set_pixel_size(64)
        header_box.append(app_icon)

        title = Gtk.Label()
        title.set_markup(
            "<span size='x-large' weight='bold'>"
            + _("Welcome to WebApps Manager")
            + "</span>"
        )
        header_box.append(title)

        content_box.append(header_box)

        # Explanation text
        explanation = Gtk.Label()
        explanation.set_wrap(True)
        explanation.set_max_width_chars(60)
        explanation.set_margin_top(12)
        explanation.set_margin_bottom(12)
        explanation.set_markup(
            _(
                "<b>What are WebApps?</b>\n\n"
                "WebApps are web applications that run in a dedicated browser window, "
                "providing a more app-like experience for your favorite websites.\n\n"
                "<b>Benefits of using WebApps:</b>\n\n"
                "• <b>Focus</b>: Work without the distractions of other browser tabs\n"
                "• <b>Desktop Integration</b>: Quick access from your application menu\n"
                "• <b>Isolated Profiles</b>: Optionally, each webapp can have its own cookies and settings\n"
            )
        )
        explanation.set_halign(Gtk.Align.START)
        content_box.append(explanation)

        # Separator before switch
        separator = Gtk.Separator(orientation=Gtk.Orientation.HORIZONTAL)
        separator.set_margin_top(12)
        content_box.append(separator)

        # Don't show again switch
        switch_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        switch_box.set_margin_top(12)

        self.show_switch = Gtk.Switch()
        self.show_switch.set_active(False)
        self.show_switch.set_valign(Gtk.Align.CENTER)

        switch_label = Gtk.Label(label=_("Show dialog on startup"))
        switch_label.set_xalign(0)
        switch_label.set_hexpand(True)

        switch_box.append(switch_label)
        switch_box.append(self.show_switch)

        # Set initial state based on saved preference
        self.show_switch.set_active(self.get_show_preference())

        content_box.append(switch_box)

        # Close button
        button_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        button_box.set_halign(Gtk.Align.CENTER)
        button_box.set_margin_top(24)

        close_button = Gtk.Button(label=_("Let's Start"))
        close_button.add_css_class("suggested-action")
        close_button.connect("clicked", self.on_close)

        button_box.append(close_button)
        content_box.append(button_box)

        # Add content box to main box
        main_box.append(content_box)

        # Set the content
        self.set_content(main_box)

    def on_close(self, button):
        """Handle close button click"""
        # Save preference based on switch state
        self.save_preference(show=self.show_switch.get_active())

        # Close the dialog
        self.destroy()

    def get_show_preference(self):
        """Get the current preference for showing the dialog at startup"""
        # If the file doesn't exist, default is to show the dialog
        if not os.path.exists(self.config_file):
            return True

        # Read the file and check the preference
        try:
            with open(self.config_file, "r") as f:
                preferences = json.load(f)
                # Return whether we should show the dialog (direct from saved setting)
                return preferences.get("show_welcome", True)
        except Exception:
            # If there's an error reading the file, default to showing the dialog
            return True

    def save_preference(self, show=True):
        """
        Save the preference for showing the welcome dialog

        Parameters:
            show (bool): If True, show the dialog; if False, don't show it
        """
        # Make sure the directory exists
        os.makedirs(os.path.dirname(self.config_file), exist_ok=True)

        # Save preference directly
        preferences = {"show_welcome": show}
        try:
            with open(self.config_file, "w") as f:
                json.dump(preferences, f)
        except Exception as e:
            print(f"Error saving welcome dialog preference: {e}")

    @staticmethod
    def should_show_welcome():
        """Check if the welcome dialog should be shown"""
        config_file = os.path.expanduser(
            "~/.config/biglinux-webapps/welcome_shown.json"
        )

        # If the file doesn't exist, we should show the dialog
        if not os.path.exists(config_file):
            return True

        # Read the file and check the preference
        try:
            with open(config_file, "r") as f:
                preferences = json.load(f)
                return preferences.get("show_welcome", True)
        except Exception:
            # If there's any error reading the file, show the dialog
            return True
