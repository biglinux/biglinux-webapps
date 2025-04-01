"""
Application module containing the main WebAppsApplication class
"""

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, Gio

from webapps.models.webapp_model import WebAppCollection
from webapps.models.browser_model import BrowserCollection
from webapps.ui.main_window import MainWindow
from webapps.utils.command_executor import CommandExecutor


class WebAppsApplication(Adw.Application):
    """Main application class for WebApps Manager"""

    def __init__(self):
        """Initialize the application"""
        super().__init__(
            application_id="org.biglinux.webapps", flags=Gio.ApplicationFlags.FLAGS_NONE
        )

        # Set up application
        self.create_action("quit", self.quit, ["<primary>q"])
        self.create_action("about", self.on_about_action)
        self.create_action("refresh", self.on_refresh_action, ["<primary>r"])

        # Initialize collections
        self.webapp_collection = WebAppCollection()
        self.browser_collection = BrowserCollection()

        # Command executor for shell commands
        self.command_executor = CommandExecutor()

        # Build UI on startup
        self.connect("activate", self.on_activate)

    def on_activate(self, app):
        """Create the main window when the application is activated"""
        # Load data first
        self.load_data()

        # Create and show the main window
        win = MainWindow(application=self)
        win.present()

    def create_action(self, name, callback, shortcuts=None):
        """Create a new application action with optional keyboard shortcuts"""
        action = Gio.SimpleAction.new(name, None)
        action.connect("activate", callback)
        self.add_action(action)

        if shortcuts:
            self.set_accels_for_action(f"app.{name}", shortcuts)

    def on_about_action(self, widget, _):
        """Show the about dialog"""
        about = Adw.AboutWindow(
            transient_for=self.props.active_window,
            application_name="WebApps Manager",
            application_icon="webapps",
            developer_name="BigLinux Team",
            version="1.0.0",
            developers=["BigLinux Team"],
            copyright="© 2023 BigLinux Team",
            license_type=Gtk.License.GPL_3_0,
            website="https://www.biglinux.com.br",
            issue_url="https://github.com/biglinux/biglinux-webapps/issues",
        )
        about.present()

    def on_refresh_action(self, widget, _):
        """Refresh the data"""
        self.load_data()

        # Notify the main window to update UI
        active_window = self.props.active_window
        if active_window and hasattr(active_window, "refresh_ui"):
            active_window.refresh_ui()

    def load_data(self):
        """Load webapp and browser data from the system"""
        # Load webapp data
        webapps_data = self.command_executor.execute_json_command("./get_json.sh")
        self.webapp_collection.load_from_json(webapps_data)

        # Load browser data
        browsers_data = self.command_executor.execute_json_command(
            "./check_browser.sh --list-json"
        )
        self.browser_collection.load_from_json(browsers_data)

        # Get default browser
        default_browser = self.command_executor.execute_command(
            "./check_browser.sh --default"
        ).strip()
        self.browser_collection.set_default(default_browser)

    def quit(self, widget, _):
        """Quit the application"""
        self.quit()
