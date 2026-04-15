"""
Business logic layer between UI and shell commands.
All webapp CRUD, export/import, and data loading lives here.
"""

import json
import logging
import os
import shutil
import tempfile
import time
import zipfile

from webapps.models.webapp_model import WebApp, WebAppCollection
from webapps.models.browser_model import BrowserCollection
from webapps.utils.browser_icon_utils import enrich_webapps_with_icons
from webapps.utils.command_executor import CommandExecutor

logger = logging.getLogger(__name__)


class WebAppService:
    """Centralized business logic for webapp operations."""

    def __init__(self) -> None:
        self.command_executor = CommandExecutor()
        self.webapp_collection = WebAppCollection()
        self.browser_collection = BrowserCollection()

    # ── data loading ─────────────────────────────────────────────

    def load_data(self) -> None:
        """Reload webapp + browser data from system."""
        webapps_data = self.command_executor.execute_json_command([
            "big-webapps",
            "json",
        ])
        enrich_webapps_with_icons(webapps_data)
        self.webapp_collection.load_from_json(webapps_data)

        browsers_data = self.command_executor.execute_json_command([
            "./check_browser.sh",
            "--list-json",
        ])
        self.browser_collection.load_from_json(browsers_data)

        default_browser = self.command_executor.execute_command([
            "./check_browser.sh",
            "--default",
        ]).strip()
        self.browser_collection.set_default(default_browser)

    # ── CRUD ─────────────────────────────────────────────────────

    def create_webapp(self, webapp: WebApp) -> bool:
        """Create webapp on disk and reload collection."""
        ok = self.command_executor.create_webapp(webapp)
        if ok:
            self.load_data()
        return ok

    def update_webapp(self, webapp: WebApp) -> bool:
        """Update webapp on disk and reload collection."""
        ok = self.command_executor.update_webapp(webapp)
        if ok:
            self.load_data()
        return ok

    def delete_webapp(self, webapp: WebApp, delete_folder: bool = False) -> bool:
        """Delete a single webapp and remove from collection."""
        ok = self.command_executor.remove_webapp(webapp, delete_folder)
        if ok:
            self.webapp_collection.remove(webapp)
        return ok

    def delete_all_webapps(self) -> bool:
        """Delete every webapp. Returns True if all succeeded."""
        webapps = self.webapp_collection.get_all()
        ok = all(
            self.command_executor.remove_webapp(wa, False) for wa in webapps
        )
        if ok:
            self.load_data()
        return ok

    # ── lookup ───────────────────────────────────────────────────

    def find_webapp(
        self, url: str, name: str, app_file: str = ""
    ) -> WebApp | None:
        """Find webapp by app_file (stable ID), then URL+name, fallback URL only."""
        if app_file:
            for wa in self.webapp_collection.get_all():
                if wa.app_file == app_file:
                    return wa
        for wa in self.webapp_collection.get_all():
            if wa.app_url == url and wa.app_name == name:
                return wa
        for wa in self.webapp_collection.get_all():
            if wa.app_url == url:
                return wa
        return None

    def get_system_default_browser(self) -> str | None:
        """Detect system default browser ID."""
        return self.command_executor.get_system_default_browser()

    # ── export ───────────────────────────────────────────────────

    def export_webapps(self, file_path: str) -> tuple[bool, str]:
        """Export all webapps to a ZIP file.

        Returns:
            (success, message)
        """
        webapps = self.webapp_collection.get_all()
        if not webapps:
            return False, "no_webapps"

        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                icons_dir = os.path.join(temp_dir, "icons")
                themes_dir = os.path.join(temp_dir, "themes")
                os.makedirs(icons_dir, exist_ok=True)
                os.makedirs(themes_dir, exist_ok=True)

                webapps_data = [
                    self._serialize_webapp_for_export(wa, icons_dir, themes_dir)
                    for wa in webapps
                ]

                with open(os.path.join(temp_dir, "webapps.json"), "w") as f:
                    json.dump(webapps_data, f, indent=2)

                with zipfile.ZipFile(file_path, "w", zipfile.ZIP_DEFLATED) as zipf:
                    for root, _dirs, files in os.walk(temp_dir):
                        for fname in files:
                            full = os.path.join(root, fname)
                            zipf.write(full, os.path.relpath(full, temp_dir))

            return True, "ok"
        except Exception as e:
            logger.error("Export failed: %s", e)
            return False, str(e)

    def _serialize_webapp_for_export(
        self, webapp: WebApp, icons_dir: str, themes_dir: str
    ) -> dict:
        """Serialize one webapp for ZIP export, copying icons/themes."""
        data: dict = {
            "browser": webapp.browser,
            "app_name": webapp.app_name,
            "app_url": webapp.app_url,
            "app_icon": webapp.app_icon,
            "app_profile": webapp.app_profile,
            "app_categories": webapp.app_categories,
        }

        home = os.path.expanduser("~")
        if webapp.app_icon_url and webapp.app_icon_url.startswith(home):
            icon_filename = os.path.basename(webapp.app_icon_url)
            try:
                shutil.copy2(webapp.app_icon_url, os.path.join(icons_dir, icon_filename))
                data["app_icon_url"] = f"icons/{icon_filename}"
            except (IOError, PermissionError) as e:
                logger.error("Failed to copy icon %s: %s", webapp.app_icon_url, e)
                data["app_icon_url"] = ""
        else:
            data["app_icon_url"] = webapp.app_icon_url

        if webapp.app_icon and not webapp.app_icon.startswith(("/", "~")):
            theme_file = os.path.expanduser(
                f"~/.local/share/icons/{webapp.app_icon}.theme"
            )
            if os.path.exists(theme_file):
                try:
                    shutil.copy2(
                        theme_file,
                        os.path.join(themes_dir, f"{webapp.app_icon}.theme"),
                    )
                except (IOError, PermissionError) as e:
                    logger.error("Failed to copy theme %s: %s", theme_file, e)

        return data

    # ── import ───────────────────────────────────────────────────

    def import_webapps(self, file_path: str) -> tuple[int, int, str]:
        """Import webapps from a ZIP file.

        Returns:
            (imported_count, duplicate_count, error_message)
            error_message is empty on success.
        """
        if not os.path.exists(file_path):
            return 0, 0, "file_not_found"

        if not zipfile.is_zipfile(file_path):
            return 0, 0, "invalid_zip"

        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                with zipfile.ZipFile(file_path, "r") as zipf:
                    # path traversal protection
                    for member in zipf.namelist():
                        real = os.path.realpath(os.path.join(temp_dir, member))
                        if not real.startswith(os.path.realpath(temp_dir) + os.sep):
                            return 0, 0, f"unsafe_path:{member}"
                    zipf.extractall(temp_dir)

                webapps_file = os.path.join(temp_dir, "webapps.json")
                if not os.path.exists(webapps_file):
                    return 0, 0, "missing_webapps_json"

                with open(webapps_file, "r") as f:
                    webapps_data = json.load(f)

                local_icons_dir = os.path.expanduser("~/.local/share/icons")
                os.makedirs(local_icons_dir, exist_ok=True)

                existing_keys = {
                    (wa.app_name, wa.app_url)
                    for wa in self.webapp_collection.get_all()
                }

                imported = 0
                duplicates = 0
                for wd in webapps_data:
                    key = (wd.get("app_name", ""), wd.get("app_url", ""))
                    if key in existing_keys:
                        duplicates += 1
                        continue
                    self._import_single_webapp(wd, temp_dir, local_icons_dir, imported)
                    imported += 1

                self.load_data()
                return imported, duplicates, ""

        except Exception as e:
            logger.error("Import failed: %s", e)
            return 0, 0, str(e)

    def _import_single_webapp(
        self, webapp_dict: dict, temp_dir: str, local_icons_dir: str, seq: int
    ) -> None:
        """Import one webapp dict, copying its icon."""
        if webapp_dict.get("app_icon_url", "").startswith("icons/"):
            icon_filename = os.path.basename(webapp_dict["app_icon_url"])
            export_icon = os.path.join(temp_dir, webapp_dict["app_icon_url"])
            local_icon = os.path.join(local_icons_dir, icon_filename)
            try:
                if os.path.exists(export_icon):
                    shutil.copy2(export_icon, local_icon)
                    webapp_dict["app_icon_url"] = local_icon
            except (IOError, PermissionError) as e:
                logger.error("Failed to copy icon %s: %s", export_icon, e)
                webapp_dict["app_icon_url"] = ""

        webapp_dict["app_file"] = f"{int(time.time()) + seq}-import"
        webapp = WebApp(webapp_dict)
        self.command_executor.create_webapp(webapp)
