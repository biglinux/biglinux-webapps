#!/usr/bin/env bash

_NAMEDESK=$(sed 's|https\:\/\/||;s|http\:\/\/||;s|www\.||;s|\/.*||;s|\.|-|g' <<< "$urldesk")
USER_DESKTOP=$(xdg-user-dir DESKTOP)
LINK_APP="$HOME/.local/share/applications/$_NAMEDESK-$RANDOM-webapp-biglinux-custom.desktop"
BASENAME_APP="${LINK_APP##*/}"
NAME="${BASENAME_APP/-webapp-biglinux-custom.desktop/}"
DIR_PROF="$HOME/.bigwebapps/$NAME"
FILE_LINK="$USER_DESKTOP/$NAME-webapp-biglinux-custom.desktop"
BASENAME_ICON="${icondesk##*/}"
NAME_FILE="${BASENAME_ICON// /-}"
ICON_FILE=~/.local/share/icons/"$NAME_FILE"

if grep -qiE 'firefox|librewolf' <<< "$browser";then
    browser_name="$browser"

    if ! grep -qiE '^http:|^https:|^localhost|^127' <<< "$urldesk";then
        urldesk="https://$urldesk"
    fi

    if [ "${icondesk##*/}" = "default-webapps.png" ];then
        cp "$icondesk" "$ICON_FILE"
    else
        mv "$icondesk" "$ICON_FILE"
    fi

    if [ "$browser" = "org.mozilla.firefox" ];then
        browser="/var/lib/flatpak/exports/bin/org.mozilla.firefox"
        DIR_PROF="$HOME/.var/app/org.mozilla.firefox/data/$NAME"
    elif [ "$browser" = "io.gitlab.librewolf-community" ];then
        browser="/var/lib/flatpak/exports/bin/io.gitlab.librewolf-community"
        DIR_PROF="$HOME/.var/app/io.gitlab.librewolf-community/data/$NAME"
    fi

DESKBIN="$HOME/.local/bin/$NAME"

cat > "$DESKBIN" <<EOF
#!/usr/bin/env sh

FOLDER=$DIR_PROF
CLASS="$browser_name-webapp-$_NAMEDESK"

if [ ! -d "\$FOLDER" ];then
    mkdir -p "\$FOLDER/chrome"
    cp -a /usr/share/bigbashview/bcc/apps/biglinux-webapps/profile/userChrome.css "\$FOLDER/chrome"
    cp -a /usr/share/bigbashview/bcc/apps/biglinux-webapps/profile/user.js "\$FOLDER"
fi

MOZ_DISABLE_GMP_SANDBOX=1 MOZ_DISABLE_CONTENT_SANDBOX=1 \\
XAPP_FORCE_GTKWINDOW_ICON=$ICON_FILE \\
$browser --class="\$CLASS" --profile "\$FOLDER" --no-remote --new-instance "$urldesk" &
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
X-KDE-StartupNotify=true" > "$LINK_APP"

    chmod +x "$LINK_APP"

    if [ "$shortcut" = "on" ];then
        ln -s "$LINK_APP" "$FILE_LINK"
        chmod 755 "$FILE_LINK"
        gio set "$FILE_LINK" -t string metadata::trust "true"
    fi

