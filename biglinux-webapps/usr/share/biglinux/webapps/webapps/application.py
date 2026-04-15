"""
Application module containing the main WebAppsApplication class
"""

import gi
import os
import subprocess
from collections.abc import Callable

from webapps import APP_VERSION
from webapps.utils.translation import _

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, Gio, Gdk, GLib  # noqa: E402

from webapps.ui.main_window import MainWindow  # noqa: E402
from webapps.utils.webapp_service import WebAppService  # noqa: E402

import logging

logger = logging.getLogger(__name__)


class WebAppsApplication(Adw.Application):
    """Main application class for WebApps Manager"""

    def __init__(self) -> None:
        """Initialize the application"""
        super().__init__(
            application_id="br.com.biglinux.webapps",
            flags=Gio.ApplicationFlags.FLAGS_NONE,
        )

        # Set up application
        self.create_action("quit", self.quit, ["<primary>q"])
        self.create_action("about", self.on_about_action)
        self.create_action("refresh", self.on_refresh_action, ["<primary>r"])
        self.create_action("export", self.on_export_action, ["<primary>e"])
        self.create_action("import", self.on_import_action, ["<primary>i"])

        # centralized business logic
        self.service = WebAppService()

        # convenience aliases for UI code that reads collections
        self.webapp_collection = self.service.webapp_collection
        self.browser_collection = self.service.browser_collection
        self.command_executor = self.service.command_executor

        # Add the remove-all action
        remove_all_action = Gio.SimpleAction.new("remove-all", None)
        remove_all_action.connect("activate", self.on_remove_all)
        self.add_action(remove_all_action)

    def do_activate(self) -> None:
        """Called when the application is activated"""
        self.service.load_data()

        # Create and show the main window
        win = MainWindow(application=self)
        win.present()

        # Add folder browsing actions
        browse_apps_action = Gio.SimpleAction.new("browse-apps", None)
        browse_apps_action.connect("activate", self.on_browse_apps_activated)
        self.add_action(browse_apps_action)

        browse_profiles_action = Gio.SimpleAction.new("browse-profiles", None)
        browse_profiles_action.connect("activate", self.on_browse_profiles_activated)
        self.add_action(browse_profiles_action)

    def on_browse_apps_activated(
        self, _action: Gio.SimpleAction, _parameter: GLib.Variant | None
    ) -> None:
        """Open applications folder in the default file manager"""
        applications_path = os.path.expanduser("~/.local/share/applications")
        self._open_folder(applications_path)

    def on_browse_profiles_activated(
        self, _action: Gio.SimpleAction, _parameter: GLib.Variant | None
    ) -> None:
        """Open profiles folder in the default file manager"""
        profiles_path = os.path.expanduser("~/.bigwebapps")
        self._open_folder(profiles_path)

    def _open_folder(self, folder_path: str) -> None:
        """Open a folder in the default file manager, creating it if needed."""
        os.makedirs(folder_path, exist_ok=True)
        try:
            Gtk.show_uri(None, f"file://{folder_path}", Gdk.CURRENT_TIME)
        except Exception:
            subprocess.Popen(["xdg-open", folder_path])

    def create_action(
        self, name: str, callback: Callable, shortcuts: list[str] | None = None
    ) -> None:
        """Create a new application action with optional keyboard shortcuts"""
        action = Gio.SimpleAction.new(name, None)
        action.connect("activate", callback)
        self.add_action(action)

        if shortcuts:
            self.set_accels_for_action(f"app.{name}", shortcuts)

    def on_about_action(
        self, _widget: Gio.SimpleAction, _param: GLib.Variant | None
    ) -> None:
        """Show the about dialog"""
        about = Adw.AboutDialog(
            application_name="WebApps Manager",
            application_icon="big-webapps",
            developer_name="BigLinux Team",
            version=APP_VERSION,
            developers=["BigLinux Team"],
            copyright="© 2023 BigLinux Team",
            license_type=Gtk.License.GPL_3_0,
            website="https://www.biglinux.com.br",
            issue_url="https://github.com/biglinux/biglinux-webapps/issues",
        )
        about.present(self.props.active_window)

    def on_refresh_action(
        self, _widget: Gio.SimpleAction, _param: GLib.Variant | None
    ) -> None:
        """Refresh the data"""
        self.service.load_data()

        # Notify the main window to update UI
        active_window = self.props.active_window
        if active_window and hasattr(active_window, "refresh_ui"):
            active_window.refresh_ui()

    def on_export_action(
        self, _widget: Gio.SimpleAction, _param: GLib.Variant | None
    ) -> None:
        """Export webapps to a file"""
        # Use direct strings to avoid translation issues with file chooser
        dialog = Gtk.FileChooserNative.new(
            "Export WebApps",
            self.props.active_window,
            Gtk.FileChooserAction.SAVE,
            "Export",
            "Cancel",
        )
        dialog.set_current_name("biglinux-webapps-export.zip")

        # Add file filter for zip files
        filter_zip = Gtk.FileFilter()
        filter_zip.set_name("ZIP archives")
        filter_zip.add_pattern("*.zip")
        dialog.add_filter(filter_zip)

        # Handle response
        dialog.connect("response", self._handle_export_response)
        dialog.show()

    def _handle_export_response(
        self, dialog: Gtk.FileChooserNative, response: int
    ) -> None:
        """Handle export file chooser response"""
        if response != Gtk.ResponseType.ACCEPT:
            return
        file_path = dialog.get_file().get_path()
        ok, msg = self.service.export_webapps(file_path)
        if ok:
            self._show_notification(_("WebApps exported successfully"))
        elif msg == "no_webapps":
            self._show_error_dialog(
                _("No WebApps"), _("There are no WebApps to export.")
            )
        else:
            self._show_error_dialog("Export Failed", f"Failed to export WebApps: {msg}")

    def on_import_action(
        self, _widget: Gio.SimpleAction, _param: GLib.Variant | None
    ) -> None:
        """Import webapps from a file"""
        # Use direct strings to avoid translation issues with file chooser
        dialog = Gtk.FileChooserNative.new(
            "Import WebApps",
            self.props.active_window,
            Gtk.FileChooserAction.OPEN,
            "Import",
            "Cancel",
        )

        # Add file filter for zip files
        filter_zip = Gtk.FileFilter()
        filter_zip.set_name("ZIP archives")
        filter_zip.add_pattern("*.zip")
        dialog.add_filter(filter_zip)

        # Handle response
        dialog.connect("response", self._handle_import_response)
        dialog.show()

    def _handle_import_response(
        self, dialog: Gtk.FileChooserNative, response: int
    ) -> None:
        """Handle import file chooser response"""
        if response != Gtk.ResponseType.ACCEPT:
            return
        file_path = dialog.get_file().get_path()
        imported, duplicates, err = self.service.import_webapps(file_path)

        if err:
            msg_map = {
                "file_not_found": _("The selected file does not exist."),
                "invalid_zip": _("The selected file is not a valid ZIP archive."),
                "missing_webapps_json": "Invalid export file: missing webapps.json",
            }
            self._show_error_dialog(
                _("Error importing WebApps"),
                msg_map.get(err, err),
            )
            return

        active_window = self.props.active_window
        if active_window and hasattr(active_window, "refresh_ui"):
            active_window.refresh_ui()

        if duplicates > 0:
            self._show_notification(
                _("Imported {} WebApps successfully ({} duplicates skipped)").format(
                    imported, duplicates
                )
            )
        else:
            self._show_notification(
                _("Imported {} WebApps successfully").format(imported)
            )

    def _show_notification(self, message: str) -> None:
        """Show a notification message"""
        active_window = self.props.active_window
        if active_window and hasattr(active_window, "show_toast"):
            active_window.show_toast(message)

    def _show_error_dialog(self, title: str, message: str) -> None:
        """Show an error dialog"""
        dialog = Adw.MessageDialog.new(self.props.active_window, title, message)
        dialog.add_response("ok", _("OK"))
        dialog.present()

    def _show_confirmation_dialog(
        self, title: str, message: str, callback: Callable[[bool], None]
    ) -> None:
        """Show a confirmation dialog with Yes/No buttons"""
        dialog = Adw.MessageDialog.new(self.props.active_window, title, message)
        dialog.add_response("no", _("No"))
        dialog.add_response("yes", _("Yes"))
        dialog.set_default_response("no")
        dialog.set_response_appearance("yes", Adw.ResponseAppearance.SUGGESTED)

        dialog.connect("response", lambda _d, response: callback(response == "yes"))
        dialog.present()

    def quit(self, _widget: Gio.SimpleAction, _param: GLib.Variant | None) -> None:
        """Quit the application"""
        self.quit()

    def on_remove_all(
        self, _action: Gio.SimpleAction, _param: GLib.Variant | None
    ) -> None:
        """Remove all webapps after confirmation"""
        # Access the active window instead of using self.win
        active_window = self.props.active_window
        if active_window and hasattr(active_window, "on_remove_all_clicked"):
            active_window.on_remove_all_clicked()
