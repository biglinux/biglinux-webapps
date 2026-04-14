"""
Central browser registry — single source of truth for browser ID mappings.

Shell scripts (check_browser.sh, big-webapps-exec) maintain their own
layer-specific data (file paths, flatpak exec commands) since they need
different fields and cannot import Python modules.
"""

# browser ID → user-facing display name
BROWSER_DISPLAY_NAMES: dict[str, str] = {
    "brave": "Brave",
    "brave-beta": "Brave Beta",
    "brave-nightly": "Brave Nightly",
    "chromium": "Chromium",
    "firefox": "Firefox",
    "google-chrome-beta": "Chrome Beta",
    "google-chrome-stable": "Chrome",
    "google-chrome-unstable": "Chrome Unstable",
    "librewolf": "Librewolf",
    "microsoft-edge-stable": "Edge",
    "vivaldi-beta": "Vivaldi Beta",
    "vivaldi-snapshot": "Vivaldi Snapshot",
    "vivaldi-stable": "Vivaldi",
    "flatpak-brave": "Brave (Flatpak)",
    "flatpak-chrome": "Chrome (Flatpak)",
    "flatpak-chrome-unstable": "Chrome Unstable (Flatpak)",
    "flatpak-chromium": "Chromium (Flatpak)",
    "flatpak-edge": "Edge (Flatpak)",
    "flatpak-firefox": "Firefox (Flatpak)",
    "flatpak-librewolf": "Librewolf (Flatpak)",
    "flatpak-ungoogled-chromium": "Chromium (Flatpak)",
}

# desktop file pattern (regex) → browser ID, ordered most-specific first
DESKTOP_PATTERN_MAP: list[tuple[str, str]] = [
    ("brave-beta", "brave-beta"),
    ("brave-nightly", "brave-nightly"),
    ("brave", "brave"),
    ("firefox", "firefox"),
    ("chromium", "chromium"),
    ("chrome.*beta", "google-chrome-beta"),
    ("chrome.*unstable", "google-chrome-unstable"),
    ("chrome", "google-chrome-stable"),
    ("edge", "microsoft-edge-stable"),
    ("vivaldi.*beta", "vivaldi-beta"),
    ("vivaldi.*snapshot", "vivaldi-snapshot"),
    ("vivaldi", "vivaldi-stable"),
    ("librewolf", "librewolf"),
    ("org.mozilla.firefox", "flatpak-firefox"),
    ("org.chromium.chromium", "flatpak-chromium"),
    ("com.google.chromedev", "flatpak-chrome-unstable"),
    ("com.google.chrome", "flatpak-chrome"),
    ("com.brave.browser", "flatpak-brave"),
    ("com.microsoft.edge", "flatpak-edge"),
    ("com.github.eloston.ungoogledchromium", "flatpak-ungoogled-chromium"),
    ("io.gitlab.librewolf", "flatpak-librewolf"),
]


def get_display_name(browser_id: str) -> str:
    """Return user-friendly name for a browser ID, fallback to raw ID."""
    return BROWSER_DISPLAY_NAMES.get(browser_id, browser_id)


def match_desktop_to_browser(desktop_name: str) -> str | None:
    """Match a desktop file name to a browser ID using pattern map."""
    import re

    lower = desktop_name.lower()
    for pattern, browser_id in DESKTOP_PATTERN_MAP:
        if re.search(pattern, lower):
            return browser_id
    return None
