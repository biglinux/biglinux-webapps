#!/usr/bin/env python3.13
"""
BigLinux WebApps Manager
A GTK4 application for managing web applications in BigLinux.
"""

import sys
import os
import gettext
import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")

from gi.repository import GLib
from webapps.application import WebAppsApplication


def main():
    """Main function to start the application."""
    app = WebAppsApplication()

    # Set application ID for proper desktop integration
    app.set_application_id("br.com.biglinux.webapps")

    # Set program name for window manager class
    GLib.set_prgname("big-webapps-gui")

    # If your WebAppsApplication inherits from Adw.Application or Gtk.Application,
    # you can try this approach to set the icon:
    try:
        # For GTK4 applications
        app.set_icon_name("big-webapps")
    except (AttributeError, TypeError):
        # If the above doesn't work, we'll just rely on the desktop file
        # and the window manager to handle the icon
        pass

    # Set up internationalization at the module level
    locale_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), "locale")
    gettext.bindtextdomain("biglinux-webapps", locale_dir)
    gettext.textdomain("biglinux-webapps")

    return app.run(sys.argv)


if __name__ == "__main__":
    sys.exit(main())
