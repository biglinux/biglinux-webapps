#!/usr/bin/env bash

_NAMEDESK="$(sed 's|https\:\/\/||;s|www\.||;s|\/.*||;s|\.|-|g' <<< $urldesk)"
USER_DESKTOP="$(xdg-user-dir DESKTOP)"
LINK_APP="$HOME/.local/share/applications/$_NAMEDESK-$RANDOM-webapp-biglinux-custom.desktop"
DIR="$(basename $LINK_APP | sed 's|-webapp-biglinux-custom.desktop||')"

if [ "$(grep 'firefox' <<< $browser)" ];then

    [ ! "$(grep 'https://' <<< $urldesk)" ] && urldesk="https://$urldesk"

    [ "$tvmode" = "on" ] && {
        YTCODE="$(basename $urldesk | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        urldesk="https://www.youtube.com/embed/$YTCODE"
    }

    if [ -z "$icondesk" ];then
        ICON_FILE="webapp"
    else
        NAME_FILE="$(basename $icondesk|sed 's| |-|g')"
        ICON_FILE="$HOME/.local/share/icons/$NAME_FILE"
        [ "$(dirname $icondesk)" = "/tmp" ] && mv "$icondesk" $ICON_FILE || cp "$icondesk" $ICON_FILE
    fi

DESKBIN="$HOME/.local/bin/$DIR"

cat > "$DESKBIN" <<EOF
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

FOLDER="$HOME/.bigwebapps/$DIR"

if [ ! "\$(grep 'toolkit.legacyUserProfileCustomizations.stylesheets' \$FOLDER/prefs.js)" ]; then
    [ -d "\$FOLDER" ] && rm -r "\$FOLDER"
    mkdir -p "\$FOLDER/chrome"
    echo 'user_pref("media.eme.enabled", true);' >> "\$FOLDER/prefs.js"
    echo 'user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);' >> "\$FOLDER/prefs.js"
fi

# Custom profile
echo '#nav-bar{visibility: collapse;} #TabsToolbar{visibility: collapse;}' >> "\$FOLDER/chrome/userChrome.css"
echo 'user_pref("browser.tabs.warnOnClose", false);' >> "\$FOLDER/user.js"
sed -i 's|user_pref("browser.urlbar.placeholderName.*||g' "\$FOLDER/prefs.js"

CLASS="$browser-webapp-$_NAMEDESK"

MOZ_DISABLE_GMP_SANDBOX=1 MOZ_DISABLE_CONTENT_SANDBOX=1 \
$browser --class="\$CLASS" --profile "\$FOLDER" --no-remote --new-instance "$urldesk" &

count=0
while [ \$count -lt 100 ]; do
    wininfo="\$(xwininfo -root -children -all | grep \\"Navigator\\"\\ \\"\$CLASS\\")"
    if [ "\$wininfo" ]; then
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
X-KDE-StartupNotify=true" > "$LINK_APP"

    chmod +x "$LINK_APP"

    [ "$shortcut" = "on" ] && {
        ln -s "$LINK_APP" "$USER_DESKTOP/$DIR-webapp-biglinux-custom.desktop"
        chmod 755 "$USER_DESKTOP/$DIR-webapp-biglinux-custom.desktop"
    }

elif [ "$(grep 'epiphany' <<< $browser)" ];then

    [ ! "$(grep 'https://' <<< $urldesk)" ] && urldesk="https://$urldesk"

    [ "$tvmode" = "on" ] && {
        YTCODE="$(basename $urldesk | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        urldesk="https://www.youtube.com/embed/$YTCODE"
    }

    FOLDER="$HOME/.bigwebapps/org.gnome.Epiphany.WebApp-$DIR-webapp-biglinux-custom"
    EPI_FILE="org.gnome.Epiphany.WebApp-$DIR-webapp-biglinux-custom.desktop"
    EPI_LINK="$HOME/.local/share/applications/$EPI_FILE"
    mkdir -p $FOLDER
    > "$FOLDER/.app"
    echo 35 > "$FOLDER/.migrated"

    ICON_FILE="$FOLDER/app-icon.png"

    if [ -z "$icondesk" ];then
        cp "/usr/share/bigbashview/bcc/apps/biglinux-webapps/img/default.png" $ICON_FILE
    else
        [ "$(dirname $icondesk)" = "/tmp" ] && mv "$icondesk" $ICON_FILE || cp "$icondesk" $ICON_FILE
    fi

echo "[Desktop Entry]
Name=$namedesk
Exec=epiphany --application-mode --profile=$FOLDER $urldesk
StartupNotify=true
Terminal=false
Type=Application
Categories=$category;
Icon=$ICON_FILE
StartupWMClass=org.gnome.Epiphany.WebApp-$DIR-webapp-biglinux-custom
X-Purism-FormFactor=Workstation;Mobile;" > "$FOLDER/$EPI_FILE"

    chmod +x "$FOLDER/$EPI_FILE"
    ln -s "$FOLDER/$EPI_FILE" "$EPI_LINK"

    [ "$shortcut" = "on" ] && {
        ln -s "$FOLDER/$EPI_FILE" "$USER_DESKTOP/$EPI_FILE"
        chmod 755 "$USER_DESKTOP/$EPI_FILE"
    }

else
    FOLDER="$HOME/.bigwebapps/$DIR"

    [ ! "$(grep 'https://' <<< $urldesk)" ] && urldesk="https://$urldesk"

    [ "$tvmode" = "on" ] && {
        YTCODE="$(basename $urldesk | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        urldesk="https://www.youtube.com/embed/$YTCODE"
    }

    [ "$newperfil" = "on" ] && browser="$browser --user-data-dir=$FOLDER"

    CUT_HTTP="$(sed 's|https://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|g;s|?||g;s|=|_|g' <<< "$urldesk")"

    if [ -z "$icondesk" ];then
        ICON_FILE="webapp"
    else
        NAME_FILE="$(basename $icondesk|sed 's| |-|g')"
        ICON_FILE="$HOME/.local/share/icons/$NAME_FILE"
        [ "$(dirname $icondesk)" = "/tmp" ] && mv "$icondesk" $ICON_FILE || cp "$icondesk" $ICON_FILE
    fi

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

    [ "$shortcut" = "on" ] && {
        ln -s "$LINK_APP" "$USER_DESKTOP/$DIR-webapp-biglinux-custom.desktop"
        chmod 755 "$USER_DESKTOP/$DIR-webapp-biglinux-custom.desktop"
    }
fi

nohup update-desktop-database -q $HOME/.local/share/applications &
nohup kbuildsycoca5 &> /dev/null &

resp="$?"
echo -n $resp
exit
