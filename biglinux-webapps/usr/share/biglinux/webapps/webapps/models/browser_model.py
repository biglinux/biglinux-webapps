"""
Browser model module containing the Browser and BrowserCollection classes
"""

from gi.repository import GObject
from webapps.utils.browser_icon_utils import get_browser_icon_name


class Browser(GObject.GObject):
    """Class representing a browser"""

    __gtype_name__ = "Browser"

    def __init__(self, browser_data=None):
        """
        Initialize a Browser instance

        Parameters:
            browser_data (dict): Dictionary containing browser data
        """
        super().__init__()

        # Default values
        self.browser_id = ""
        self.is_default = False

        # Load data if provided
        if browser_data:
            self.load_from_dict(browser_data)

    def load_from_dict(self, browser_data):
        """
        Load data from a dictionary

        Parameters:
            browser_data (dict): Dictionary containing browser data
        """
        self.browser_id = browser_data.get("browser", "")
        self.is_default = browser_data.get("is_default", False)

    def get_friendly_name(self):
        """
        Get a user-friendly name for the browser

        Returns:
            str: User-friendly browser name
        """
        browser_name_map = {
            "brave": "Brave",
            "firefox": "Firefox",
            "chromium": "Chromium",
            "google-chrome-stable": "Chrome",
            "vivaldi-stable": "Vivaldi",
            "flatpak-brave": "Brave (Flatpak)",
            "flatpak-chrome": "Chrome (Flatpak)",
            "flatpak-chrome-unstable": "Chrome Unstable (Flatpak)",
            "flatpak-chromium": "Chromium (Flatpak)",
            "flatpak-edge": "Edge (Flatpak)",
            "microsoft-edge-stable": "Edge",
            "librewolf": "Librewolf",
            "flatpak-ungoogled-chromium": "Chromium (Flatpak)",
            "flatpak-firefox": "Firefox (Flatpak)",
            "flatpak-librewolf": "Librewolf (Flatpak)",
            "brave-beta": "Brave Beta",
            "brave-nightly": "Brave Nightly",
            "google-chrome-beta": "Chrome Beta",
            "google-chrome-unstable": "Chrome Unstable",
            "vivaldi-beta": "Vivaldi Beta",
            "vivaldi-snapshot": "Vivaldi Snapshot",
        }

        return browser_name_map.get(self.browser_id, self.browser_id)

    def get_browser_icon_name(self):
        """
        Get icon name for the browser

        Returns:
            str: Icon name for the browser
        """
        return get_browser_icon_name(self.browser_id)

    def is_firefox_based(self):
        """
        Check if the browser is Firefox-based

        Returns:
            bool: True if the browser is Firefox-based, False otherwise
        """
        return (
            "firefox" in self.browser_id.lower()
            or "librewolf" in self.browser_id.lower()
        )


class BrowserCollection:
    """Collection of Browser objects"""

    def __init__(self):
        """Initialize an empty Browser collection"""
        self.browsers = []
        self.default_browser_id = None

    def load_from_json(self, json_data):
        """
        Load browsers from JSON data

        Parameters:
            json_data (list): List of browser dictionaries
        """
        self.browsers = []

        if not json_data:
            return

        for browser_data in json_data:
            browser = Browser(browser_data)
            self.browsers.append(browser)

    def get_all(self):
        """
        Get all browsers

        Returns:
            list: List of all Browser objects
        """
        return self.browsers

    def set_default(self, browser_id):
        """
        Set the default browser

        Parameters:
            browser_id (str): ID of the default browser
        """
        self.default_browser_id = browser_id

        for browser in self.browsers:
            browser.is_default = browser.browser_id == browser_id

    def get_default(self):
        """
        Get the default browser

        Returns:
            Browser or None: Default Browser object if found, None otherwise
        """
        for browser in self.browsers:
            if browser.is_default or browser.browser_id == self.default_browser_id:
                return browser

        # If no default is set but we have browsers, return the first one
        return self.browsers[0] if self.browsers else None

    def get_by_id(self, browser_id):
        """
        Get a browser by its ID

        Parameters:
            browser_id (str): Browser ID to search for

        Returns:
            Browser or None: Browser object if found, None otherwise
        """
        for browser in self.browsers:
            if browser.browser_id == browser_id:
                return browser
        return None
