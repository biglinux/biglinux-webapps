#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

NAMEDESK="$(sed 'y/áÁàÀãÃâÂéÉêÊíÍóÓõÕôÔúÚüÜçÇ/aAaAaAaAeEeEiIoOoOoOuUuUcC/;
				 s|^ *||;s| *$||g;s| |-|g;s|/|-|g;
				 s|.*|\L&|' <<< "$p_namedesk")"

if [ "$p_browser" = "firefox" -o "$p_browser" = "waterfox-latest" ];then

    if [ "$(egrep "(http|https)://" <<< "$p_urldesk")" = "" ];then

        if [ "$p_tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$p_urldesk")" != "" ];then
            urldesk="https://www.youtube.com/embed/$(basename "$p_urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        else
            urldesk="https://$p_urldesk"
        fi

    else
        if [ "$p_tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$p_urldesk")" != "" ];then
            urldesk="https://www.youtube.com/embed/$(basename "$p_urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        else
            urldesk="$p_urldesk"
        fi
    fi

    CHECKURL=$(curl -o /dev/null --silent --head --write-out '%{http_code}' "$p_urldesk")

    if [ $CHECKURL -ge 400 -o $CHECKURL -eq 000 ];then
        kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --error $"Algo de errado aconteceu...\nPor favor, tente novamente!"
        echo '<script>window.location.replace("index-install.sh.htm");</script>'
        exit
    fi


    if [ -z "$p_icondesk" -o "$p_icondesk" = "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png" ];then
        ICON_FILE="/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
    else
    	if [ "$(dirname "$p_icondesk")" = "/tmp" ];then
			mv "$p_icondesk" $HOME/.local/share/icons
		else
			cp "$p_icondesk" $HOME/.local/share/icons
		fi
		NAME_FILE=$(basename "$p_icondesk")
    	FILE_PNG=$(sed 's|\..*|.png|' <<< $NAME_FILE)
    	convert $HOME/.local/share/icons/"$NAME_FILE" -thumbnail 32x32 \
    			-alpha on -background none -flatten $HOME/.local/share/icons/"$p_browser-$NAMEDESK-$FILE_PNG"
    	rm $HOME/.local/share/icons/"$NAME_FILE"

        ICON_FILE="$HOME/.local/share/icons/$p_browser-$NAMEDESK-$FILE_PNG"
    fi

cat > $HOME/.local/bin/"$NAMEDESK-$p_browser" <<EOF
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
#

if [ $(echo '"$(grep "toolkit.legacyUserProfileCustomizations.stylesheets" "$HOME/.bigwebapps/'$NAMEDESK-$p_browser'/prefs.js")" = ""') ]; then
    rm -R "\$HOME/.bigwebapps/$NAMEDESK-$p_browser"
    mkdir -p "\$HOME/.bigwebapps/$NAMEDESK-$p_browser/chrome"
    echo 'user_pref("media.eme.enabled", true);' >> "\$HOME/.bigwebapps/$NAMEDESK-$p_browser"/prefs.js
    echo 'user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);' >> "\$HOME/.bigwebapps/$NAMEDESK-$p_browser"/prefs.js
fi

#
# Custom profile
#
echo \
"#nav-bar {
    visibility: collapse;
}
#TabsToolbar {
    visibility: collapse;
}" \
>> "\$HOME/.bigwebapps/$NAMEDESK-$p_browser"/chrome/userChrome.css

echo \
"user_pref(\"browser.tabs.warnOnClose\", false);" \
>> "\$HOME/.bigwebapps/$NAMEDESK-$p_browser"/user.js

sed -i 's|user_pref("browser.urlbar.placeholderName.*||g' "\$HOME/.bigwebapps/$NAMEDESK-$p_browser"/prefs.js


MOZ_DISABLE_GMP_SANDBOX=1 MOZ_DISABLE_CONTENT_SANDBOX=1 $p_browser --class=$(echo "$p_browser"'webapp-'"$NAMEDESK") -profile "\$HOME/.bigwebapps/$NAMEDESK-$p_browser" -no-remote -new-instance "$urldesk" &

