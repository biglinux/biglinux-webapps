#!/usr/bin/env bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

ONLY=false

mkdir -p ~/.bigwebapps

if [ -e /usr/lib/brave-browser/brave ] || [ -e /usr/lib/brave-bin/brave ];then
    printf "%s" "brave" > ~/.bigwebapps/BROWSER
elif [ -e /opt/google/chrome/google-chrome ];then
    printf "%s" "google-chrome-stable" > ~/.bigwebapps/BROWSER
elif [ -e /usr/lib/chromium/chromium ];then
    printf "%s" "chromium" > ~/.bigwebapps/BROWSER
elif [ -e /opt/microsoft/msedge/microsoft-edge ];then
    printf "%s" "microsoft-edge-stable" > ~/.bigwebapps/BROWSER
elif [ -e /usr/bin/epiphany ];then
    ONLY=true
elif [ -e /usr/lib/firefox/firefox ];then
    ./change_browser.sh "brave" "firefox"
elif [ -e /usr/lib/librewolf/librewolf ];then
    ./change_browser.sh "brave" "librewolf"
elif [ -e /opt/vivaldi/vivaldi ];then
    printf "%s" "vivaldi-stable" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/com.brave.Browser ];then
    printf "%s" "com.brave.Browser" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/com.google.Chrome ];then
    printf "%s" "com.google.Chrome" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/org.chromium.Chromium ];then
    printf "%s" "org.chromium.Chromium" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/com.microsoft.Edge ];then
    printf "%s" "com.microsoft.Edge" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/org.gnome.Epiphany ];then
    ONLY=true
elif [ -e /var/lib/flatpak/exports/bin/org.mozilla.firefox ];then
    ./change_browser.sh "brave" "org.mozilla.firefox"
elif [ -e /var/lib/flatpak/exports/bin/io.gitlab.librewolf-community ];then
    ./change_browser.sh "brave" "io.gitlab.librewolf-community"
else
    kdialog --sorry $"Não existem navegadores instalados compatíveis com os WebApps!" --title "WebApps BigLinux"
    exit
fi

if [ "$ONLY" = "true" ];then
    kdialog --sorry $"Será necessário instalar mais um navegador compatível!" --title "WebApps BigLinux"
    exit
fi

[ "$(<~/.bigwebapps/BROWSER)" = "brave-browser" ] && printf "%s" "brave" > ~/.bigwebapps/BROWSER
exit
