"""
Application module containing the main WebAppsApplication class
"""

import gi
import json
import os
import shutil
import tempfile
import zipfile
import time
import subprocess

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, Gio, Gdk

from webapps.models.webapp_model import WebAppCollection
from webapps.models.browser_model import BrowserCollection
from webapps.ui.main_window import MainWindow
from webapps.utils.command_executor import CommandExecutor
from webapps.utils.translation import _


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
        self.create_action("export", self.on_export_action, ["<primary>e"])
        self.create_action("import", self.on_import_action, ["<primary>i"])

        # Initialize collections
        self.webapp_collection = WebAppCollection()
        self.browser_collection = BrowserCollection()

        # Command executor for shell commands
        self.command_executor = CommandExecutor()

        # Register actions
        # (assuming actions like refresh, export, import are registered here)

        # Add the remove-all action
        remove_all_action = Gio.SimpleAction.new("remove-all", None)
        remove_all_action.connect("activate", self.on_remove_all)
        self.add_action(remove_all_action)

    def do_activate(self):
        """Called when the application is activated"""
        # Load data first
        self.load_data()

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

    def on_browse_apps_activated(self, action, parameter):
        """Open applications folder in the default file manager"""
        applications_path = os.path.expanduser("~/.local/share/applications")
        self._open_folder(applications_path)

    def on_browse_profiles_activated(self, action, parameter):
        """Open profiles folder in the default file manager"""
        profiles_path = os.path.expanduser("~/.bigwebapps")
        self._open_folder(profiles_path)

    def _open_folder(self, folder_path):
        """Open a folder in the default file manager"""
        if os.path.exists(folder_path):
            try:
                # Use Gtk.show_uri to open the folder in the default file manager
                Gtk.show_uri(None, f"file://{folder_path}", Gdk.CURRENT_TIME)
            except Exception as e:
                print(f"Error opening folder: {e}")
                # Fallback to xdg-open if Gtk.show_uri fails
                subprocess.Popen(["xdg-open", folder_path])
        else:
            print(f"Folder does not exist: {folder_path}")
            # Create the folder if it doesn't exist and then open it
            os.makedirs(folder_path, exist_ok=True)
            self._open_folder(folder_path)

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
            application_icon="big-webapps",
            developer_name="BigLinux Team",
            version="3.0.0",
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

    def on_export_action(self, widget, param):
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

    def _handle_export_response(self, dialog, response):
        """Handle export file chooser response"""
        if response == Gtk.ResponseType.ACCEPT:
            file_path = dialog.get_file().get_path()

            try:
                # Get all webapps
                webapps = self.webapp_collection.get_all()

                if not webapps:
                    self._show_error_dialog(
                        _("No WebApps"), _("There are no WebApps to export.")
                    )
                    return

                # Create temporary directory for export
                with tempfile.TemporaryDirectory() as temp_dir:
                    # Create webapps.json file
                    webapps_data = []
                    icons_dir = os.path.join(temp_dir, "icons")
                    themes_dir = os.path.join(temp_dir, "themes")
                    os.makedirs(icons_dir, exist_ok=True)
                    os.makedirs(themes_dir, exist_ok=True)

                    # Process each webapp
                    for webapp in webapps:
                        webapp_dict = {
                            "browser": webapp.browser,
                            "app_name": webapp.app_name,
                            "app_url": webapp.app_url,
                            "app_icon": webapp.app_icon,
                            "app_profile": webapp.app_profile,
                            "app_categories": webapp.app_categories,
                        }

                        # Handle icon if it's in home folder
                        if webapp.app_icon_url and webapp.app_icon_url.startswith(
                            os.path.expanduser("~")
                        ):
                            # Extract icon filename
                            icon_filename = os.path.basename(webapp.app_icon_url)
                            # Copy icon to temp directory
                            icon_dest = os.path.join(icons_dir, icon_filename)
                            try:
                                shutil.copy2(webapp.app_icon_url, icon_dest)
                                # Store relative path to icon
                                webapp_dict["app_icon_url"] = f"icons/{icon_filename}"
                            except (IOError, PermissionError) as e:
                                print(f"Failed to copy icon {webapp.app_icon_url}: {e}")
                                webapp_dict["app_icon_url"] = ""
                        else:
                            # Just store the original URL if not in home folder
                            webapp_dict["app_icon_url"] = webapp.app_icon_url

                        # Also handle any custom theme icons (.theme files)
                        if webapp.app_icon and not webapp.app_icon.startswith((
                            "/",
                            "~",
                        )):
                            # Check if there might be a theme file associated with this icon
                            theme_file = os.path.expanduser(
                                f"~/.local/share/icons/{webapp.app_icon}.theme"
                            )
                            if os.path.exists(theme_file):
                                theme_name = f"{webapp.app_icon}.theme"
                                theme_dest = os.path.join(themes_dir, theme_name)
                                try:
                                    shutil.copy2(theme_file, theme_dest)
                                    # No need to store this reference as it's implied by the icon name
                                except (IOError, PermissionError) as e:
                                    print(
                                        f"Failed to copy theme file {theme_file}: {e}"
                                    )

                        webapps_data.append(webapp_dict)

                    # Write webapps data to JSON file
                    with open(os.path.join(temp_dir, "webapps.json"), "w") as f:
                        json.dump(webapps_data, f, indent=2)

                    # Create ZIP archive
                    with zipfile.ZipFile(file_path, "w", zipfile.ZIP_DEFLATED) as zipf:
                        for root, _, files in os.walk(temp_dir):
                            for file in files:
                                file_path_full = os.path.join(root, file)
                                zipf.write(
                                    file_path_full,
                                    os.path.relpath(file_path_full, temp_dir),
                                )

                # Show success message using string directly
                self._show_notification("WebApps exported successfully")

            except Exception as e:
                print(f"Error exporting webapps: {e}")
                # Use direct strings to avoid translation issues
                self._show_error_dialog(
                    "Export Failed", f"Failed to export WebApps: {str(e)}"
                )

    def on_import_action(self, widget, param):
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

    def _handle_import_response(self, dialog, response):
        """Handle import file chooser response"""
        if response == Gtk.ResponseType.ACCEPT:
            file_path = dialog.get_file().get_path()

            try:
                # Validate the file exists
                if not os.path.exists(file_path):
                    self._show_error_dialog(
                        _("File Not Found"), _("The selected file does not exist.")
                    )
                    return

                # Validate it's a zip file
                if not zipfile.is_zipfile(file_path):
                    self._show_error_dialog(
                        _("Invalid File"),
                        _("The selected file is not a valid ZIP archive."),
                    )
                    return

                # Create temporary directory for import
                with tempfile.TemporaryDirectory() as temp_dir:
                    # Extract ZIP archive
                    with zipfile.ZipFile(file_path, "r") as zipf:
                        zipf.extractall(temp_dir)

                    # Read webapps data from JSON file
                    webapps_file = os.path.join(temp_dir, "webapps.json")
                    if not os.path.exists(webapps_file):
                        raise FileNotFoundError(
                            "Invalid export file: missing webapps.json"
                        )

                    with open(webapps_file, "r") as f:
                        webapps_data = json.load(f)

                    # Create local icons directory if it doesn't exist
                    local_icons_dir = os.path.expanduser("~/.local/share/icons")
                    os.makedirs(local_icons_dir, exist_ok=True)

                    # Get existing webapps for duplicate checking
                    existing_webapps = self.webapp_collection.get_all()

                    # Create sets of (name, url) tuples for faster lookup
                    existing_webapp_keys = {
                        (webapp.app_name, webapp.app_url) for webapp in existing_webapps
                    }

                    import_count = 0
                    duplicate_count = 0

                    for webapp_dict in webapps_data:
                        # Check if this webapp already exists (same name and URL)
                        webapp_key = (
                            webapp_dict.get("app_name", ""),
                            webapp_dict.get("app_url", ""),
                        )

                        if webapp_key in existing_webapp_keys:
                            duplicate_count += 1
                            continue  # Skip this webapp

                        # Handle icon if it was included in the export
                        if webapp_dict.get("app_icon_url", "").startswith("icons/"):
                            icon_filename = os.path.basename(
                                webapp_dict["app_icon_url"]
                            )
                            export_icon_path = os.path.join(
                                temp_dir, webapp_dict["app_icon_url"]
                            )
                            local_icon_path = os.path.join(
                                local_icons_dir, icon_filename
                            )

                            try:
                                if os.path.exists(export_icon_path):
                                    # Copy icon to local icons directory
                                    shutil.copy2(export_icon_path, local_icon_path)
                                    # Update icon URL to point to local copy
                                    webapp_dict["app_icon_url"] = local_icon_path
                            except (IOError, PermissionError) as e:
                                print(f"Failed to copy icon {export_icon_path}: {e}")
                                webapp_dict["app_icon_url"] = ""

                        # Generate a unique app_file name
                        import_timestamp = int(time.time()) + import_count
                        webapp_dict["app_file"] = f"{import_timestamp}-import"
                        import_count += 1

                        # Create the webapp
                        from webapps.models.webapp_model import WebApp

                        webapp = WebApp(webapp_dict)
                        self.command_executor.create_webapp(webapp)

                    # Reload data
                    self.load_data()

                    # Update UI
                    active_window = self.props.active_window
                    if active_window and hasattr(active_window, "refresh_ui"):
                        active_window.refresh_ui()

                    # Show success message with information about duplicates
                    if duplicate_count > 0:
                        self._show_notification(
                            _(
                                "Imported {} WebApps successfully ({} duplicates skipped)"
                            ).format(import_count, duplicate_count)
                        )
                    else:
                        self._show_notification(
                            _("Imported {} WebApps successfully").format(import_count)
                        )

            except Exception as e:
                print(f"Error importing webapps: {e}")
                self._show_error_dialog(_("Error importing WebApps"), str(e))

    def _show_notification(self, message):
        """Show a notification message"""
        active_window = self.props.active_window
        if active_window and hasattr(active_window, "show_toast"):
            active_window.show_toast(message)

    def _show_error_dialog(self, title, message):
        """Show an error dialog"""
        dialog = Adw.MessageDialog.new(self.props.active_window, title, message)
        dialog.add_response("ok", _("OK"))
        dialog.present()

    def _show_confirmation_dialog(self, title, message, callback):
        """Show a confirmation dialog with Yes/No buttons"""
        dialog = Adw.MessageDialog.new(self.props.active_window, title, message)
        dialog.add_response("no", _("No"))
        dialog.add_response("yes", _("Yes"))
        dialog.set_default_response("no")
        dialog.set_response_appearance("yes", Adw.ResponseAppearance.SUGGESTED)

        dialog.connect("response", lambda d, response: callback(response == "yes"))
        dialog.present()

    def quit(self, widget, _):
        """Quit the application"""
        self.quit()

    def on_remove_all(self, action, param):
        """Remove all webapps after confirmation"""
        # Access the active window instead of using self.win
        active_window = self.props.active_window
        if active_window and hasattr(active_window, "on_remove_all_clicked"):
            active_window.on_remove_all_clicked()
