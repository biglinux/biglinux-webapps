"""
Utility functions for handling browser icons
"""

from __future__ import annotations

import logging
import os
from pathlib import Path

from gi.repository import Gdk, GLib, Gtk

logger = logging.getLogger(__name__)

# absolute path → icons dir relative to package root
BROWSER_ICONS_PATH = str(Path(__file__).resolve().parent.parent.parent / "icons")

_ICON_SIZES = (64, 48, 128, 32, 256, 512, 24, 22, 16)
_LOCAL_ICON_DIR = os.path.expanduser("~/.local/share/icons/")
_LOCAL_ICON_EXTS = (".svg", ".png", ".webp", ".xpm", ".ico")


def get_browser_icon_name(browser_id: str) -> str:
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


def set_image_from_browser_icon(
    image: Gtk.Image, browser_id: str, pixel_size: int = 48
) -> None:
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


def resolve_app_icon_path(icon_name: str, icon_theme: Gtk.IconTheme) -> str:
    """Resolve webapp icon name → filesystem path.

    Checks local user icons, then falls back to GTK icon theme
    with progressive name shortening (e.g. ``foo-bar`` → ``foo``).

    Returns:
        Resolved path string, or ``"Icon not found"`` on failure.
    """
    if not icon_name:
        return "Icon not found"

    if icon_name.startswith("/"):
        return icon_name

    # user-local icons (big-webapps copies custom icons here)
    local_path = _LOCAL_ICON_DIR + icon_name
    if os.path.exists(local_path):
        return local_path
    # big-webapps strips extension on create → try common extensions
    for ext in _LOCAL_ICON_EXTS:
        path_with_ext = local_path + ext
        if os.path.exists(path_with_ext):
            return path_with_ext

    # GTK4 icon theme lookup with progressive name shortening
    parts = icon_name.split("-")
    for end in range(len(parts), 0, -1):
        candidate = "-".join(parts[:end])
        for size in _ICON_SIZES:
            paintable = icon_theme.lookup_icon(
                candidate,
                None,
                size,
                1,
                Gtk.TextDirection.NONE,
                Gtk.IconLookupFlags(0),
            )
            if paintable:
                gfile = paintable.get_file()
                if gfile:
                    path = gfile.get_path()
                    if path:
                        return path

    return "Icon not found"


def enrich_webapps_with_icons(apps_data: list[dict]) -> list[dict]:
    """Add ``app_icon_url`` to each webapp dict using the running GTK display.

    Must be called after ``Gtk.init()`` / from a running GTK application.
    """
    display = Gdk.Display.get_default()
    if display is None:
        logger.warning("No display available — cannot resolve icon paths")
        return apps_data
    icon_theme = Gtk.IconTheme.get_for_display(display)
    for app in apps_data:
        icon_name = app.get("app_icon", "")
        app["app_icon_url"] = resolve_app_icon_path(icon_name, icon_theme)
    return apps_data
