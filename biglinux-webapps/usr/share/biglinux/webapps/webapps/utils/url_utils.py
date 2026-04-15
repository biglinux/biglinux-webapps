"""
URL utilities for fetching website information
"""

import gi
import requests
import threading
import re
import tempfile
import os
import io
from urllib.parse import urlparse, urljoin
from html.parser import HTMLParser
from PIL import Image  # Add Pillow import
from collections.abc import Callable

gi.require_version("Gtk", "4.0")
from gi.repository import GLib

import logging

logger = logging.getLogger(__name__)


class WebsiteMetadataParser(HTMLParser):
    """Parser for extracting title and icons from HTML"""

    def __init__(self) -> None:
        super().__init__()
        self.title: str | None = None
        self.icons: list[str] = []
        self.og_title: str | None = None
        self.twitter_title: str | None = None
        self.og_image: str | None = None
        self.twitter_image: str | None = None
        self._in_title: bool = False

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attrs_dict = dict(attrs)

        if tag == "title":
            self._in_title = True

        elif tag == "meta":
            # Handle Open Graph and Twitter metadata
            property_attr = attrs_dict.get("property", "")
            name_attr = attrs_dict.get("name", "")
            content = attrs_dict.get("content")

            if content:
                if property_attr == "og:title":
                    self.og_title = content
                elif name_attr == "twitter:title":
                    self.twitter_title = content
                elif property_attr == "og:image":
                    self.og_image = content
                elif name_attr == "twitter:image":
                    self.twitter_image = content

        elif tag == "link":
            rel = attrs_dict.get("rel", "").lower()
            href = attrs_dict.get("href")

            if href:
                # Match common icon rel types
                if any(
                    x in rel
                    for x in ["icon", "shortcut icon", "apple-touch-icon", "mask-icon"]
                ):
                    self.icons.append(href)

    def handle_endtag(self, tag: str) -> None:
        if tag == "title":
            self._in_title = False

    def handle_data(self, data: str) -> None:
        if self._in_title:
            if self.title is None:
                self.title = data
            else:
                self.title += data

    def get_best_title(self) -> str | None:
        if self.title:
            return self.title.strip()
        if self.og_title:
            return self.og_title.strip()
        if self.twitter_title:
            return self.twitter_title.strip()
        return None

    def get_all_icons(self) -> list[str]:
        all_icons = self.icons.copy()
        if self.og_image:
            all_icons.append(self.og_image)
        if self.twitter_image:
            all_icons.append(self.twitter_image)
        return all_icons


class WebsiteInfoFetcher:
    """Class for fetching website information like title and favicons"""

    def __init__(self) -> None:
        """Initialize the fetcher"""
        self.user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36"

    def fetch_info(self, url: str, callback: Callable[[str, list[str]], None]) -> None:
        """
        Fetch website information (title and icons) in a background thread

        Parameters:
            url (str): Website URL
            callback (callable): Callback function to call with the results
        """
        if not url.startswith(("http://", "https://")):
            url = "https://" + url

        thread = threading.Thread(target=self._fetch_info_thread, args=(url, callback))
        thread.daemon = True
        thread.start()

    def _resolve_title(self, parser: WebsiteMetadataParser, url: str) -> str:
        """Pick best title from parsed metadata, fallback to domain."""
        title = parser.get_best_title()
        if title:
            return re.sub(r"\s+", " ", title)
        domain = urlparse(url).netloc.replace("www.", "")
        return domain.capitalize()

    def _collect_icon_urls(
        self, raw_icons: list[str], url: str, session: requests.Session
    ) -> list[str]:
        """Normalize raw icon hrefs → absolute URLs, append favicon.ico if found."""
        icons: list[str] = []
        for href in raw_icons:
            if href:
                if not href.startswith(("http://", "https://")):
                    href = urljoin(url, href)
                if href not in icons:
                    icons.append(href)

        parsed = urlparse(url)
        favicon_url = f"{parsed.scheme}://{parsed.netloc}/favicon.ico"
        if favicon_url not in icons:
            try:
                head = session.head(favicon_url, timeout=5)
                if head.status_code == 200:
                    icons.insert(0, favicon_url)
            except Exception:
                pass
        return icons

    def _fetch_info_thread(
        self, url: str, callback: Callable[[str, list[str]], None]
    ) -> None:
        """
        Fetch website information in a background thread

        Parameters:
            url (str): Website URL
            callback (callable): Callback function to call with the results
        """
        try:
            session = requests.Session()
            session.headers.update({"User-Agent": self.user_agent})

            response = session.get(url, timeout=10)
            response.raise_for_status()

            parser = WebsiteMetadataParser()
            parser.feed(response.text)

            title = self._resolve_title(parser, url)
            icons = self._collect_icon_urls(parser.get_all_icons(), url, session)

            icon_paths = []
            for icon_url in icons:
                try:
                    icon_path = self._download_icon(icon_url, session)
                    if icon_path:
                        icon_paths.append(icon_path)
                except Exception as e:
                    logger.error("Error downloading icon %s: %s", icon_url, e)

            GLib.idle_add(callback, title, icon_paths)

        except Exception as e:
            logger.error("Error fetching website info: %s", e)
            GLib.idle_add(callback, "", [])

    def _download_icon(self, icon_url: str, session: requests.Session) -> str | None:
        """
        Download an icon to a temporary file and convert non-PNG/SVG to PNG

        Parameters:
            icon_url (str): Icon URL
            session (requests.Session): Requests session

        Returns:
            str: Path to the downloaded icon file, or None if download failed
        """
        try:
            response = session.get(icon_url, timeout=10)
            response.raise_for_status()

            # Check content type to determine if conversion is needed
            content_type = response.headers.get("Content-Type", "").lower()

            # If it's already PNG or SVG, save directly
            if "image/png" in content_type or "image/svg+xml" in content_type:
                fd, path = tempfile.mkstemp(
                    prefix="webapp_icon_",
                    suffix=".png" if "image/png" in content_type else ".svg",
                )
                with os.fdopen(fd, "wb") as f:
                    f.write(response.content)
                return path

            # For other formats, convert to PNG for better compatibility
            try:
                # Create a temporary file with the original content
                img = Image.open(io.BytesIO(response.content))

                # Convert to RGB if needed (handles RGBA, CMYK, etc.)
                if img.mode != "RGB":
                    img = img.convert("RGB")

                # Create a temporary file for the PNG
                fd, path = tempfile.mkstemp(prefix="webapp_icon_", suffix=".png")

                # Save as PNG
                img.save(os.fdopen(fd, "wb"), format="PNG")
                return path

            except Exception as e:
                logger.error("Error converting image: %s", e)
                # Fallback: save original format if conversion fails
                fd, path = tempfile.mkstemp(prefix="webapp_icon_", suffix=".png")
                with os.fdopen(fd, "wb") as f:
                    f.write(response.content)
                return path

        except Exception as e:
            logger.error("Error downloading icon %s: %s", icon_url, e)
            return None