count=0
while [ \$count -lt 100 ]; do
    if [ $(echo '"$(xwininfo -root -children -all | grep -iE "Navigator.*'$p_browser'webapp-'$NAMEDESK'")" != ""') ]; then
        /usr/share/biglinux/webapps/bin/xseticon -id $(echo '"$(xwininfo -root -children -all | grep -iE "Navigator.*'$p_browser'webapp-'$NAMEDESK'" | awk '$(echo "'{print "'$1'"}'")')"') $ICON_FILE
        count=100
    else
        let count=count+1;
    fi
    sleep 0.5
done
EOF

chmod +x $HOME/.local/bin/"$NAMEDESK-$p_browser"

echo "#!/usr/bin/env xdg-open
[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$p_namedesk
Exec=$HOME/.local/bin/$NAMEDESK-$p_browser
Icon=$ICON_FILE
X-KDE-StartupNotify=true" > /tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop

xdg-desktop-menu install --novendor $HOME/.local/share/desktop-directories/web-apps.directory \
/tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop
rm /tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop

    if [ "$p_shortcut" = "on" ];then
        ln $HOME/.local/share/applications/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop \
        "$(xdg-user-dir DESKTOP)/$p_namedesk"
        chmod 755 "$(xdg-user-dir DESKTOP)/$p_namedesk"
    fi


elif [ "$p_browser" = "falkon" ]; then

	if [ "$(egrep "(http|https)://" <<< "$p_urldesk")" = "" ];then

        if [ "$p_tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$p_urldesk")" != "" ];then
            urldesk="https://www.youtube.com/embed/$(basename "$p_urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        else
            urldesk="https://$p_urldesk"
        fi
    else
        if [ "$p_tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$p_urldesk")" != "" ];then
            urldesk="https://www.youtube.com/embed/$(basename "$p_urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
        else
            urldesk="$p_urldesk"
        fi
    fi

    CHECKURL=$(curl -o /dev/null --silent --head --write-out '%{http_code}' "$p_urldesk")

    if [ $CHECKURL -ge 400 -o $CHECKURL -eq 000 ];then
        kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --error $"Algo de errado aconteceu...\nPor favor, tente novamente!"
        echo '<script>window.location.replace("index-install.sh.htm");</script>'
        exit
    fi

    mkdir -p $HOME/.config/falkon/profiles/"$NAMEDESK-$p_browser"
    cp /usr/share/biglinux/webapps/falkon/settings.ini $HOME/.config/falkon/profiles/"$NAMEDESK-$p_browser"

    if [ -z "$p_icondesk" -o "$p_icondesk" = "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png" ];then
        ICON_FILE="/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
    else
    	if [ "$(dirname "$p_icondesk")" = "/tmp" ];then
			mv "$p_icondesk" $HOME/.local/share/icons
		else
			cp "$p_icondesk" $HOME/.local/share/icons
		fi
		NAME_FILE=$(basename "$p_icondesk")
    	FILE_PNG=$(sed 's|\..*|.png|' <<< $NAME_FILE)
    	convert $HOME/.local/share/icons/"$NAME_FILE" -thumbnail 32x32 \
    			-alpha on -background none -flatten $HOME/.local/share/icons/"$p_browser-$NAMEDESK-$FILE_PNG"
    	rm $HOME/.local/share/icons/"$NAME_FILE"

        ICON_FILE="$HOME/.local/share/icons/$p_browser-$NAMEDESK-$FILE_PNG"
    fi

echo "#!/usr/bin/env xdg-open
[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$p_namedesk
Exec=falkon -p $NAMEDESK-$p_browser $urldesk
Icon=$ICON_FILE
X-KDE-StartupNotify=true" > /tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop

xdg-desktop-menu install --novendor $HOME/.local/share/desktop-directories/web-apps.directory \
/tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop
rm /tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop

    if [ "$p_shortcut" = "on" ];then
        ln $HOME/.local/share/applications/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop \
        "$(xdg-user-dir DESKTOP)/$p_namedesk"
        chmod 755 "$(xdg-user-dir DESKTOP)/$p_namedesk"
    fi

else

    if [ "$(egrep "(http|https)://" <<< "$p_urldesk")" != "" ];then

        if [ "$p_tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$p_urldesk")" != "" ];then
            p_urldesk="https://www.youtube.com/embed/$(basename "$p_urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
            CUT_HTTP=$(sed 's|https://||;s|http://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|' <<< "$p_urldesk")
        else
            CUT_HTTP=$(sed 's|https://||;s|http://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|' <<< "$p_urldesk")
        fi

        [ "$p_newperfil" = "on" ] && user="--user-data-dir=$HOME/.bigwebapps/$NAMEDESK-$p_browser" || user=
    else

        if [ "$p_tvmode" = "on" -a "$(egrep "(youtu.be|youtube)" <<< "$p_urldesk")" != "" ];then
            p_urldesk="https://www.youtube.com/embed/$(basename "$p_urldesk" | sed 's|watch?v=||;s|&list=.*||;s|&feature=.*||')"
            CUT_HTTP=$(sed 's|https://||;s|http://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|' <<< "$p_urldesk")
        else
            CUT_HTTP=$(sed 's|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|' <<< "$p_urldesk")
            p_urldesk="https://$p_urldesk"
        fi

        [ "$p_newperfil" = "on" ] && user="--user-data-dir=$HOME/.bigwebapps/$NAMEDESK-$p_browser" || user=
    fi

    CHECKURL=$(curl -o /dev/null --silent --head --write-out '%{http_code}' "$p_urldesk")

    if [ $CHECKURL -ge 400 -o $CHECKURL -eq 000 ];then
        kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --error $"Algo de errado aconteceu...\nPor favor, tente novamente!"
        echo '<script>window.location.replace("index-install.sh.htm");</script>'
        exit
    fi

    if [ -z "$p_icondesk" -o "$p_icondesk" = "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png" ];then
        ICON_FILE="/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
    else
    	if [ "$(dirname "$p_icondesk")" = "/tmp" ];then
			mv "$p_icondesk" $HOME/.local/share/icons
		else
			cp "$p_icondesk" $HOME/.local/share/icons
		fi
		NAME_FILE=$(basename "$p_icondesk")
    	FILE_PNG=$(sed 's|\..*|.png|' <<< $NAME_FILE)
    	convert $HOME/.local/share/icons/"$NAME_FILE" -thumbnail 32x32 \
    			-alpha on -background none -flatten $HOME/.local/share/icons/"$p_browser-$NAMEDESK-$FILE_PNG"
    	rm $HOME/.local/share/icons/"$NAME_FILE"

        ICON_FILE="$HOME/.local/share/icons/$p_browser-$NAMEDESK-$FILE_PNG"
    fi

echo "#!/usr/bin/env xdg-open
[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$p_namedesk
Exec=$p_browser $user --class=\"$CUT_HTTP,Chromium-browser\" --profile-directory=Default --app=$p_urldesk
Icon=$ICON_FILE
StartupWMClass=$CUT_HTTP" > /tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop

xdg-desktop-menu install --novendor $HOME/.local/share/desktop-directories/web-apps.directory \
/tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop
rm /tmp/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop

    if [ "$p_shortcut" = "on" ];then
        ln $HOME/.local/share/applications/"$NAMEDESK-$p_browser"-webapp-biglinux-custom.desktop \
        "$(xdg-user-dir DESKTOP)/$p_namedesk"
        chmod 755 "$(xdg-user-dir DESKTOP)/$p_namedesk"
    fi
fi

if [ "$?" = "0" ]; then

    nohup update-desktop-database -q $HOME/.local/share/applications &
    nohup kbuildsycoca5 &> /dev/null &

    kdialog --title "BigLinux WebApps" --icon "internet-web-browser" \
            --yesno $"O WebApp foi instalado com sucesso!\nVocê deseja instalar outro WebApp?"

    if [ "$?" != "0" ]; then
        echo '<script>window.location.replace("index.sh.htm");</script>'
        exit
    else
        echo '<script>window.location.replace("index-install.sh.htm");</script>'
        exit
    fi
else
    kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --error $"Algo de errado aconteceu...\nPor favor, tente novamente!"
    echo '<script>window.location.replace("index-install.sh.htm");</script>'
    exit
fi
