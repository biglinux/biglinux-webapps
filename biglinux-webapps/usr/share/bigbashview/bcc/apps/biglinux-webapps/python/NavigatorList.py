from os import path
import json

NATIVE_NAVIGATORS_LIST = {
    "brave": {
        "name": "brave",
        "path": "/usr/lib/brave-browser/brave",
        "label": "BRAVE"
    },
    "google-chrome-stable": {
        "name": "google-chrome-stable",
        "path": "/opt/google/chrome/google-chrome",
        "label": "CHROME"
    },
    "chromium": {
        "name": "chromium",
        "path": "/usr/lib/chromium/chromium",
        "label": "CHROMIUM"
    },
    "microsoft-edge-stable": {
        "name": "microsoft-edge-stable",
        "path": "/opt/microsoft/msedge/microsoft-edge",
        "label": "EDGE"
    },
    "firefox": {
        "name": "firefox",
        "path": "/usr/lib/firefox/firefox",
        "label": "FIREFOX"
    },
    "falkon": {
        "name": "falkon",
        "path": "/usr/bin/falkon",
        "label": "FALKON"
    },
    "librewolf": {
        "name": "librewolf",
        "path": "/usr/lib/librewolf/librewolf",
        "label": "LIBREWOLF"
    },
    "vivaldi-stable": {
        "name": "vivaldi-stable",
        "path": "/opt/vivaldi/vivaldi",
        "label": "VIVALDI"
    }
}

FLATPAK_NAVIGATORS_LIST = {
    "brave": {
        "name": "com.brave.Browser",
        "path": "/var/lib/flatpak/exports/bin/com.brave.Browser",
        "label": "BRAVE (FLATPAK)"
    },
    "chrome": {
        "name": "com.google.Chrome",
        "path": "/var/lib/flatpak/exports/bin/com.google.Chrome",
        "label": "CHROME (FLATPAK)"
    },
    "chromium": {
        "name": "org.chromium.Chromium",
        "path": "/var/lib/flatpak/exports/bin/org.chromium.Chromium",
        "label": "CHROMIUM (FLATPAK)"
    },
    "edge": {
        "name": "com.microsoft.Edge",
        "path": "/var/lib/flatpak/exports/bin/com.microsoft.Edge",
        "label": "EDGE (FLATPAK)"
    },
    "epiphany": {
        "name": "org.gnome.Epiphany",
        "path": "/var/lib/flatpak/exports/bin/org.gnome.Epiphany",
        "label": "EPIPHANY (FLATPAK)"
    },
    "firefox": {
        "name": "org.mozilla.firefox",
        "path": "/var/lib/flatpak/exports/bin/org.mozilla.firefox",
        "label": "FIREFOX (FLATPAK)"
    },
    "librewolf": {
        "name": "io.gitlab.librewolf-community",
        "path": "/var/lib/flatpak/exports/bin/io.gitlab.librewolf-community",
        "label": "LIBREWOLF (FLATPAK)"
    },
    "ungoogled": {
        "name": "com.github.Eloston.UngoogledChromium",
        "path": "/var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium",
        "label": "UNGOOGLED (FLATPAK)"
    }
}

INSTALLED_BROWSERS = []


def checkNavigators(navigatorList: dict[str, dict[str, str]], callback):
    for navigator in navigatorList:
        navigatorPath = navigatorList[navigator]["path"]
        if path.isfile(navigatorPath):
            callback()
            INSTALLED_BROWSERS.append(navigatorList[navigator].copy())


nativesCount = 0


def incrementNative():
    global nativesCount
    nativesCount += 1


checkNavigators(NATIVE_NAVIGATORS_LIST, incrementNative)


print(json.dumps({"browsers": INSTALLED_BROWSERS}, indent=4))
