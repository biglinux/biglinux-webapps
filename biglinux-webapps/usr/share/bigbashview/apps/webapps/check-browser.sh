#!/usr/bin/env bash

if [ -e /usr/lib/brave-browser/brave ] || [ -e /opt/brave-bin/brave ];then
    echo "brave"
elif [ -e /opt/google/chrome/google-chrome ];then
    echo "google-chrome-stable"
elif [ -e /usr/lib/chromium/chromium ];then
    echo "chromium"
elif [ -e /opt/microsoft/msedge/microsoft-edge ];then
    echo "microsoft-edge-stable"
elif [ -e /usr/lib/firefox/firefox ];then
    echo "firefox"
elif [ -e /usr/lib/librewolf/librewolf ];then
    echo "librewolf"
elif [ -e /opt/vivaldi/vivaldi ];then
    echo "vivaldi-stable"
elif [ -e /var/lib/flatpak/exports/bin/com.brave.Browser ];then
    echo "/var/lib/flatpak/exports/bin/com.brave.Browser"
elif [ -e /var/lib/flatpak/exports/bin/com.google.Chrome ];then
    echo "/var/lib/flatpak/exports/bin/com.google.Chrome"
elif [ -e /var/lib/flatpak/exports/bin/org.chromium.Chromium ];then
    echo "/var/lib/flatpak/exports/bin/org.chromium.Chromium"
elif [ -e /var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium ];then
    echo "/var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium"
elif [ -e /var/lib/flatpak/exports/bin/com.microsoft.Edge ];then
    echo "/var/lib/flatpak/exports/bin/com.microsoft.Edge"
elif [ -e /var/lib/flatpak/exports/bin/org.mozilla.firefox ];then
    echo "/var/lib/flatpak/exports/bin/org.mozilla.firefox"
elif [ -e /var/lib/flatpak/exports/bin/io.gitlab.librewolf-community ];then
    echo "/var/lib/flatpak/exports/bin/io.gitlab.librewolf-community"
fi
