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
from bs4 import BeautifulSoup
from PIL import Image  # Add Pillow import

gi.require_version("Gtk", "4.0")
from gi.repository import GLib


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

            # Parse the HTML
            soup = BeautifulSoup(response.text, "html.parser")

            # Get the title
            title = self._extract_title(soup, url)

            # Get favicons
            icons = self._extract_favicons(soup, url, session)

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

    def _extract_title(self, soup, url):
        """
        Extract title from HTML

        Parameters:
            soup (BeautifulSoup): Parsed HTML
            url (str): Website URL

        Returns:
            str: Website title
        """
        # Try to get the title tag
        title_tag = soup.find("title")
        if title_tag and title_tag.text:
            # Clean up the title
            title = title_tag.text.strip()
            title = re.sub(r"\s+", " ", title)
            return title

        # Alternative: Try Open Graph title
        og_title = soup.find("meta", property="og:title")
        if og_title and og_title.get("content"):
            return og_title.get("content").strip()

        # Alternative: Try Twitter title
        twitter_title = soup.find("meta", attrs={"name": "twitter:title"})
        if twitter_title and twitter_title.get("content"):
            return twitter_title.get("content").strip()

        # Fallback: Use domain name
        domain = urlparse(url).netloc.replace("www.", "")
        return domain.capitalize()

    def _extract_favicons(self, soup, url, session):
        """
        Extract favicon URLs from HTML

        Parameters:
            soup (BeautifulSoup): Parsed HTML
            url (str): Website URL
            session (requests.Session): Requests session

        Returns:
            list: List of favicon URLs
        """
        base_url = url
        parsed_url = urlparse(url)
        domain = f"{parsed_url.scheme}://{parsed_url.netloc}"

        icons = []

        # Look for favicon in common locations first
        favicon_url = urljoin(domain, "/favicon.ico")
        try:
            response = session.head(favicon_url, timeout=5)
            if response.status_code == 200:
                icons.append(favicon_url)
        except Exception:
            pass

        # Find link rel="icon" and rel="shortcut icon" tags
        icon_links = soup.find_all(
            "link",
            rel=re.compile(r"(shortcut icon|icon|apple-touch-icon|mask-icon)", re.I),
        )

        for link in icon_links:
            href = link.get("href")
            if href:
                # Make sure the URL is absolute
                if not href.startswith(("http://", "https://")):
                    href = urljoin(base_url, href)
                icons.append(href)

        # Find Apple touch icons
        apple_icons = soup.find_all("link", rel=re.compile(r"apple-touch-icon", re.I))
        for link in apple_icons:
            href = link.get("href")
            if href:
                if not href.startswith(("http://", "https://")):
                    href = urljoin(base_url, href)
                icons.append(href)

        # Find Open Graph images
        og_image = soup.find("meta", property="og:image")
        if og_image and og_image.get("content"):
            icons.append(og_image.get("content"))

        # Find Twitter images
        twitter_image = soup.find("meta", attrs={"name": "twitter:image"})
        if twitter_image and twitter_image.get("content"):
            icons.append(twitter_image.get("content"))

        # Remove duplicates while maintaining order
        unique_icons = []
        for icon in icons:
            if icon not in unique_icons:
                unique_icons.append(icon)

        return unique_icons

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
