#!/usr/bin/env bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

NAMEDESKK="$(basename -s .desktop "$filedesk" | sed 's|-webapp-biglinux-custom||')"
ICONDESK="$(grep "Icon=" $filedesk | sed 's|Icon=||')"
DESKNAME="$(grep "Name=" $filedesk | sed 's|Name=||')"

if [ "$(grep "firefox$" $filedesk)" != "" ];then

    if [ -d $HOME/.bigwebapps/"$NAMEDESKK" ]; then
        rm -r $HOME/.bigwebapps/"$NAMEDESKK"
    fi
    unlink "$(xdg-user-dir DESKTOP)/$DESKNAME.desktop" &> /dev/null
    rm "$(grep "Exec=" "$filedesk" | sed 's|Exec=||')"
    xdg-desktop-menu uninstall "$filedesk"
    rm "$ICONDESK"
else
    if [ -d $HOME/.bigwebapps/"$NAMEDESKK" ]; then
        rm -r $HOME/.bigwebapps/"$NAMEDESKK"
    fi
    unlink "$(xdg-user-dir DESKTOP)/$DESKNAME.desktop" &> /dev/null
    xdg-desktop-menu uninstall "$filedesk"
    rm "$ICONDESK"
fi

nohup update-desktop-database -q $HOME/.local/share/applications &
nohup kbuildsycoca5 &> /dev/null &

NAMEDESK="$(sed 'y/áÁàÀãÃâÂéÉêÊíÍóÓõÕôÔúÚüÜçÇ/aAaAaAaAeEeEiIoOoOoOuUuUcC/;s|^ *||;s| *$||g;s| |-|g;s|/|-|g;s|.*|\L&|' <<< "$namedesk")"

