#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

kdialog --title "BigLinux WebApps" --icon "internet-web-browser" \
        --yesno $"Você tem certeza que deseja remover este WebApp?"

if [ "$?" != "0" ]; then
    exit
else
    NAMEDESK="$(basename -s .desktop "$p_filedesk" | sed 's|-webapp-biglinux-custom||')"
    ICONDESK="$(grep "Icon=" $p_filedesk | sed 's|Icon=||')"
    DESKNAME="$(grep "Name=" $p_filedesk | sed 's|Name=||')"

    if [ "$(grep "firefox$" $p_filedesk)" != "" ];then

        if [ -d $HOME/.bigwebapps/"$NAMEDESK" ]; then
            rm -r $HOME/.bigwebapps/"$NAMEDESK"
        fi
        unlink "$(xdg-user-dir DESKTOP)/$DESKNAME" &> /dev/null
        rm "$(grep "Exec=" "$p_filedesk" | sed 's|Exec=||')"
        xdg-desktop-menu uninstall "$p_filedesk"
        rm "$ICONDESK"
    else
        if [ -d $HOME/.bigwebapps/"$NAMEDESK" ]; then
            rm -r $HOME/.bigwebapps/"$NAMEDESK"
        fi
        unlink "$(xdg-user-dir DESKTOP)/$DESKNAME" &> /dev/null
        xdg-desktop-menu uninstall "$p_filedesk"
        rm "$ICONDESK"
    fi

    nohup update-desktop-database -q $HOME/.local/share/applications &
    nohup kbuildsycoca5 &> /dev/null &
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
