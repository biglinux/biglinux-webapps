import gi
import sys
import os
import json

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
from gi.repository import Gtk, Gdk


def get_icon_path(icon_name, icon_theme):
    """
    Attempts to find the icon path based on the icon name, suitable for SVG, PNG, and other formats.
    If the icon is not found, it tries to progressively remove parts of the name separated by '-'.
    """
    if icon_name.startswith("/"):
        return icon_name  # Returns the absolute path if specified

    # Check user-local icons first (big-webapps copies custom icons here)
    local_icon_dir = os.path.expanduser("~/.local/share/icons/")
    local_icon_path = local_icon_dir + icon_name
    if os.path.exists(local_icon_path):
        return local_icon_path
    # big-webapps strips extension on create → try common extensions
    for ext in (".svg", ".png", ".webp", ".xpm", ".ico"):
        path_with_ext = local_icon_path + ext
        if os.path.exists(path_with_ext):
            return path_with_ext

    # Fall back to icon theme lookup (GTK4 API)
    parts = icon_name.split("-")
    for end in range(len(parts), 0, -1):
        modified_icon_name = "-".join(parts[:end])
        for size in [64, 48, 128, 32, 256, 512, 24, 22, 16]:
            paintable = icon_theme.lookup_icon(
                modified_icon_name,
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


def get_app_info_from_json(json_file):
    with open(json_file, "r") as file:
        apps_data = json.load(file)

    # GTK4 requires init + display for icon theme
    Gtk.init()
    display = Gdk.Display.get_default()
    icon_theme = Gtk.IconTheme.get_for_display(display)
    for app in apps_data:
        icon_name = app.get("app_icon", "")
        icon_path = get_icon_path(icon_name, icon_theme)
        app["app_icon_url"] = icon_path

    return json.dumps(apps_data, indent=4, ensure_ascii=False)


if __name__ == "__main__":
    if len(sys.argv) > 1:
        json_file = sys.argv[1]
        print(get_app_info_from_json(json_file))
    else:
        print("Please provide the JSON file path as an argument.")
