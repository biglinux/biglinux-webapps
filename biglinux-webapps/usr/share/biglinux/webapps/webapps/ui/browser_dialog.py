"""
BrowserDialog module containing the dialog for selecting a browser
"""

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, GObject, Gdk

# Import shared browser icon utilities
from webapps.utils.browser_icon_utils import set_image_from_browser_icon
from webapps.utils.translation import _


class BrowserDialog(Adw.Window):
    """Dialog for selecting a browser for a webapp"""

    # Define custom signals
    __gsignals__ = {"response": (GObject.SignalFlags.RUN_FIRST, None, (int,))}

    def __init__(self, parent, webapp, browser_collection):
        """Initialize the BrowserDialog"""
        super().__init__(
            transient_for=parent,
            modal=True,
            destroy_with_parent=True,
            width_request=400,
            height_request=500,
        )

        self.webapp = webapp
        self.browser_collection = browser_collection
        self.selected_browser = None

        # Get the command executor from parent if available
        # This is needed to detect the system default browser
        if hasattr(parent, "command_executor"):
            self.command_executor = parent.command_executor
        else:
            self.command_executor = None

        # Create UI
        self.setup_ui()

    def setup_ui(self):
        """Set up the UI components"""
        # Create content area
        content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        content.set_margin_top(2)
        content.set_margin_bottom(12)
        content.set_margin_start(12)
        content.set_margin_end(12)

        # Add key event controller to handle ESC key to close dialog
        key_controller = Gtk.EventControllerKey.new()
        key_controller.connect("key-pressed", self.on_key_pressed)
        self.add_controller(key_controller)

        # Header
        header = Adw.HeaderBar()
        header.set_title_widget(Gtk.Label(label=_("Select Browser")))
        header.add_css_class("flat")
        content.append(header)

        # Create a list box for browser options
        list_box = Gtk.ListBox()
        list_box.set_selection_mode(Gtk.SelectionMode.SINGLE)
        list_box.add_css_class("boxed-list")
        list_box.connect("row-selected", self.on_browser_selected)

        # Get all browsers
        browsers = self.browser_collection.get_all()

        # Try to get system default browser ID if command executor is available
        self.system_default_browser_id = None
        if self.command_executor and hasattr(
            self.command_executor, "get_system_default_browser"
        ):
            self.system_default_browser_id = (
                self.command_executor.get_system_default_browser()
            )
            print(f"System default browser detected: {self.system_default_browser_id}")

        # Add browser options to the list box
        for browser in browsers:
            row = self._create_browser_row(browser)
            list_box.append(row)

            # Select browser in the list - prioritize:
            # 1. Current browser from webapp (if set)
            # 2. System default browser (if detected and no current browser)
            if browser.browser_id == self.webapp.browser:
                list_box.select_row(row)
                self.selected_browser = browser
                print(f"Selected existing browser: {browser.browser_id}")
            elif (
                not self.webapp.browser
                and not self.selected_browser
                and self.system_default_browser_id
                and browser.browser_id == self.system_default_browser_id
            ):
                list_box.select_row(row)
                self.selected_browser = browser
                print(f"Selected system default browser: {browser.browser_id}")

        # Add the list box to a scrolled window
        scrolled = Gtk.ScrolledWindow()
        scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scrolled.set_min_content_height(400)
        scrolled.set_child(list_box)

        content.append(scrolled)

        # Add buttons
        button_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        button_box.set_halign(Gtk.Align.END)
        button_box.set_spacing(8)
        button_box.set_margin_top(12)

        cancel_button = Gtk.Button(label=_("Cancel"))
        cancel_button.connect("clicked", self.on_cancel_clicked)

        select_button = Gtk.Button(label=_("Select"))
        select_button.add_css_class("suggested-action")
        select_button.connect("clicked", self.on_select_clicked)

        button_box.append(cancel_button)
        button_box.append(select_button)
        content.append(button_box)

        # Use set_content() instead of set_child() for Adw.Window
        self.set_content(content)

    def _create_browser_row(self, browser):
        """
        Create a row for a browser

        Parameters:
            browser (Browser): Browser object

        Returns:
            Gtk.ListBoxRow: Row for the browser
        """
        row = Gtk.ListBoxRow()

        # Store the browser in the row
        row.browser = browser

        # Create a box for the row content
        box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        box.set_margin_top(8)
        box.set_margin_bottom(8)
        box.set_margin_start(8)
        box.set_margin_end(8)

        # Browser icon
        icon = Gtk.Image()
        set_image_from_browser_icon(icon, browser, pixel_size=32)
        box.append(icon)

        # Browser name
        label = Gtk.Label(label=browser.get_friendly_name())
        label.set_halign(Gtk.Align.START)
        label.set_hexpand(True)
        box.append(label)

        # Default indicator - only show System Default, or Default if no system default is detected
        is_system_default = (
            hasattr(self, "system_default_browser_id")
            and self.system_default_browser_id == browser.browser_id
        )

        if is_system_default:
            default_label = Gtk.Label(label=_("System Default"))
            default_label.add_css_class("caption")
            default_label.add_css_class("dim-label")
            box.append(default_label)
        elif browser.is_default and not self.system_default_browser_id:
            # Only show this if no system default was detected
            default_label = Gtk.Label(label=_("Default"))
            default_label.add_css_class("caption")
            default_label.add_css_class("dim-label")
            box.append(default_label)

        row.set_child(box)

        return row

    def on_browser_selected(self, list_box, row):
        """Handle browser selection"""
        if row:
            self.selected_browser = row.browser

    def on_cancel_clicked(self, button):
        """Handle cancel button click"""
        self.close()
        # Emit our custom response signal
        self.emit("response", Gtk.ResponseType.CANCEL)

    def on_select_clicked(self, button):
        """Handle select button click"""
        if self.selected_browser:
            self.close()
            # Emit our custom response signal
            self.emit("response", Gtk.ResponseType.OK)
        else:
            self.show_error_dialog(_("Please select a browser."))

    def show_error_dialog(self, message):
        """
        Show an error dialog

        Parameters:
            message (str): Error message to display
        """
        dialog = Adw.MessageDialog(transient_for=self, heading=_("Error"), body=message)
        dialog.add_response("ok", _("OK"))
        dialog.present()

    def get_selected_browser(self):
        """
        Get the selected browser

        Returns:
            Browser: Selected browser
        """
        return self.selected_browser

    def on_key_pressed(self, controller, keyval, keycode, state):
        """Handle key press events"""
        if keyval == Gdk.KEY_Escape:
            self.close()
            self.emit("response", Gtk.ResponseType.CANCEL)
            return True
        return False
