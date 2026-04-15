"""
Template gallery dialog — browse and apply curated webapp templates.
"""

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw, GObject

from webapps.templates.registry import WebAppTemplate, build_default_registry
from webapps.utils.translation import _

import logging

logger = logging.getLogger(__name__)


class TemplateGallery(Adw.Window):
    """Grid gallery of curated webapp templates."""

    __gsignals__ = {
        "template-selected": (
            GObject.SignalFlags.RUN_FIRST,
            None,
            (str,),  # template_id
        ),
    }

    def __init__(self, parent: Gtk.Window) -> None:
        super().__init__(
            title=_("Templates"),
            transient_for=parent,
            modal=True,
            destroy_with_parent=True,
            default_width=650,
            default_height=500,
        )

        self.registry = build_default_registry()
        self._build_ui()

    # ── UI ──────────────────────────────────────────────────────────

    def _build_ui(self) -> None:
        content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)

        # header
        header = Adw.HeaderBar()
        header.set_title_widget(Gtk.Label(label=_("Choose a Template")))
        header.add_css_class("flat")
        content.append(header)

        # search bar
        self.search_entry = Gtk.SearchEntry()
        self.search_entry.set_placeholder_text(_("Search templates..."))
        self.search_entry.set_margin_start(12)
        self.search_entry.set_margin_end(12)
        self.search_entry.set_margin_top(6)
        self.search_entry.set_margin_bottom(6)
        self.search_entry.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [_("Search templates")],
        )
        self.search_entry.connect("search-changed", self._on_search_changed)
        content.append(self.search_entry)

        # scrolled area
        scrolled = Gtk.ScrolledWindow()
        scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scrolled.set_vexpand(True)

        self.main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        scrolled.set_child(self.main_box)
        content.append(scrolled)

        self._populate_all()
        self.set_content(content)

    def _populate_all(self) -> None:
        """Populate gallery with all templates grouped by category."""
        self._clear_main_box()
        categories = self.registry.get_categories()
        for cat in sorted(categories):
            templates = self.registry.get_by_category(cat)
            if templates:
                self._add_category_section(cat, templates)

    def _populate_search(self, query: str) -> None:
        """Populate gallery with search results."""
        self._clear_main_box()
        results = self.registry.search(query)
        if results:
            self._add_category_section(_("Search Results"), results)
        else:
            label = Gtk.Label(label=_("No templates found"))
            label.set_margin_top(24)
            label.add_css_class("dim-label")
            self.main_box.append(label)

    def _add_category_section(
        self, title: str, templates: list[WebAppTemplate]
    ) -> None:
        """Add a category header + FlowBox of template cards."""
        # category heading
        heading = Gtk.Label(label=title, xalign=0)
        heading.add_css_class("heading")
        heading.set_margin_start(16)
        heading.set_margin_top(12)
        heading.set_margin_bottom(4)
        self.main_box.append(heading)

        flow = Gtk.FlowBox()
        flow.set_selection_mode(Gtk.SelectionMode.NONE)
        flow.set_homogeneous(True)
        flow.set_max_children_per_line(5)
        flow.set_min_children_per_line(2)
        flow.set_margin_start(12)
        flow.set_margin_end(12)
        flow.set_margin_bottom(8)
        flow.set_row_spacing(8)
        flow.set_column_spacing(8)

        for tmpl in templates:
            card = self._make_card(tmpl)
            flow.append(card)

        self.main_box.append(flow)

    def _make_card(self, tmpl: WebAppTemplate) -> Gtk.Button:
        """Build a clickable card for a template."""
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        box.set_margin_top(8)
        box.set_margin_bottom(8)
        box.set_margin_start(4)
        box.set_margin_end(4)
        box.set_halign(Gtk.Align.CENTER)

        icon = Gtk.Image.new_from_icon_name(tmpl.icon or "application-x-executable")
        icon.set_pixel_size(48)
        box.append(icon)

        label = Gtk.Label(label=tmpl.name)
        label.set_ellipsize(3)  # PANGO_ELLIPSIZE_END
        label.set_max_width_chars(14)
        label.add_css_class("caption")
        box.append(label)

        btn = Gtk.Button()
        btn.set_child(box)
        btn.add_css_class("flat")
        btn.set_tooltip_text(tmpl.comment or tmpl.name)
        btn.connect("clicked", self._on_card_clicked, tmpl.template_id)

        btn.update_property(
            [Gtk.AccessibleProperty.LABEL],
            [tmpl.name],
        )

        return btn

    def _clear_main_box(self) -> None:
        while child := self.main_box.get_first_child():
            self.main_box.remove(child)

    # ── Signals ─────────────────────────────────────────────────────

    def _on_search_changed(self, entry: Gtk.SearchEntry) -> None:
        query = entry.get_text().strip()
        if query:
            self._populate_search(query)
        else:
            self._populate_all()

    def _on_card_clicked(self, _btn: Gtk.Button, template_id: str) -> None:
        self.emit("template-selected", template_id)
        self.close()
