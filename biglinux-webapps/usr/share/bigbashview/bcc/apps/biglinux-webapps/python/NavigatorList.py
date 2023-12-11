from os import path
import json

NATIVE_NAVIGATORS_LIST = {
    "brave": {
        "name": "brave",
        "path": "/usr/lib/brave-browser/brave",
        "label": "BRAVE",
        "icon": "icons/brave.svg"
    },
    "google-chrome-stable": {
        "name": "google-chrome-stable",
        "path": "/opt/google/chrome/google-chrome",
        "label": "CHROME",
        "icon": "icons/chrome.svg"
    },
    "chromium": {
        "name": "chromium",
        "path": "/usr/lib/chromium/chromium",
        "label": "CHROMIUM",
        "icon": "icons/chromium.svg"
    },
    "microsoft-edge-stable": {
        "name": "microsoft-edge-stable",
        "path": "/opt/microsoft/msedge/microsoft-edge",
        "label": "EDGE",
        "icon": "icons/edge.svg"
    },
    "firefox": {
        "name": "firefox",
        "path": "/usr/lib/firefox/firefox",
        "label": "FIREFOX",
        "icon": "icons/firefox.svg"
    },
    "falkon": {
        "name": "falkon",
        "path": "/usr/bin/falkon",
        "label": "FALKON",
        "icon": "icons/falkon.svg"
    },
    "librewolf": {
        "name": "librewolf",
        "path": "/usr/lib/librewolf/librewolf",
        "label": "LIBREWOLF",
        "icon": "icons/librewolf.svg"
    },
    "vivaldi-stable": {
        "name": "vivaldi-stable",
        "path": "/opt/vivaldi/vivaldi",
        "label": "VIVALDI",
        "icon": "icons/vivaldi.svg"
    }
}

FLATPAK_NAVIGATORS_LIST = {
    "brave": {
        "name": "com.brave.Browser",
        "path": "/var/lib/flatpak/exports/bin/com.brave.Browser",
        "label": "BRAVE (FLATPAK)",
        "icon": "icons/brave.svg"
    },
    "chrome": {
        "name": "com.google.Chrome",
        "path": "/var/lib/flatpak/exports/bin/com.google.Chrome",
        "label": "CHROME (FLATPAK)",
        "icon": "icons/chrome.svg"
    },
    "chromium": {
        "name": "org.chromium.Chromium",
        "path": "/var/lib/flatpak/exports/bin/org.chromium.Chromium",
        "label": "CHROMIUM (FLATPAK)",
        "icon": "icons/chromium.svg"
    },
    "edge": {
        "name": "com.microsoft.Edge",
        "path": "/var/lib/flatpak/exports/bin/com.microsoft.Edge",
        "label": "EDGE (FLATPAK)",
        "icon": "icons/edge.svg"
    },
    "epiphany": {
        "name": "org.gnome.Epiphany",
        "path": "/var/lib/flatpak/exports/bin/org.gnome.Epiphany",
        "label": "EPIPHANY (FLATPAK)",
        "icon": "icons/epiphany.svg"
    },
    "firefox": {
        "name": "org.mozilla.firefox",
        "path": "/var/lib/flatpak/exports/bin/org.mozilla.firefox",
        "label": "FIREFOX (FLATPAK)",
        "icon": "icons/firefox.svg"
    },
    "librewolf": {
        "name": "io.gitlab.librewolf-community",
        "path": "/var/lib/flatpak/exports/bin/io.gitlab.librewolf-community",
        "label": "LIBREWOLF (FLATPAK)",
        "icon": "icons/librewolf.svg"
    },
    "ungoogled": {
        "name": "com.github.Eloston.UngoogledChromium",
        "path": "/var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium",
        "label": "UNGOOGLED (FLATPAK)",
        "icon": "icons/ungoogled.svg"
    }
}

INSTALLED_BROWSERS = []


def checkNavigators(navigatorList: dict[str, dict[str, str]], getDynamicValues):
    for navigator in navigatorList:
        navigatorPath = navigatorList[navigator]["path"]
        if path.isfile(navigatorPath):
            getDynamicValues()
            INSTALLED_BROWSERS.append({
                **navigatorList[navigator].copy(),
                **getDynamicValues()
            })


checkNavigators(NATIVE_NAVIGATORS_LIST, lambda: ({"native": True}))
checkNavigators(FLATPAK_NAVIGATORS_LIST, lambda: ({"flatpak": True}))


print(json.dumps({"browsers": INSTALLED_BROWSERS}, indent=4))
