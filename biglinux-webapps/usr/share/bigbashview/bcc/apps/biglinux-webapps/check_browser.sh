#!/usr/bin/env bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

ONLY=false

mkdir -p ~/.bigwebapps
BASEDIR=/usr/share/bigbashview/bcc/apps/biglinux-webapps

if [ -e /usr/lib/brave-browser/brave ] || [ -e /opt/brave-bin/brave ];then
    printf "brave" > ~/.bigwebapps/BROWSER
elif [ -e /opt/google/chrome/google-chrome ];then
    printf "google-chrome-stable" > ~/.bigwebapps/BROWSER
elif [ -e /usr/lib/chromium/chromium ];then
    printf "chromium" > ~/.bigwebapps/BROWSER
elif [ -e /opt/microsoft/msedge/microsoft-edge ];then
    printf "microsoft-edge-stable" > ~/.bigwebapps/BROWSER
elif [ -e /usr/lib/firefox/firefox ];then
    "$BASEDIR"/change_browser.sh "brave" "firefox"
elif [ -e /usr/lib/librewolf/librewolf ];then
    "$BASEDIR"/change_browser.sh "brave" "librewolf"
elif [ -e /usr/bin/falkon ];then
    printf "falkon" > ~/.bigwebapps/BROWSER
elif [ -e /opt/vivaldi/vivaldi ];then
    printf "vivaldi-stable" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/com.brave.Browser ];then
    printf "com.brave.Browser" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/com.google.Chrome ];then
    printf "com.google.Chrome" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/org.chromium.Chromium ];then
    printf "org.chromium.Chromium" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium ];then
    printf "com.github.Eloston.UngoogledChromium" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/com.microsoft.Edge ];then
    printf "com.microsoft.Edge" > ~/.bigwebapps/BROWSER
elif [ -e /var/lib/flatpak/exports/bin/org.gnome.Epiphany ];then
    ONLY=true
elif [ -e /var/lib/flatpak/exports/bin/org.mozilla.firefox ];then
    "$BASEDIR"/change_browser.sh "brave" "org.mozilla.firefox"
elif [ -e /var/lib/flatpak/exports/bin/io.gitlab.librewolf-community ];then
    "$BASEDIR"/change_browser.sh "brave" "io.gitlab.librewolf-community"
else
    yad --image=emblem-warning --image-on-top --form --width=500 --height=100 --fixed \
    --align=center \
    --text $"Não existem navegadores instalados compatíveis com os WebApps!" \
    --button=$" Fechar" \
    --center --on-top --borders=20 --title=$"Navegador Compatível" \
    --window-icon=emblem-warning
    exit
fi

if [ "$ONLY" = "true" ];then
    yad --image=emblem-warning --image-on-top --form --width=500 --height=100 --fixed \
    --align=center \
    --text $"Será necessário instalar mais um navegador compatível!" \
    --button=$" Fechar" \
    --center --on-top --borders=20 --title=$"Navegador Compatível" \
    --window-icon=emblem-warning
    exit
fi

[ "$(<~/.bigwebapps/BROWSER)" = "brave-browser" ] && printf "brave" > ~/.bigwebapps/BROWSER
exit
