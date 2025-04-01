"""
WebAppRow module containing the row widget for displaying a webapp in a list
"""

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, GObject, GdkPixbuf

# Import shared browser icon utilities
from webapps.utils.browser_icon_utils import set_image_from_browser_icon


class WebAppRow(Gtk.ListBoxRow):
    """Row widget for displaying a webapp in a list"""

    __gsignals__ = {
        "edit-clicked": (GObject.SignalFlags.RUN_FIRST, None, (GObject.TYPE_PYOBJECT,)),
        "browser-clicked": (
            GObject.SignalFlags.RUN_FIRST,
            None,
            (GObject.TYPE_PYOBJECT,),
        ),
        "delete-clicked": (
            GObject.SignalFlags.RUN_FIRST,
            None,
            (GObject.TYPE_PYOBJECT,),
        ),
    }

    def __init__(self, webapp, browser_collection):
        """Initialize the WebAppRow"""
        super().__init__()
        self.webapp = webapp
        self.browser_collection = browser_collection
        self.setup_ui()

    def setup_ui(self):
        """Set up the UI components"""
        # Main box
        box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        box.set_margin_top(8)
        box.set_margin_bottom(8)
        box.set_margin_start(8)
        box.set_margin_end(8)

        # App icon
        self.icon = Gtk.Image()
        self.icon.set_pixel_size(48)
        self.set_icon_from_path(self.webapp.app_icon_url)
        box.append(self.icon)

        # App info
        info_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        info_box.set_hexpand(True)

        # App name
        name_label = Gtk.Label(label=self.webapp.app_name)
        name_label.set_halign(Gtk.Align.START)
        name_label.set_wrap(True)
        name_label.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        name_label.set_ellipsize(True)  # Enable ellipsis for long text
        name_label.set_max_width_chars(25)  # Limit max width
        name_label.add_css_class("heading")
        info_box.append(name_label)

        # App URL
        url_label = Gtk.Label(label=self.webapp.app_url)
        url_label.set_halign(Gtk.Align.START)
        url_label.set_wrap(True)
        url_label.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        url_label.set_ellipsize(True)  # Enable ellipsis for long text
        url_label.set_max_width_chars(30)  # Limit max width
        url_label.add_css_class("caption")
        url_label.add_css_class("dim-label")
        info_box.append(url_label)
        box.append(info_box)

        # Actions box - make it more compact and styled as a pill
        actions_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        actions_box.set_halign(Gtk.Align.END)
        actions_box.add_css_class("linked")  # Makes buttons connect as a pill

        # Browser button
        browser = self.browser_collection.get_by_id(self.webapp.browser)
        browser_button = Gtk.Button()
        browser_button.set_tooltip_text(
            f"Browser: {browser.get_friendly_name() if browser else self.webapp.browser}"
        )
        browser_icon = Gtk.Image()
        set_image_from_browser_icon(browser_icon, self.webapp.browser, pixel_size=27)
        browser_button.set_child(browser_icon)
        browser_button.connect("clicked", self.on_browser_clicked)
        actions_box.append(browser_button)

        # Edit button
        edit_button = Gtk.Button()
        edit_button.set_tooltip_text("Edit WebApp")
        edit_icon = Gtk.Image()
        edit_icon.set_from_icon_name("document-edit-symbolic")
        edit_icon.set_pixel_size(20)
        edit_button.set_child(edit_icon)
        edit_button.connect("clicked", self.on_edit_clicked)
        actions_box.append(edit_button)

        # Delete button
        delete_button = Gtk.Button()
        delete_button.set_tooltip_text("Delete WebApp")
        delete_icon = Gtk.Image()
        delete_icon.set_from_icon_name("user-trash-symbolic")
        delete_icon.set_pixel_size(20)
        delete_button.set_child(delete_icon)
        delete_button.connect("clicked", self.on_delete_clicked)

        # Only add the destructive style to the icon, not the whole button
        # to maintain the unified pill appearance
        delete_icon.add_css_class("error")

        actions_box.append(delete_button)

        box.append(actions_box)

        self.set_child(box)

    def set_icon_from_path(self, icon_path):
        """
        Set the icon from a file path or icon name

        Parameters:
            icon_path (str): Path to the icon file or icon name
        """
        if not icon_path or icon_path == "/home/bruno/.local/share/icons/":
            self.icon.set_from_icon_name("webapp-generic")
            return

        try:
            if icon_path.startswith("/"):
                # Try to load from file path
                pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_size(icon_path, 48, 48)
                self.icon.set_from_pixbuf(pixbuf)
            else:
                # Try to load as icon name
                self.icon.set_from_icon_name(icon_path)
        except Exception as e:
            print(f"Error loading icon {icon_path}: {e}")
            self.icon.set_from_icon_name("webapp-generic")

    def get_browser_icon_name(self, browser_id):
        """
        Get icon name for a browser

        Parameters:
            browser_id (str): Browser identifier

        Returns:
            str: Icon name for the browser
        """
        # Icon mapping directly to files in the icons subfolder
        browser_icon_map = {
            "brave": "brave.svg",
            "brave-beta": "brave-beta.svg",
            "brave-nightly": "brave-nightly.svg",
            "firefox": "firefox.svg",
            "firefox-developer-edition": "firefox-developer-edition.svg",
            "firefox-nightly": "firefox-nightly.svg",
            "chromium": "chromium.svg",
            "chromium-dev": "chromium-dev.svg",
            "google-chrome-stable": "google-chrome-stable.svg",
            "google-chrome-beta": "google-chrome-beta.svg",
            "google-chrome-unstable": "google-chrome-unstable.svg",
            "vivaldi-stable": "vivaldi-stable.svg",
            "vivaldi-beta": "vivaldi-beta.svg",
            "vivaldi-snapshot": "vivaldi-snapshot.svg",
            "microsoft-edge-stable": "microsoft-edge-stable.svg",
            "microsoft-edge-beta": "microsoft-edge-beta.svg",
            "microsoft-edge-dev": "microsoft-edge-dev.svg",
            "librewolf": "librewolf.svg",
            "ungoogled-chromium": "ungoogled-chromium.svg",
            "flatpak-brave": "flatpak-brave.svg",
            "flatpak-chrome": "flatpak-chrome.svg",
            "flatpak-chromium": "flatpak-chromium.svg",
            "flatpak-edge": "flatpak-edge.svg",
            "flatpak-firefox": "flatpak-firefox.svg",
            "flatpak-librewolf": "flatpak-librewolf.svg",
            "flatpak-ungoogled-chromium": "flatpak-ungoogled-chromium.svg",
        }

        # Try to get the matching icon name
        icon_name = browser_icon_map.get(browser_id)

        # Default fallback
        if not icon_name:
            icon_name = "default-webapps.png"

        return icon_name

    def setup_compact_button(self, button):
        """
        Reduce padding and margins in a button to make it more compact

        Parameters:
            button (Gtk.Button): Button to make compact
        """
        # Set minimal padding
        button.set_margin_start(2)
        button.set_margin_end(2)
        button.set_margin_top(1)
        button.set_margin_bottom(1)

        # Add CSS for compact buttons
        button.add_css_class("flat")

        # Set a smaller minimum width
        button.set_size_request(28, 28)

        return button

    def on_edit_clicked(self, button):
        """Handle edit button click"""
        self.emit("edit-clicked", self.webapp)

    def on_browser_clicked(self, button):
        """Handle browser button click"""
        self.emit("browser-clicked", self.webapp)

    def on_delete_clicked(self, button):
        """Handle delete button click"""
        self.emit("delete-clicked", self.webapp)
