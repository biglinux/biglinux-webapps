"""
WebApp model module containing the WebApp and WebAppCollection classes
"""

from gi.repository import GObject
from urllib.parse import urlparse


class WebApp(GObject.GObject):
    """Class representing a web application"""

    __gtype_name__ = "WebApp"

    def __init__(self, app_data=None):
        """
        Initialize a WebApp instance

        Parameters:
            app_data (dict): Dictionary containing webapp data
        """
        super().__init__()

        # Default values
        self.browser = ""
        self.app_file = ""
        self.app_name = ""
        self.app_url = ""
        self.app_icon = ""
        self.app_profile = "Default"
        self.app_categories = "Webapps"
        self.app_icon_url = ""

        # Load data if provided
        if app_data:
            self.load_from_dict(app_data)

    def load_from_dict(self, app_data):
        """
        Load data from a dictionary

        Parameters:
            app_data (dict): Dictionary containing webapp data
        """
        self.browser = app_data.get("browser", "")
        self.app_file = app_data.get("app_file", "")
        self.app_name = app_data.get("app_name", "")
        self.app_url = app_data.get("app_url", "")
        self.app_icon = app_data.get("app_icon", "")
        self.app_profile = app_data.get("app_profile", "Default")
        self.app_categories = app_data.get("app_categories", "Webapps")
        self.app_icon_url = app_data.get("app_icon_url", "")

    def get_main_category(self):
        """
        Get the main category of the webapp

        Returns:
            str: Main category
        """
        if not self.app_categories:
            return "Webapps"

        categories = self.app_categories.split(";")
        return categories[0] if categories else "Webapps"

    def set_main_category(self, category):
        """
        Set the main category of the webapp

        Parameters:
            category (str): New main category
        """
        if not category:
            return

        categories = self.app_categories.split(";") if self.app_categories else []
        if categories and categories[0] == category:
            return

        # Filter out the new category if it already exists in other positions
        other_categories = [c for c in categories[1:] if c and c != category]
        self.app_categories = ";".join([category] + other_categories)

    def derive_profile_name(self):
        """
        Derive a profile name from the URL

        Returns:
            str: Derived profile name
        """
        try:
            url_obj = urlparse(self.app_url)
            hostname = url_obj.netloc
            # Remove dots from the hostname to create a profile name
            return hostname.replace(".", "")
        except Exception:
            # If URL parsing fails, attempt manual extraction
            import re

            match = re.search(r"^(?:https?://)?([^/]+)", self.app_url)
            if match and match.group(1):
                return match.group(1).replace(".", "")
            return "Default"


class WebAppCollection:
    """Collection of WebApp objects with filtering and categorization capabilities"""

    def __init__(self):
        """Initialize an empty WebApp collection"""
        self.webapps = []

    def load_from_json(self, json_data):
        """
        Load webapps from JSON data

        Parameters:
            json_data (list): List of webapp dictionaries
        """
        self.webapps = []

        if not json_data:
            return

        for app_data in json_data:
            webapp = WebApp(app_data)
            self.webapps.append(webapp)

    def get_all(self):
        """
        Get all webapps

        Returns:
            list: List of all WebApp objects
        """
        return self.webapps

    def filter_by_text(self, filter_text):
        """
        Filter webapps by text

        Parameters:
            filter_text (str): Text to filter by

        Returns:
            list: Filtered list of WebApp objects
        """
        if not filter_text:
            return self.webapps

        filter_text = filter_text.lower()

        return [
            app
            for app in self.webapps
            if (
                filter_text in app.app_name.lower()
                or filter_text in app.app_url.lower()
                or filter_text in app.app_file.lower()
            )
        ]

    def get_categorized(self, filter_text=None):
        """
        Get webapps categorized by their categories

        Parameters:
            filter_text (str, optional): Text to filter by

        Returns:
            dict: Dictionary of category -> list of WebApp objects
        """
        apps = self.filter_by_text(filter_text) if filter_text else self.webapps
        categorized = {}

        for app in apps:
            categories = app.app_categories.split(";")
            for category in categories:
                if not category:
                    continue

                if category not in categorized:
                    categorized[category] = []

                categorized[category].append(app)

        return categorized

    def add(self, webapp):
        """
        Add a webapp to the collection

        Parameters:
            webapp (WebApp): WebApp object to add
        """
        self.webapps.append(webapp)

    def remove(self, webapp):
        """
        Remove a webapp from the collection

        Parameters:
            webapp (WebApp): WebApp object to remove
        """
        self.webapps = [app for app in self.webapps if app.app_file != webapp.app_file]