if [ "$browser" = "firefox" ];then

    if [ "$(egrep "(http|https)://" <<< "$urldesk")" = "" ];then

        if [ "$tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$urldesk")" != "" ];then
            urldesk="https://www.youtube.com/embed/$(basename "$urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        else
            urldesk="https://$urldesk"
        fi

    else
        if [ "$tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$urldesk")" != "" ];then
            urldesk="https://www.youtube.com/embed/$(basename "$urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        else
            urldesk="$urldesk"
        fi
    fi

    if [ -z "$icondesk" -o "$icondesk" = "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png" ];then
        ICON_FILE="/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
    else
    	if [ "$(dirname "$icondesk")" = "/tmp" ];then
			mv "$icondesk" $HOME/.local/share/icons
		else
			cp "$icondesk" $HOME/.local/share/icons
		fi
		NAME_FILE=$(basename "$icondesk")
    	FILE_PNG=$(sed 's|\..*|.png|' <<< $NAME_FILE)
    	convert "$HOME/.local/share/icons/$NAME_FILE" -thumbnail 32x32 \
    			-alpha on -background none -flatten "$HOME/.local/share/icons/$browser-$NAMEDESK-$FILE_PNG"
    	rm "$HOME/.local/share/icons/$NAME_FILE"

        ICON_FILE="$HOME/.local/share/icons/$browser-$NAMEDESK-$FILE_PNG"
    fi

cat > "$HOME/.local/bin/$NAMEDESK-$browser" <<EOF
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
#

if [ "\$(grep "toolkit.legacyUserProfileCustomizations.stylesheets" "\$HOME/.bigwebapps/$NAMEDESK-$browser/prefs.js")" = "" ]; then
    rm -R "\$HOME/.bigwebapps/$NAMEDESK-$browser"
    mkdir -p "\$HOME/.bigwebapps/$NAMEDESK-$browser/chrome"
    echo 'user_pref("media.eme.enabled", true);' >> "\$HOME/.bigwebapps/$NAMEDESK-$browser/prefs.js"
    echo 'user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);' >> "\$HOME/.bigwebapps/$NAMEDESK-$browser/prefs.js"
fi

# Custom profile
echo "#nav-bar{visibility: collapse;} #TabsToolbar{visibility: collapse;}" >> "\$HOME/.bigwebapps/$NAMEDESK-$browser/chrome/userChrome.css"
echo "user_pref(\"browser.tabs.warnOnClose\", false);" >> "\$HOME/.bigwebapps/$NAMEDESK-$browser/user.js"
sed -i 's|user_pref("browser.urlbar.placeholderName.*||g' "\$HOME/.bigwebapps/$NAMEDESK-$browser/prefs.js"

MOZ_DISABLE_GMP_SANDBOX=1 MOZ_DISABLE_CONTENT_SANDBOX=1 \
$browser --class=$browser-webapp-$NAMEDESK -profile "\$HOME/.bigwebapps/$NAMEDESK-$browser" \
-no-remote -new-instance "$urldesk" &

count=0
while [ \$count -lt 100 ]; do
    if [ "\$(xwininfo -root -children -all | grep -iE "Navigator.*$browser-webapp-$NAMEDESK")" != "" ]; then
/usr/share/biglinux/webapps/bin/xseticon -id "\$(xwininfo -root -children -all | grep -iE "Navigator.*$browser-webapp-$NAMEDESK" | awk '{print \$1}')" $ICON_FILE
        count=100
    else
        let count=count+1;
    fi
    sleep 0.5
done
EOF

chmod +x "$HOME/.local/bin/$NAMEDESK-$browser"

echo "#!/usr/bin/env xdg-open
[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$namedesk
Exec=$HOME/.local/bin/$NAMEDESK-$browser
Icon=$ICON_FILE
X-KDE-StartupNotify=true" > "/tmp/$NAMEDESK-$browser-webapp-biglinux-custom.desktop"

xdg-desktop-menu install --novendor $HOME/.local/share/desktop-directories/web-apps.directory \
"/tmp/$NAMEDESK-$browser-webapp-biglinux-custom.desktop"
rm "/tmp/$NAMEDESK-$browser-webapp-biglinux-custom.desktop"

    if [ "$shortcut" = "on" ];then
        ln "$HOME/.local/share/applications/$NAMEDESK-$browser-webapp-biglinux-custom.desktop" \
        "$(xdg-user-dir DESKTOP)/$namedesk.desktop"
        chmod 777 "$(xdg-user-dir DESKTOP)/$namedesk.desktop"
        gio set "$(xdg-user-dir DESKTOP)/$namedesk.desktop" -t string metadata::trust "true"
    fi

else

    if [ "$(egrep "(http|https)://" <<< "$urldesk")" != "" ];then

        if [ "$tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$urldesk")" != "" ];then
            urldesk="https://www.youtube.com/embed/$(basename "$urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
            CUT_HTTP=$(sed 's|https://||;s|http://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|' <<< "$urldesk")
        else
            CUT_HTTP=$(sed 's|https://||;s|http://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|' <<< "$urldesk")
        fi

        [ "$newperfil" = "on" ] && user="--user-data-dir=$HOME/.bigwebapps/$NAMEDESK-$browser" || user=
    else

        if [ "$tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$urldesk")" != "" ];then
            urldesk="https://www.youtube.com/embed/$(basename "$urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
            CUT_HTTP=$(sed 's|https://||;s|http://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|' <<< "$urldesk")
        else
            CUT_HTTP=$(sed 's|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|' <<< "$urldesk")
            urldesk="https://$urldesk"
        fi

        [ "$newperfil" = "on" ] && user="--user-data-dir=$HOME/.bigwebapps/$NAMEDESK-$browser" || user=
    fi

    if [ -z "$icondesk" -o "$icondesk" = "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png" ];then
        ICON_FILE="/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
    else
    	if [ "$(dirname "$icondesk")" = "/tmp" ];then
			mv "$icondesk" $HOME/.local/share/icons
		else
			cp "$icondesk" $HOME/.local/share/icons
		fi
		NAME_FILE=$(basename "$icondesk")
    	FILE_PNG=$(sed 's|\..*|.png|' <<< $NAME_FILE)
    	convert "$HOME/.local/share/icons/$NAME_FILE" -thumbnail 32x32 \
    			-alpha on -background none -flatten "$HOME/.local/share/icons/$browser-$NAMEDESK-$FILE_PNG"
    	rm "$HOME/.local/share/icons/$NAME_FILE"

        ICON_FILE="$HOME/.local/share/icons/$browser-$NAMEDESK-$FILE_PNG"
    fi

echo "#!/usr/bin/env xdg-open
[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$namedesk
Exec=$browser $user --class=\"$CUT_HTTP,Chromium-browser\" --profile-directory=Default --app=$urldesk
Icon=$ICON_FILE
StartupWMClass=$CUT_HTTP" > "/tmp/$NAMEDESK-$browser-webapp-biglinux-custom.desktop"

xdg-desktop-menu install --novendor $HOME/.local/share/desktop-directories/web-apps.directory \
"/tmp/$NAMEDESK-$browser-webapp-biglinux-custom.desktop"
rm "/tmp/$NAMEDESK-$browser-webapp-biglinux-custom.desktop"

    if [ "$shortcut" = "on" ];then
        ln "$HOME/.local/share/applications/$NAMEDESK-$browser-webapp-biglinux-custom.desktop" \
        "$(xdg-user-dir DESKTOP)/$namedesk.desktop"
        chmod 777 "$(xdg-user-dir DESKTOP)/$namedesk.desktop"
        gio set "$(xdg-user-dir DESKTOP)/$namedesk.desktop" -t string metadata::trust "true"
    fi
fi

nohup update-desktop-database -q $HOME/.local/share/applications &
nohup kbuildsycoca5 &> /dev/null &

kdialog --title "BigLinux WebApps" --icon "internet-web-browser" \
        --yesno $"O WebApp foi editado com sucesso!\nVocê deseja editar outro WebApp?"

if [ "$?" != "0" ]; then
    echo '<script>window.location.replace("index.sh.htm");</script>'
    exit
else
    echo '<script>window.location.replace("index-install.sh.htm");</script>'
    exit
fi
