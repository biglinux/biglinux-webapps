#!/usr/bin/env bash

ICONDESK=$(awk -F'=' '/Icon/{print $2}' "$filedesk")
LINK=$(xdg-user-dir DESKTOP)/"${filedesk##*/}"

if grep -q '..no.first.run' "$filedesk";then
    DATA_DIR=$(awk '/Exec/{sub(/--user-data-dir=/,"");print $2}' "$filedesk")
    [ -d "$DATA_DIR" ] && rm -r "$DATA_DIR"
fi

if grep -q '..profile=' "$filedesk";then
    EPI_DATA=$(awk '/Exec/{sub(/--profile=/,"");print $3}' "$filedesk")
    DIR_PORTAL_APP=~/.local/share/xdg-desktop-portal/applications
    DIR_PORTAL_FILEDESK="$DIR_PORTAL_APP/${filedesk##*/}"
    [ -e "$DIR_PORTAL_FILEDESK" ] && rm "$DIR_PORTAL_FILEDESK"
    rm -r "$EPI_DATA"
fi

if grep -q '.local.bin' "$filedesk";then
    DESKBIN=~/.local/bin/$(sed -n '/^Exec/s/.*\/\([^\/]*\)$/\1/p' "$filedesk")
    DATA_FOLDER=$(sed -n '/^FOLDER/s/.*=\([^\n]*\).*/\1/p' "$DESKBIN")
    rm "$DESKBIN"
    rm -r "$DATA_FOLDER"
fi

if [ -L "$LINK" -o -e "$LINK" ];then
    unlink "$LINK"
fi

if [ -n "$(grep 'falkon' "$filedesk")" ];then
    folder=$(awk '/Exec=/{print $3}' "$filedesk")
    rm -r ${HOME}/.config/falkon/profiles/${folder}
fi

[ -e "$ICONDESK" ] && rm "$ICONDESK"
rm "$filedesk"

nohup update-desktop-database -q ~/.local/share/applications &
nohup kbuildsycoca5 &> /dev/null &
exit
