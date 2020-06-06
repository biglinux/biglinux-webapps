#!/bin/bash



#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

NAMEDESK="$(grep "Name=" $p_filedesk | sed 's|Name=||' |\
           sed 'y/áÁàÀãÃâÂéÉêÊíÍóÓõÕôÔúÚüÜçÇ/aAaAaAaAeEeEiIoOoOoOuUuUcC/' |\
           tr '[:upper:]' '[:lower:]' |\
           sed 's|\ |-|g;s|\/|-|g')"

ICONDESK="$(grep "Icon=" $p_filedesk | sed 's|Icon=||')"
DESKNAME="$(grep "Name=" $p_filedesk | sed 's|Name=||')"

kdialog --title "BigLinux WebApps" --icon "internet-web-browser" \
		--yesno $"Você tem certeza que deseja remover este WebApp?"

if [ "$?" != "0" ]; then
    echo "<META http-equiv=\"refresh\" content=\"0;URL=index.sh.htm\">"
    exit
else

    if [ "$(grep "firefox" <<< "$p_filedesk")" != "" ];then

        if [ -d $HOME/.bigwebapps/"$NAMEDESK-firefox" ]; then
            rm -r $HOME/.bigwebapps/"$NAMEDESK-firefox"
        fi
        unlink "$(xdg-user-dir DESKTOP)/$DESKNAME"
        rm "$(grep "Exec=" "$p_filedesk" | sed 's|Exec=||')"
        xdg-desktop-menu uninstall "$p_filedesk"
        rm "$ICONDESK"

    elif [ "$(grep "waterfox-latest" <<< "$p_filedesk")" != "" ];then

        if [ -d $HOME/.bigwebapps/"$NAMEDESK-waterfox-latest" ]; then
            rm -r $HOME/.bigwebapps/"$NAMEDESK-waterfox-latest"
        fi
        unlink "$(xdg-user-dir DESKTOP)/$DESKNAME"
        rm "$(grep "Exec=" "$p_filedesk" | sed 's|Exec=||')"
        xdg-desktop-menu uninstall "$p_filedesk"
        rm "$ICONDESK"

    elif [ "$(grep "falkon" <<< "$p_filedesk")" != "" ];then

        if [ -d $HOME/.config/falkon/profiles/"$NAMEDESK" ]; then

            rm -r $HOME/.config/falkon/profiles/"$NAMEDESK"
        fi

        unlink "$(xdg-user-dir DESKTOP)/$DESKNAME"
        xdg-desktop-menu uninstall "$p_filedesk"
        rm "$ICONDESK"

    else
        unlink "$(xdg-user-dir DESKTOP)/$DESKNAME"
        xdg-desktop-menu uninstall "$p_filedesk"
        rm "$ICONDESK"
    fi

    kdialog --title "BigLinux WebApps" --icon "internet-web-browser" \
    --yesno $"O WebApp foi removido com sucesso!\nVocê deseja remover outro WebApp?"

    if [ "$?" != "0" ]; then
        echo "<META http-equiv=\"refresh\" content=\"0;URL=index.sh.htm\">"
        exit
    else
        echo "<META http-equiv=\"refresh\" content=\"0;URL=index-remove.sh.htm\">"
        exit
    fi
fi
