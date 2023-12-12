from os import path
import json

import gettext
lang_translations = gettext.translation(
    'biglinux-webapps',
    localedir='/usr/share/locale',
    fallback=True
)
lang_translations.install()

# define _ shortcut for translations
_ = lang_translations.gettext

NATIVE_NAVIGATORS_LIST = {
    "brave": {
        "name": "brave",
        "path": "/usr/lib/brave-browser/brave",
        "label": _("BRAVE"),
        "icon": "icons/brave.svg"
    },
    "google-chrome-stable": {
        "name": "google-chrome-stable",
        "path": "/opt/google/chrome/google-chrome",
        "label": _("CHROME"),
        "icon": "icons/chrome.svg"
    },
    "chromium": {
        "name": "chromium",
        "path": "/usr/lib/chromium/chromium",
        "label": _("CHROMIUM"),
        "icon": "icons/chromium.svg"
    },
    "microsoft-edge-stable": {
        "name": "microsoft-edge-stable",
        "path": "/opt/microsoft/msedge/microsoft-edge",
        "label": _("EDGE"),
        "icon": "icons/edge.svg"
    },
    "firefox": {
        "name": "firefox",
        "path": "/usr/lib/firefox/firefox",
        "label": _("FIREFOX"),
        "icon": "icons/firefox.svg"
    },
    "falkon": {
        "name": "falkon",
        "path": "/usr/bin/falkon",
        "label": _("FALKON"),
        "icon": "icons/falkon.svg"
    },
    "librewolf": {
        "name": "librewolf",
        "path": "/usr/lib/librewolf/librewolf",
        "label": _("LIBREWOLF"),
        "icon": "icons/librewolf.svg"
    },
    "vivaldi-stable": {
        "name": "vivaldi-stable",
        "path": "/opt/vivaldi/vivaldi",
        "label": _("VIVALDI"),
        "icon": "icons/vivaldi.svg"
    }
}

FLATPAK_NAVIGATORS_LIST = {
    "brave": {
        "name": "com.brave.Browser",
        "path": "/var/lib/flatpak/exports/bin/com.brave.Browser",
        "label": _("BRAVE (FLATPAK)"),
        "icon": "icons/brave.svg"
    },
    "chrome": {
        "name": "com.google.Chrome",
        "path": "/var/lib/flatpak/exports/bin/com.google.Chrome",
        "label": _("CHROME (FLATPAK)"),
        "icon": "icons/chrome.svg"
    },
    "chromium": {
        "name": "org.chromium.Chromium",
        "path": "/var/lib/flatpak/exports/bin/org.chromium.Chromium",
        "label": _("CHROMIUM (FLATPAK)"),
        "icon": "icons/chromium.svg"
    },
    "edge": {
        "name": "com.microsoft.Edge",
        "path": "/var/lib/flatpak/exports/bin/com.microsoft.Edge",
        "label": _("EDGE (FLATPAK)"),
        "icon": "icons/edge.svg"
    },
    "epiphany": {
        "name": "org.gnome.Epiphany",
        "path": "/var/lib/flatpak/exports/bin/org.gnome.Epiphany",
        "label": _("EPIPHANY (FLATPAK)"),
        "icon": "icons/epiphany.svg"
    },
    "firefox": {
        "name": "org.mozilla.firefox",
        "path": "/var/lib/flatpak/exports/bin/org.mozilla.firefox",
        "label": _("FIREFOX (FLATPAK)"),
        "icon": "icons/firefox.svg"
    },
    "librewolf": {
        "name": "io.gitlab.librewolf-community",
        "path": "/var/lib/flatpak/exports/bin/io.gitlab.librewolf-community",
        "label": _("LIBREWOLF (FLATPAK)"),
        "icon": "icons/librewolf.svg"
    },
    "ungoogled": {
        "name": "com.github.Eloston.UngoogledChromium",
        "path": "/var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium",
        "label": _("UNGOOGLED (FLATPAK)"),
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
