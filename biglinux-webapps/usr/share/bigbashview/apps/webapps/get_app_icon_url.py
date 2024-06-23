import gi
import sys
import os
import json
gi.require_version('Gtk', '3.0')
from gi.repository import Gio, Gtk

def get_icon_path(icon_name, icon_theme):
    """
    Attempts to find the icon path based on the icon name, suitable for SVG, PNG, and other formats.
    If the icon is not found, it tries to progressively remove parts of the name separated by '-'.
    """
    if icon_name.startswith('/'):
        return icon_name  # Returns the absolute path if specified
    
    parts = icon_name.split('-')
    for end in range(len(parts), 0, -1):
        modified_icon_name = '-'.join(parts[:end])
        for size in [64, 48, 128, 32, 256, 512, 24, 22, 16]:
            icon_info = icon_theme.lookup_icon(modified_icon_name, size, Gtk.IconLookupFlags.USE_BUILTIN)
            if icon_info:
                return icon_info.get_filename()
    
    # Check in $HOME/.local/share/icons directly
    local_icon_path = os.path.expanduser(f"~/.local/share/icons/{icon_name}")
    if os.path.exists(local_icon_path):
        return local_icon_path

    return "Icon not found"

def get_app_info_from_json(json_file):
    with open(json_file, 'r') as file:
        apps_data = json.load(file)
    
    icon_theme = Gtk.IconTheme.get_default()
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
