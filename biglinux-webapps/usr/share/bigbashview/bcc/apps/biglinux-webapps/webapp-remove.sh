#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps


ICON="$(grep "Icon=" $p_filedesk | sed 's|Icon=||')"
if [ "$(grep ".svg" <<< $ICON)" = "" ]; then
    ICON="$ICON.svg"
fi

kdialog --yesno $"\nVocê tem certeza que deseja remover este WebApp? \n"

if [ "$?" != "0" ]; then
    echo "<META http-equiv=\"refresh\" content=\"0;URL=index-remove.sh.htm\">"
    exit
else
    xdg-desktop-menu uninstall "$p_filedesk"
    rm "$ICON"

    kdialog --msgbox $"\nO WebApp personalizado foi removido com sucesso! \n"

    kdialog --yesno $"\nVocê deseja remover outro WebApp personalizado? \n"

    if [ "$?" != "0" ]; then
        echo "<META http-equiv=\"refresh\" content=\"0;URL=index.sh.htm\">"
        exit
    else
        echo "<META http-equiv=\"refresh\" content=\"0;URL=index-remove.sh.htm\">"
        exit
    fi
fi
