"""
Command executor module for running shell commands
"""

import os
import json
import subprocess
from pathlib import Path


class CommandExecutor:
    """Class for executing shell commands and parsing their output"""

    def __init__(self):
        """Initialize the CommandExecutor"""
        # Store the base directory to run commands from
        self.base_dir = Path(os.path.dirname(os.path.realpath(__file__))).parent.parent

    def execute_command(self, command, input_data=None):
        """
        Execute a shell command and return its output

        Parameters:
            command (str): Command to execute
            input_data (str, optional): Input data to pass to the command

        Returns:
            str: Command output
        """
        try:
            # Change to the base directory
            original_dir = os.getcwd()
            os.chdir(self.base_dir)

            # Execute the command
            process = subprocess.Popen(
                command,
                shell=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                stdin=subprocess.PIPE if input_data else None,
                text=True,
            )

            # Provide input if needed
            stdout, stderr = process.communicate(input=input_data)

            # Change back to the original directory
            os.chdir(original_dir)

            # Check for errors
            if process.returncode != 0:
                print(f"Command failed: {command}")
                print(f"Error: {stderr}")
                return ""

            return stdout
        except Exception as e:
            print(f"Error executing command: {e}")
            return ""

    def execute_json_command(self, command, input_data=None):
        """
        Execute a shell command and parse its output as JSON

        Parameters:
            command (str): Command to execute
            input_data (str, optional): Input data to pass to the command

        Returns:
            dict or list: Parsed JSON data
        """
        output = self.execute_command(command, input_data)

        if not output:
            return []

        try:
            return json.loads(output)
        except json.JSONDecodeError as e:
            print(f"Error parsing JSON: {e}")
            print(f"Output: {output}")
            return []

    def create_webapp(self, webapp):
        """
        Create a new webapp

        Parameters:
            webapp (WebApp): WebApp object to create

        Returns:
            bool: True if successful, False otherwise
        """
        # Encode parameters properly
        browser = webapp.browser
        app_name = webapp.app_name
        app_url = webapp.app_url
        app_icon_url = webapp.app_icon_url
        app_categories = webapp.app_categories
        app_profile = webapp.app_profile

        # Build the command
        command = f"big-webapps create '{browser}' '{app_name}' '{app_url}' '{app_icon_url}' '{app_categories}' '{app_profile}'"

        # Execute the command
        output = self.execute_command(command)

        # Check if the command was successful
        return output != ""

    def update_webapp(self, webapp):
        """
        Update an existing webapp

        Parameters:
            webapp (WebApp): WebApp object to update

        Returns:
            bool: True if successful, False otherwise
        """
        # First remove the existing webapp
        remove_command = f"big-webapps remove '{webapp.app_file}'"
        self.execute_command(remove_command)

        # Then create a new one
        return self.create_webapp(webapp)

    def remove_webapp(self, webapp, delete_folder=False):
        """
        Remove a webapp

        Parameters:
            webapp (WebApp): WebApp object to remove
            delete_folder (bool): Whether to delete the configuration folder

        Returns:
            bool: True if successful, False otherwise
        """
        if delete_folder:
            command = f"big-webapps remove-with-folder '{webapp.app_file}' '{webapp.browser}' '{webapp.app_profile}'"
        else:
            command = f"big-webapps remove  '{webapp.app_file}' '{webapp.browser}' '{webapp.app_profile}'"

        output = self.execute_command(command)

        # Check if the command was successful
        return output != ""

    def select_icon(self):
        """
        Open the icon selector dialog

        Returns:
            str: Path to the selected icon
        """
        command = "./select_icon.sh"
        return self.execute_command(command).strip()

    def get_system_default_browser(self):
        """
        Detect the system's default browser

        Returns:
            str: The browser ID or None if detection failed
        """
        try:
            # Try using xdg-settings first
            result = self.execute_command("xdg-settings get default-web-browser")
            if result.strip():
                browser_desktop = result.strip()
                # Convert .desktop filename to browser ID
                if "brave" in browser_desktop.lower():
                    return "brave"
                elif "brave-beta" in browser_desktop.lower():
                    return "brave-beta"
                elif "brave-nightly" in browser_desktop.lower():
                    return "brave-nightly"
                elif "firefox" in browser_desktop.lower():
                    return "firefox"
                elif "chromium" in browser_desktop.lower():
                    return "chromium"
                elif (
                    "chrome" in browser_desktop.lower()
                    and "beta" in browser_desktop.lower()
                ):
                    return "google-chrome-beta"
                elif (
                    "chrome" in browser_desktop.lower()
                    and "unstable" in browser_desktop.lower()
                ):
                    return "google-chrome-unstable"
                elif "chrome" in browser_desktop.lower():
                    return "google-chrome-stable"
                elif "edge" in browser_desktop.lower():
                    return "microsoft-edge-stable"
                elif (
                    "vivaldi" in browser_desktop.lower()
                    and "beta" in browser_desktop.lower()
                ):
                    return "vivaldi-beta"
                elif (
                    "vivaldi" in browser_desktop.lower()
                    and "snapshot" in browser_desktop.lower()
                ):
                    return "vivaldi-snapshot"
                elif "vivaldi" in browser_desktop.lower():
                    return "vivaldi-stable"
                elif "librewolf" in browser_desktop.lower():
                    return "librewolf"
                elif "org.mozilla.firefox" in browser_desktop.lower():
                    return "flatpak-firefox"
                elif "org.chromium.Chromium" in browser_desktop.lower():
                    return "flatpak-chromium"
                elif "com.google.Chrome" in browser_desktop.lower():
                    return "flatpak-chrome"
                elif "com.google.ChromeDev" in browser_desktop.lower():
                    return "flatpak-chrome-unstable"
                elif "com.brave.Browser" in browser_desktop.lower():
                    return "flatpak-brave"
                elif "com.microsoft.Edge" in browser_desktop.lower():
                    return "flatpak-edge"
                elif "com.github.Eloston.UngoogledChromium" in browser_desktop.lower():
                    return "flatpak-ungoogled-chromium"
                elif "io.gitlab.librewolf" in browser_desktop.lower():
                    return "flatpak-librewolf"

            # Try xdg-mime as fallback
            result = self.execute_command(
                "xdg-mime query default x-scheme-handler/http"
            )
            if result.strip():
                browser_desktop = result.strip()
                # Convert .desktop filename to browser ID
                if "brave" in browser_desktop.lower():
                    return "brave"
                elif "brave-beta" in browser_desktop.lower():
                    return "brave-beta"
                elif "brave-nightly" in browser_desktop.lower():
                    return "brave-nightly"
                elif "firefox" in browser_desktop.lower():
                    return "firefox"
                elif "chromium" in browser_desktop.lower():
                    return "chromium"
                elif (
                    "chrome" in browser_desktop.lower()
                    and "beta" in browser_desktop.lower()
                ):
                    return "google-chrome-beta"
                elif (
                    "chrome" in browser_desktop.lower()
                    and "unstable" in browser_desktop.lower()
                ):
                    return "google-chrome-unstable"
                elif "chrome" in browser_desktop.lower():
                    return "google-chrome-stable"
                elif "edge" in browser_desktop.lower():
                    return "microsoft-edge-stable"
                elif (
                    "vivaldi" in browser_desktop.lower()
                    and "beta" in browser_desktop.lower()
                ):
                    return "vivaldi-beta"
                elif (
                    "vivaldi" in browser_desktop.lower()
                    and "snapshot" in browser_desktop.lower()
                ):
                    return "vivaldi-snapshot"
                elif "vivaldi" in browser_desktop.lower():
                    return "vivaldi-stable"
                elif "librewolf" in browser_desktop.lower():
                    return "librewolf"
                elif "org.mozilla.firefox" in browser_desktop.lower():
                    return "flatpak-firefox"
                elif "org.chromium.Chromium" in browser_desktop.lower():
                    return "flatpak-chromium"
                elif "com.google.Chrome" in browser_desktop.lower():
                    return "flatpak-chrome"
                elif "com.google.ChromeDev" in browser_desktop.lower():
                    return "flatpak-chrome-unstable"
                elif "com.brave.Browser" in browser_desktop.lower():
                    return "flatpak-brave"
                elif "com.microsoft.Edge" in browser_desktop.lower():
                    return "flatpak-edge"
                elif "com.github.Eloston.UngoogledChromium" in browser_desktop.lower():
                    return "flatpak-ungoogled-chromium"
                elif "io.gitlab.librewolf" in browser_desktop.lower():
                    return "flatpak-librewolf"
        except Exception as e:
            print(f"Error detecting system default browser: {e}")

        return None
