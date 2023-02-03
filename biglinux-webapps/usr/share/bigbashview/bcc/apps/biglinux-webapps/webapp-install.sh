#!/usr/bin/env bash

FLATPAK_BIN=/var/lib/flatpak/exports/bin
SNAPD_BIN=/var/lib/snapd/snap/bin

_NAMEDESK=$(sed 's|https\:\/\/||;s|www\.||;s|\/.*||;s|\.|-|g' <<<"$urldesk")
USER_DESKTOP=$(xdg-user-dir DESKTOP)

NAME="${_NAMEDESK}-$RANDOM"
BASENAME_APP="$NAME-webapp-biglinux-custom.desktop"
LINK_APP="$HOME/.local/share/applications/$BASENAME_APP"
DIR_PROF="$HOME/.bigwebapps/$NAME"
FILE_LINK="$USER_DESKTOP/$BASENAME_APP"

BASENAME_ICON="${icondesk##*/}"
NAME_FILE="${BASENAME_ICON// /-}"
ICON_FILE=~/.local/share/icons/"$NAME_FILE"

if [[ "$urldesk" != *"://"* ]]; then
    urldesk="https://$urldesk"
fi

shopt -s nocasematch
case "$browser" in
*firefox | *firedragon | *librewolf*)
    browser_name="$browser"

    case "$browser" in
    *.firefox | *.librewolf*)
        if [ -x "$FLATPAK_BIN/$browser" ]; then
            browser="$FLATPAK_BIN/$browser"
            DIR_PROF="$HOME/.var/app/$browser_name/data/$NAME"
        fi
        ;;
    esac

    if [ "${icondesk##*/}" = "default-webapps.png" ]; then
        cp "$icondesk" "$ICON_FILE"
    else
        mv "$icondesk" "$ICON_FILE"
    fi

    DESKBIN="$HOME/.local/bin/$NAME"

    cat >"$DESKBIN" <<EOF
#!/usr/bin/env sh
#
# Amofi - App mode for Firefox
# Copyright (C) 2017-2019  SEPBIT
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# dash version 0.5
#
# @author    Vitor Guia <contato@vitor.guia.nom.br>
# @Modified by Bruno Goncalves <www.biglinux.com.br>
# @copyright 2017-2019 SEPBIT
# @license   http://www.gnu.org/licenses GPL-3.0-or-later
# @see       https://notabug.org/sepbit/amofi Repository of Amofi

FOLDER=$DIR_PROF

if ! grep -q 'toolkit.legacyUserProfileCustomizations.stylesheets' "\$FOLDER/prefs.js" 2>/dev/null;then
    [ -d "\$FOLDER" ] && rm -r "\$FOLDER"
    mkdir -p "\$FOLDER/chrome"
    echo 'user_pref("media.eme.enabled", true);' >> "\$FOLDER/prefs.js"
    echo 'user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);' >> "\$FOLDER/prefs.js"
    sed -i 's|user_pref("browser.urlbar.placeholderName.*||g' "\$FOLDER/prefs.js"

    # Custom profile
    echo '#nav-bar{visibility: collapse;} #TabsToolbar{visibility: collapse;}' >> "\$FOLDER/chrome/userChrome.css"
    echo 'user_pref("browser.tabs.warnOnClose", false);' >> "\$FOLDER/user.js"
fi

CLASS="$browser_name-webapp-$_NAMEDESK"

MOZ_DISABLE_GMP_SANDBOX=1 MOZ_DISABLE_CONTENT_SANDBOX=1 \
$browser --class="\$CLASS" --profile "\$FOLDER" --no-remote --new-instance "$urldesk" &

count=0
while [ \$count -lt 100 ];do
    wininfo=\$(xwininfo -root -children -all | grep \\"Navigator\\"\\ \\"\$CLASS\\")
    if [ "\$wininfo" ];then
        xseticon -id "\$(awk '{print \$1}' <<< \$wininfo)" $ICON_FILE
        count=100
    else
        let count=count+1;
    fi
    sleep 0.5
done
EOF

    chmod +x "$DESKBIN"

    echo "[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$namedesk
Exec=$DESKBIN
Icon=$ICON_FILE
Categories=$category;
X-KDE-StartupNotify=true" >"$LINK_APP"

    chmod +x "$LINK_APP"

    if [ "$shortcut" = "on" ]; then
        ln -s "$LINK_APP" "$FILE_LINK"
        chmod 755 "$FILE_LINK"
        gio set "$FILE_LINK" -t string metadata::trust "true"
    fi
    ;;
