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

gi.require_version("Gtk", "4.0")
from gi.repository import GLib


class WebsiteMetadataParser(HTMLParser):
    """Parser for extracting title and icons from HTML"""

    def __init__(self):
        super().__init__()
        self.title = None
        self.icons = []
        self.og_title = None
        self.twitter_title = None
        self.og_image = None
        self.twitter_image = None
        self._in_title = False

    def handle_starttag(self, tag, attrs):
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

    def handle_endtag(self, tag):
        if tag == "title":
            self._in_title = False

    def handle_data(self, data):
        if self._in_title:
            if self.title is None:
                self.title = data
            else:
                self.title += data

    def get_best_title(self):
        if self.title:
            return self.title.strip()
        if self.og_title:
            return self.og_title.strip()
        if self.twitter_title:
            return self.twitter_title.strip()
        return None

    def get_all_icons(self):
        all_icons = self.icons.copy()
        if self.og_image:
            all_icons.append(self.og_image)
        if self.twitter_image:
            all_icons.append(self.twitter_image)
        return all_icons


class WebsiteInfoFetcher:
    """Class for fetching website information like title and favicons"""

    def __init__(self):
        """Initialize the fetcher"""
        self.user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36"

    def fetch_info(self, url, callback):
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

    def _fetch_info_thread(self, url, callback):
        """
        Fetch website information in a background thread

        Parameters:
            url (str): Website URL
            callback (callable): Callback function to call with the results
        """
        try:
            # Create a session with our user agent
            session = requests.Session()
            session.headers.update({"User-Agent": self.user_agent})

            # Fetch the page
            response = session.get(url, timeout=10)
            response.raise_for_status()

            # Parse the HTML with our custom parser
            parser = WebsiteMetadataParser()
            parser.feed(response.text)

            # Get the title
            title = parser.get_best_title()
            if title:
                # Clean up the title
                title = re.sub(r"\s+", " ", title)
            else:
                # Fallback: Use domain name
                domain = urlparse(url).netloc.replace("www.", "")
                title = domain.capitalize()

            # Get favicons
            raw_icons = parser.get_all_icons()
            icons = []

            # Normalize icon URLs
            base_url = url
            for icon_href in raw_icons:
                if icon_href:
                    if not icon_href.startswith(("http://", "https://")):
                        icon_href = urljoin(base_url, icon_href)
                    if icon_href not in icons:
                        icons.append(icon_href)

            # Look for favicon in common locations (root)
            parsed_url = urlparse(url)
            domain_url = f"{parsed_url.scheme}://{parsed_url.netloc}"
            favicon_url = urljoin(domain_url, "/favicon.ico")

            # Check if we already have this specific favicon
            if favicon_url not in icons:
                try:
                    head_response = session.head(favicon_url, timeout=5)
                    if head_response.status_code == 200:
                        icons.insert(
                            0, favicon_url
                        )  # Prioritize default favicon if found
                except Exception:
                    pass

            # Save icons to temporary files
            icon_paths = []
            for icon_url in icons:
                try:
                    icon_path = self._download_icon(icon_url, session)
                    if icon_path:
                        icon_paths.append(icon_path)
                except Exception as e:
                    print(f"Error downloading icon {icon_url}: {e}")

            # Call the callback in the main thread
            GLib.idle_add(callback, title, icon_paths)

        except Exception as e:
            print(f"Error fetching website info: {e}")
            GLib.idle_add(callback, "", [])

    def _download_icon(self, icon_url, session):
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
                print(f"Error converting image: {e}")
                # Fallback: save original format if conversion fails
                fd, path = tempfile.mkstemp(prefix="webapp_icon_", suffix=".png")
                with os.fdopen(fd, "wb") as f:
                    f.write(response.content)
                return path

        except Exception as e:
            print(f"Error downloading icon {icon_url}: {e}")
            return None