elif grep -q 'org.gnome.Epiphany' <<< "$browser";then

    if ! grep -Eq '^http:|^https:|^localhost|^127' <<< "$urldesk";then
        urldesk="https://$urldesk"
    fi

    DIR_PORTAL="$HOME/.local/share/xdg-desktop-portal"
    DIR_PORTAL_APP="$DIR_PORTAL/applications"
    DIR_PORTAL_ICON="$DIR_PORTAL/icons/64x64"

    mkdir -p "$DIR_PORTAL_APP"
    mkdir -p "$DIR_PORTAL_ICON"

    FOLDER_DATA="$HOME/.var/app/org.gnome.Epiphany/data/org.gnome.Epiphany.WebApp_$NAME-webapp-biglinux-custom"
    browser="/var/lib/flatpak/exports/bin/org.gnome.Epiphany"
    EPI_FILEDESK="org.gnome.Epiphany.WebApp_$NAME-webapp-biglinux-custom.desktop"
    EPI_DIR_FILEDESK="$DIR_PORTAL_APP/$EPI_FILEDESK"
    EPI_FILE_ICON="$DIR_PORTAL_ICON/${EPI_FILEDESK/.desktop/}.png"

    EPI_LINK="$HOME/.local/share/applications/$EPI_FILEDESK"
    EPI_DESKTOP_LINK="$USER_DESKTOP/$EPI_FILEDESK"
    mkdir -p "$FOLDER_DATA"
    true > "$FOLDER_DATA/.app"
    echo -n 37 > "$FOLDER_DATA/.migrated"

    if [ "${icondesk##*/}" = "default-webapps.png" ];then
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
X-Flatpak=org.gnome.Epiphany" > "$EPI_DIR_FILEDESK"

    chmod +x "$EPI_DIR_FILEDESK"
    ln -s "$EPI_DIR_FILEDESK" "$EPI_LINK"

    if [ "$shortcut" = "on" ];then
        ln -s "$EPI_DIR_FILEDESK" "$EPI_DESKTOP_LINK"
        chmod 755 "$EPI_DESKTOP_LINK"
        gio set "$EPI_DESKTOP_LINK" -t string metadata::trust "true"
    fi

else
    case $browser in
        com.brave.Browser)
            browser="/var/lib/flatpak/exports/bin/com.brave.Browser"
            DIR_PROF="$HOME/.var/app/com.brave.Browser/data/$NAME"
        ;;

        com.google.Chrome)
            browser="/var/lib/flatpak/exports/bin/com.google.Chrome"
            DIR_PROF="$HOME/.var/app/com.google.Chrome/data/$NAME"
        ;;

        com.microsoft.Edge)
            browser="/var/lib/flatpak/exports/bin/com.microsoft.Edge"
            DIR_PROF="$HOME/.var/app/com.microsoft.Edge/data/$NAME"
        ;;

        org.chromium.Chromium)
            browser="/var/lib/flatpak/exports/bin/org.chromium.Chromium"
            DIR_PROF="$HOME/.var/app/org.chromium.Chromium/data/$NAME"
        ;;

        com.github.Eloston.UngoogledChromium)
            browser="/var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium"
            DIR_PROF="$HOME/.var/app/com.github.Eloston.UngoogledChromium/data/$NAME"
        ;;
    esac

    if ! grep -Eq '^http:|^https:|^localhost|^127' <<< "$urldesk";then
        urldesk="https://$urldesk"
    fi

    if [ "$newperfil" = "on" ];then
        browser="$browser --user-data-dir=$DIR_PROF --no-first-run"
    fi

    if [ "${icondesk##*/}" = "default-webapps.png" ];then
        cp "$icondesk" "$ICON_FILE"
    else
        mv "$icondesk" "$ICON_FILE"
    fi

    CUT_HTTP=$(sed 's|https://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|g;s|?||g;s|=|_|g' <<< "$urldesk")

echo "[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$namedesk
Exec=$browser --class=$CUT_HTTP,Chromium-browser --profile-directory=Default --app=$urldesk
Icon=$ICON_FILE
Categories=$category;
StartupWMClass=$CUT_HTTP" > "$LINK_APP"

    chmod +x "$LINK_APP"

    if [ "$shortcut" = "on" ];then
        ln -s "$LINK_APP" "$FILE_LINK"
        chmod 755 "$FILE_LINK"
        gio set "$FILE_LINK" -t string metadata::trust "true"
    fi
fi

nohup update-desktop-database -q ~/.local/share/applications &
nohup kbuildsycoca5 &> /dev/null &

rm -f /tmp/*.png
rm -rf /tmp/.bigwebicons
exit
