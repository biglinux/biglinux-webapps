#!/usr/bin/env bash

ICONDESK=$(awk -F'=' '/Icon/{print $2}' "$filedesk")
LINK=$(xdg-user-dir DESKTOP)/"${filedesk##*/}"

if grep -q '..no.first.run' "$filedesk";then
    DATA_DIR=$(awk '/Exec/{sub(/--user-data-dir=/,"");print $2;exit}' "$filedesk")
    yad --text "$DATA_DIR"
    exit
    [ -d "$DATA_DIR" ] && rm -r "$DATA_DIR"
fi

if grep -q '..profile=' "$filedesk";then
    EPI_DATA=$(awk '/Exec/{sub(/--profile=/,"");print $3;exit}' "$filedesk")
    DIR_PORTAL_APP=~/.local/share/xdg-desktop-portal/applications
    DIR_PORTAL_FILEDESK="$DIR_PORTAL_APP/${filedesk##*/}"
    yad --text "$EPI_DATA\n$DIR_PORTAL_APP\n$DIR_PORTAL_FILEDESK"
    exit
    [ -e "$DIR_PORTAL_FILEDESK" ] && rm "$DIR_PORTAL_FILEDESK"
    rm -r "$EPI_DATA"
fi

if grep -q '.local.bin' "$filedesk";then
    DESKBIN=$(awk -F'=' '/^Exec/{print $2;exit}' "$filedesk")
    DATA_FOLDER=$(awk -F'=' '/^FOLDER/{print $2}' "$DESKBIN")
    yad --text "$DESKBIN\n$DATA_FOLDER"
    exit
    rm "$DESKBIN"
    rm -r "$DATA_FOLDER"
fi

if [ -L "$LINK" -o -e "$LINK" ];then
    unlink "$LINK"
fi

if [ -n "$(grep 'falkon' "$filedesk")" ];then
    folder=$(awk '/Exec=/{print $3;exit}' "$filedesk")
    yad --text "$folder"
    exit
    rm -r "${HOME}/.config/falkon/profiles/${folder}"
fi

[ -e "$ICONDESK" ] && rm "$ICONDESK"
rm "$filedesk"

nohup update-desktop-database -q ~/.local/share/applications &
nohup kbuildsycoca5 &> /dev/null &
exit
