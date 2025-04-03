#!/usr/bin/env python3.13
"""
BigLinux WebApps Manager
A GTK4 application for managing web applications in BigLinux.
"""

import sys
import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")

from gi.repository import GLib

# Import from our utility to ensure proper initialization
from webapps.application import WebAppsApplication


def main():
    """Main function to start the application."""
    app = WebAppsApplication()

    # Set application ID for proper desktop integration
    app.set_application_id("br.com.biglinux.webapps")

    # Set program name for window manager class
    GLib.set_prgname("big-webapps-gui")

    try:
        # For GTK4 applications
        app.set_icon_name("big-webapps")
    except (AttributeError, TypeError):
        pass

    return app.run(sys.argv)


if __name__ == "__main__":
    sys.exit(main())
