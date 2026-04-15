"""
FaviconPicker — FlowBox widget for selecting website favicons.
Emits ``icon-selected(path)`` when user picks an icon.
"""

import gi
import logging

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, GObject, GdkPixbuf

from webapps.utils.translation import _

logger = logging.getLogger(__name__)


class FaviconPicker(Gtk.Box):
    """FlowBox-based favicon selector.

    Signals:
        icon-selected(str): emitted with absolute path of chosen icon.
    """

    __gsignals__ = {
        "icon-selected": (GObject.SignalFlags.RUN_FIRST, None, (str,)),
    }

    _CSS = b"""
        .favicon-selected {
            border: 2px solid @accent_color;
            border-radius: 6px;
            padding: 2px;
        }
    """

    _css_registered = False

    def __init__(self) -> None:
        super().__init__(orientation=Gtk.Orientation.VERTICAL)

        self._selected_child: Gtk.FlowBoxChild | None = None
        self.connect("map", self._on_map)

        self._flowbox = Gtk.FlowBox()
        self._flowbox.set_selection_mode(Gtk.SelectionMode.SINGLE)
        self._flowbox.set_max_children_per_line(5)
        self._flowbox.set_homogeneous(True)
        self._flowbox.set_margin_top(8)
        self._flowbox.set_margin_bottom(8)
        self._flowbox.connect("child-activated", self._on_child_activated)
        self.append(self._flowbox)

    # -- public API ----------------------------------------------------------

    def load_icons(self, icon_paths: list[str]) -> None:
        """Replace current icons with *icon_paths* and show the picker."""
        self._clear()
        for idx, path in enumerate(icon_paths, 1):
            container = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
            container.favicon_url = path
            container.update_property(
                [Gtk.AccessibleProperty.LABEL],
                [_("Icon {0} of {1}").format(idx, len(icon_paths))],
            )

            image = Gtk.Image()
            image.set_pixel_size(48)

            try:
                pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_size(path, 48, 48)
                image.set_from_pixbuf(pixbuf)
            except Exception as e:
                logger.error("Error loading favicon %s: %s", path, e)
                image.set_from_icon_name("image-missing")

            container.append(image)
            self._flowbox.append(container)

    # -- private -------------------------------------------------------------

    def _clear(self) -> None:
        self._selected_child = None
        while self._flowbox.get_first_child():
            self._flowbox.remove(self._flowbox.get_first_child())

    def _on_map(self, _widget: Gtk.Widget) -> None:
        """Register selection CSS once the widget is mapped to a display."""
        if FaviconPicker._css_registered:
            return
        provider = Gtk.CssProvider()
        provider.load_from_data(self._CSS)
        Gtk.StyleContext.add_provider_for_display(
            self.get_display(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        FaviconPicker._css_registered = True

    def _on_child_activated(
        self, _flowbox: Gtk.FlowBox, child: Gtk.FlowBoxChild
    ) -> None:
        # remove previous highlight
        if self._selected_child is not None:
            self._selected_child.remove_css_class("favicon-selected")
        child.add_css_class("favicon-selected")
        self._selected_child = child

        container = child.get_child()
        path = getattr(container, "favicon_url", None)
        if path:
            self.emit("icon-selected", path)