*epiphany)
    case "$browser" in
    *.Epiphany)
        if [ -x "$FLATPAK_BIN/$browser" ]; then
            DIR_PORTAL="$HOME/.local/share/xdg-desktop-portal"
            DIR_PORTAL_APP="$DIR_PORTAL/applications"
            DIR_PORTAL_ICON="$DIR_PORTAL/icons/64x64"

            mkdir -p "$DIR_PORTAL_APP"
            mkdir -p "$DIR_PORTAL_ICON"

            FOLDER_DATA="$HOME/.var/app/org.gnome.Epiphany/data/org.gnome.Epiphany.WebApp_$NAME-webapp-biglinux-custom"
            browser="/var/lib/flatpak/exports/bin/org.gnome.Epiphany"
            FLATPAK_LINE="X-Flatpak=org.gnome.Epiphany"
            EPI_FILEDESK="org.gnome.Epiphany.WebApp_$NAME-webapp-biglinux-custom.desktop"
            EPI_DIR_FILEDESK="$DIR_PORTAL_APP/$EPI_FILEDESK"
            EPI_FILE_ICON="$DIR_PORTAL_ICON/${EPI_FILEDESK/.desktop/}.png"
        fi
        ;;
    *)
        FOLDER_DATA="$HOME/.bigwebapps/org.gnome.Epiphany.WebApp_$NAME-webapp-biglinux-custom"
        FLATPAK_LINE=""
        EPI_FILEDESK="org.gnome.Epiphany.WebApp_$NAME-webapp-biglinux-custom.desktop"
        EPI_DIR_FILEDESK="$FOLDER_DATA/$EPI_FILEDESK"
        EPI_FILE_ICON="$FOLDER_DATA/app-icon.png"
        ;;
    esac

    EPI_LINK="$HOME/.local/share/applications/$EPI_FILEDESK"
    EPI_DESKTOP_LINK="$USER_DESKTOP/$EPI_FILEDESK"
    mkdir -p "$FOLDER_DATA"
    true >"$FOLDER_DATA/.app"
    echo -n 37 >"$FOLDER_DATA/.migrated"

    if [ "${icondesk##*/}" = "default-webapps.png" ]; then
        cp "$icondesk" "$EPI_FILE_ICON"
    else
        mv "$icondesk" "$EPI_FILE_ICON"
    fi

    echo "[Desktop Entry]
Name=$namedesk
Exec=$browser --application-mode --profile=$FOLDER_DATA $urldesk
StartupNotify=true
Terminal=false
Type=Application
Categories=$category;
Icon=$EPI_FILE_ICON
StartupWMClass=$namedesk
X-Purism-FormFactor=Workstation;Mobile;
$FLATPAK_LINE" >"$EPI_DIR_FILEDESK"

    chmod +x "$EPI_DIR_FILEDESK"
    ln -s "$EPI_DIR_FILEDESK" "$EPI_LINK"

    if [ "$shortcut" = "on" ]; then
        ln -s "$EPI_DIR_FILEDESK" "$EPI_DESKTOP_LINK"
        chmod 755 "$EPI_DESKTOP_LINK"
        gio set "$EPI_DESKTOP_LINK" -t string metadata::trust "true"
    fi
    ;;
*)
    if [[ "$browser" = *"."* ]] && [ -x "$FLATPAK_BIN/$browser" ]; then
        browser="$FLATPAK_BIN/$browser"
        DIR_PROF="$HOME/.var/app/$browser_name/data/$NAME"
    fi

    if [ "$newperfil" = "on" ]; then
        browser="$browser --user-data-dir=$DIR_PROF --no-first-run"
    fi

    if [ "${icondesk##*/}" = "default-webapps.png" ]; then
        cp "$icondesk" "$ICON_FILE"
    else
        mv "$icondesk" "$ICON_FILE"
    fi

    CUT_HTTP=$(sed 's|https://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|g;s|?||g;s|=|_|g' <<<"$urldesk")

    echo "[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$namedesk
Exec=$browser --class=$CUT_HTTP,Chromium-browser --profile-directory=Default --app=$urldesk
Icon=$ICON_FILE
Categories=$category;
StartupWMClass=$CUT_HTTP" >"$LINK_APP"

    chmod +x "$LINK_APP"

    if [ "$shortcut" = "on" ]; then
        ln -s "$LINK_APP" "$FILE_LINK"
        chmod 755 "$FILE_LINK"
        gio set "$FILE_LINK" -t string metadata::trust "true"
    fi
    ;;

esac

nohup update-desktop-database -q ~/.local/share/applications &
nohup kbuildsycoca5 &>/dev/null &

rm -f /tmp/*.png
rm -rf /tmp/.bigwebicons
exit
