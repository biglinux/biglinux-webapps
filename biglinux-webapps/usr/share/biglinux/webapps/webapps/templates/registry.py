"""Template registry — discovery, validation, lookup for webapp presets."""

from __future__ import annotations

import logging
from dataclasses import dataclass, field

logger = logging.getLogger(__name__)


@dataclass(frozen=True, slots=True)
class WebAppTemplate:
    """Immutable preset for a known web service."""

    template_id: str
    name: str
    url: str
    icon: str
    category: str
    mime_types: tuple[str, ...] = ()
    url_schemes: tuple[str, ...] = ()
    features: tuple[str, ...] = ()
    profile: str = ""
    comment: str = ""
    generic_name: str = ""
    keywords: tuple[str, ...] = ()
    file_handler: str = ""  # "upload" | "url" | ""


@dataclass
class TemplateRegistry:
    """Central store for all webapp templates with lookup helpers."""

    _templates: dict[str, WebAppTemplate] = field(default_factory=dict)
    _by_category: dict[str, list[WebAppTemplate]] = field(default_factory=dict)

    def register(self, tpl: WebAppTemplate) -> None:
        """Add template to registry."""
        self._templates[tpl.template_id] = tpl
        self._by_category.setdefault(tpl.category, []).append(tpl)

    def register_many(self, templates: list[WebAppTemplate]) -> None:
        for t in templates:
            self.register(t)

    def get(self, template_id: str) -> WebAppTemplate | None:
        return self._templates.get(template_id)

    def get_all(self) -> list[WebAppTemplate]:
        return list(self._templates.values())

    def get_by_category(self, category: str) -> list[WebAppTemplate]:
        return list(self._by_category.get(category, []))

    def get_categories(self) -> list[str]:
        return sorted(self._by_category.keys())

    def match_url(self, url: str) -> WebAppTemplate | None:
        """Find template matching a URL (best-effort domain match)."""
        url_lower = url.lower()
        for tpl in self._templates.values():
            # extract domain from template URL for matching
            tpl_domain = _extract_domain(tpl.url)
            if tpl_domain and tpl_domain in url_lower:
                return tpl
        return None

    def search(self, query: str) -> list[WebAppTemplate]:
        """Search templates by name, category, or keywords."""
        q = query.lower()
        results = []
        for tpl in self._templates.values():
            if (
                q in tpl.name.lower()
                or q in tpl.category.lower()
                or any(q in kw.lower() for kw in tpl.keywords)
            ):
                results.append(tpl)
        return results


def _extract_domain(url: str) -> str:
    """Extract base domain from URL for matching."""
    from urllib.parse import urlparse

    try:
        parsed = urlparse(url)
        host = parsed.hostname or ""
        # strip www.
        if host.startswith("www."):
            host = host[4:]
        return host
    except Exception:
        return ""


def build_default_registry() -> TemplateRegistry:
    """Build registry with all bundled templates."""
    from webapps.templates.office365 import OFFICE365_TEMPLATES
    from webapps.templates.google import GOOGLE_TEMPLATES
    from webapps.templates.communication import COMMUNICATION_TEMPLATES
    from webapps.templates.media import MEDIA_TEMPLATES
    from webapps.templates.productivity import PRODUCTIVITY_TEMPLATES

    registry = TemplateRegistry()
    for group in (
        OFFICE365_TEMPLATES,
        GOOGLE_TEMPLATES,
        COMMUNICATION_TEMPLATES,
        MEDIA_TEMPLATES,
        PRODUCTIVITY_TEMPLATES,
    ):
        registry.register_many(group)

    logger.debug("Template registry loaded: %d templates", len(registry._templates))
    return registry
