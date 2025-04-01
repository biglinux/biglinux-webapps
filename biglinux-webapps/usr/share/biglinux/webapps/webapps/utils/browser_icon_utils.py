"""
Utilities for handling browser icons
"""

import os
import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, GdkPixbuf

# Define the path to the icons directory relative to the main script location
SCRIPT_DIR = os.path.dirname(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
)
ICONS_DIR = os.path.join(SCRIPT_DIR, "icons")

# Icon mapping directly to files in the icons subfolder
BROWSER_ICON_MAP = {
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


def get_browser_icon_name(browser_id):
    """
    Get icon name for a browser

    Parameters:
        browser_id (str): Browser identifier

    Returns:
        str: Icon name for the browser
    """
    # Try to get the matching icon name
    icon_name = BROWSER_ICON_MAP.get(browser_id)

    # Default fallback
    if not icon_name:
        icon_name = "default-webapps.png"

    return icon_name


def get_browser_icon_path(browser_id):
    """
    Get the full path to the browser icon file

    Parameters:
        browser_id (str): Browser identifier

    Returns:
        str: Full path to the browser icon file
    """
    icon_name = get_browser_icon_name(browser_id)
    return os.path.join(ICONS_DIR, icon_name)


def set_image_from_browser_icon(image_widget, browser_id, pixel_size=32):
    """
    Set a Gtk.Image widget to display the browser icon

    Parameters:
        image_widget (Gtk.Image): The image widget to update
        browser_id (str): Browser identifier
        pixel_size (int): Desired pixel size for the icon
    """
    image_widget.set_pixel_size(pixel_size)

    # Get full path to the icon file
    icon_path = get_browser_icon_path(browser_id)
    if os.path.exists(icon_path):
        try:
            image_widget.set_from_file(icon_path)
        except Exception as e:
            print(f"Error loading icon from file {icon_path}: {e}")
            image_widget.set_from_icon_name("web-browser")
    else:
        # Fallback to system icon
        image_widget.set_from_icon_name("web-browser")
