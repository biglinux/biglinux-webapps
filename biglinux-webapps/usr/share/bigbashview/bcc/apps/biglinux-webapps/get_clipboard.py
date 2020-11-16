import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk, Gdk

def clipboard_gtk():
    clipboard = Gtk.Clipboard.get(Gdk.SELECTION_CLIPBOARD)
    text = clipboard.wait_for_text()
    if text is not None:
        return text

print(clipboard_gtk())