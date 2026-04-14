"""
Command executor module for running shell commands
"""

import logging
import json
import re
import shutil
import subprocess
from pathlib import Path

from webapps.utils.browser_registry import match_desktop_to_browser

logger = logging.getLogger(__name__)


class CommandExecutor:
    """Class for executing shell commands and parsing their output"""

    def __init__(self):
        """Initialize the CommandExecutor"""
        # scripts (get_json.sh, check_browser.sh) live one level above the Python package
        self.base_dir = Path(__file__).resolve().parent.parent.parent

    def execute_command(self, argv: list[str], input_data: str | None = None) -> str:
        """
        Execute a command as an argument list (no shell).

        Parameters:
            argv: Command and arguments as a list
            input_data: Optional stdin data

        Returns:
            Command stdout
        """
        try:
            result = subprocess.run(
                argv,
                cwd=self.base_dir,
                capture_output=True,
                text=True,
                input=input_data,
            )
            if result.returncode != 0:
                logger.error("Command failed: %s\n%s", argv, result.stderr)
                return ""
            return result.stdout
        except Exception as e:
            logger.error("Error executing command %s: %s", argv, e)
            return ""

    def execute_json_command(
        self, argv: list[str], input_data: str | None = None
    ) -> list | dict:
        """
        Execute a command and parse its output as JSON.

        Parameters:
            argv: Command and arguments as a list
            input_data: Optional stdin data

        Returns:
            Parsed JSON data
        """
        output = self.execute_command(argv, input_data)

        if not output:
            return []

        try:
            return json.loads(output)
        except json.JSONDecodeError as e:
            logger.error("Error parsing JSON: %s\nOutput: %s", e, output)
            return []

    def create_webapp(self, webapp) -> bool:
        """
        Create a new webapp.

        Parameters:
            webapp: WebApp object to create

        Returns:
            True if successful
        """
        browser = "__viewer__" if webapp.app_mode == "app" else webapp.browser
        argv = [
            "big-webapps",
            "create",
            browser,
            webapp.app_name,
            webapp.app_url,
            webapp.app_icon_url,
            webapp.app_categories,
            webapp.app_profile,
        ]
        logger.debug("create_webapp argv: %s", argv)
        logger.debug(
            "create_webapp icon_url=%r icon=%r", webapp.app_icon_url, webapp.app_icon
        )
        output = self.execute_command(argv)
        return output != ""

    def update_webapp(self, webapp) -> bool:
        """
        Update an existing webapp (remove then create).

        Parameters:
            webapp: WebApp object to update

        Returns:
            True if successful
        """
        self.execute_command(["big-webapps", "remove", webapp.app_file])
        return self.create_webapp(webapp)

    def remove_webapp(self, webapp, delete_folder: bool = False) -> bool:
        """
        Remove a webapp.

        Parameters:
            webapp: WebApp object to remove
            delete_folder: Whether to delete the configuration folder

        Returns:
            True if successful
        """
        if delete_folder:
            argv = [
                "big-webapps",
                "remove-with-folder",
                webapp.app_file,
                webapp.browser,
                webapp.app_profile,
            ]
        else:
            argv = [
                "big-webapps",
                "remove",
                webapp.app_file,
                webapp.browser,
                webapp.app_profile,
            ]
        output = self.execute_command(argv)
        # cleanup viewer config/data if this was an app-mode webapp
        if webapp.app_mode == "app":
            self._cleanup_viewer_data(webapp.app_url)
        return output != ""

    def _cleanup_viewer_data(self, url: str) -> None:
        """Remove viewer config and persistent data for a given URL."""
        app_id = re.sub(r"https?://", "", url)
        app_id = app_id.replace("/", "_")
        app_id = re.sub(r"[^a-zA-Z0-9_-]", "", app_id)
        if not app_id:
            return
        home = Path.home()
        config_json = home / ".config" / "biglinux-webapps" / f"{app_id}.json"
        data_dir = home / ".local" / "share" / "biglinux-webapps" / app_id
        if config_json.exists():
            config_json.unlink()
            logger.debug("Removed viewer config: %s", config_json)
        if data_dir.exists():
            shutil.rmtree(data_dir, ignore_errors=True)
            logger.debug("Removed viewer data: %s", data_dir)

    def select_icon(self) -> str:
        """
        Open the icon selector dialog.

        Returns:
            Path to the selected icon
        """
        result = self.execute_command(["./select_icon.sh"]).strip()
        return result

    def get_system_default_browser(self) -> str | None:
        """
        Detect the system's default browser.

        Returns:
            Browser ID or None if detection failed
        """
        try:
            # xdg-settings first
            result = self.execute_command([
                "xdg-settings",
                "get",
                "default-web-browser",
            ])
            if result.strip():
                match = match_desktop_to_browser(result.strip())
                if match:
                    return match

            # xdg-mime fallback
            result = self.execute_command([
                "xdg-mime",
                "query",
                "default",
                "x-scheme-handler/http",
            ])
            if result.strip():
                match = match_desktop_to_browser(result.strip())
                if match:
                    return match
        except Exception as e:
            logger.error("Error detecting system default browser: %s", e)

        return None
