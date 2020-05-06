#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps


NAMEDESK=$(echo "$p_namedesk" |\
           sed 'y/áÁàÀãÃâÂéÉêÊíÍóÓõÕôÔúÚüÜçÇ/aAaAaAaAeEeEiIoOoOoOuUuUcC/' |\
           tr '[:upper:]' '[:lower:]' |\
           sed 's|\ |-|g;s|\/|-|g')

if [ "$(echo "$p_urldesk" | egrep "(http|https)://")" != "" ];then

	CUT_HTTP=$(echo "$p_urldesk" |\
                     sed 's/https:\/\///;s/http:\/\///' |\
                     tr '/' '_' |\
                     sed 's/_/__/1;s/_$//;s/_$//')
else

	kdialog --error $"\nA url inserida é inválida! \n Por favor, verifique sua url e tente novamente! \n"
	echo '<script>window.location.replace("index-install.sh.htm?namedesk='"$p_namedesk"'&urldesk=https://'"$p_urldesk"'&icondesk='"$p_icondesk"'");</script>'
	exit
fi

ICONFILE=$(echo "$p_icondesk" | awk -F'/' '{print $NF}')
if [ -z "$ICONFILE" ]; then
	ICON_FILE="internet-web-browser"
else
    if [ "$(grep ".svg" <<< $ICONFILE)" != "" ]; then
        ICONFILE="$(sed 's|.svg||' <<< $ICONFILE)"
    fi
	cp -u "$p_icondesk" "$HOME/.local/share/icons"
	ICON_FILE="$HOME/.local/share/icons/$ICONFILE"
    
fi

echo "#!/usr/bin/env xdg-open
[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$p_namedesk
Exec=/usr/bin/biglinux-webapp --class=\"$CUT_HTTP,Chromium-browser\" --profile-directory=Default --app=$p_urldesk
Icon=$ICON_FILE
StartupWMClass=$CUT_HTTP" > /tmp/"$NAMEDESK"-webapp-biglinux-custom.desktop

xdg-desktop-menu install --novendor $HOME/.local/share/desktop-directories/web-apps.directory \
/tmp/"$NAMEDESK"-webapp-biglinux-custom.desktop
rm /tmp/"$NAMEDESK"-webapp-biglinux-custom.desktop

if [ "$?" = "0" ]; then
    kdialog --msgbox $"\nO WebApp personalizado foi criado com sucesso! \n"
    kdialog --yesno $"\nVocê deseja criar outro WebApp personalizado? \n"

    if [ "$?" != "0" ]; then
        echo '<script>window.location.replace("index.sh.htm");</script>'
        exit
    else
        echo '<script>window.location.replace("index-install.sh.htm");</script>'
        exit
    fi
else
    kdialog --error $"\nAlgo de errado aconteceu... \nPor favor, tente novamente! \n"
    echo '<script>window.location.replace("index-install.sh.htm");</script>'
    exit
fi
