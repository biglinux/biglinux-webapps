"""
Utility functions for handling browser icons
"""

from gi.repository import GLib
import os

# Folder with browser icons
BROWSER_ICONS_PATH = "icons"


def get_browser_icon_name(browser_id):
    """
    Get the icon name for a browser

    Parameters:
        browser_id (str): Browser identifier

    Returns:
        str: Icon filename for the browser
    """
    # Handle browser objects that might have been passed
    if hasattr(browser_id, "browser_id"):
        browser_id = browser_id.browser_id

    # Simply append .svg to the browser_id to get the icon filename
    return f"{browser_id}.svg" if browser_id else "default-webapps.png"


def set_image_from_browser_icon(image, browser_id, pixel_size=48):
    """
    Set a Gtk.Image from a browser icon

    Parameters:
        image (Gtk.Image): Image widget to set
        browser_id (str): Browser identifier or Browser object
        pixel_size (int): Size of the icon in pixels
    """
    image.set_pixel_size(pixel_size)

    # Handle browser objects that might have been passed
    if hasattr(browser_id, "browser_id"):
        browser_id = browser_id.browser_id

    # Get the icon filename
    icon_filename = get_browser_icon_name(browser_id)

    # Get icon path using the correct path
    icon_path = os.path.join(BROWSER_ICONS_PATH, icon_filename)

    # Try to load icon
    if os.path.exists(icon_path):
        try:
            image.set_from_file(icon_path)
        except GLib.Error:
            image.set_from_icon_name("browser-symbolic")
    else:
        image.set_from_icon_name("browser-symbolic")
