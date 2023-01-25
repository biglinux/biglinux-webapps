#!/usr/bin/env bash

LOCAL_DIR=~/.local/share/applications/"$1"


case "$2" in
    firefox|org.mozilla.firefox|librewolf|io.gitlab.librewolf-community)
        if [ ! -e "$LOCAL_DIR" ];then
            cp "$PWD"/assets/"$2"/desk/"$1" ~/.local/share/applications
            cp "$PWD"/assets/"$2"/bin/"${1%%.*}-$2" ~/.local/bin
        else
            DESKBIN=~/.local/bin/$(sed -n '/^Exec/s/.*\/\([^\/]*\)$/\1/p' "$LOCAL_DIR")
            DATA_FOLDER=~/$(sed -n '/^FOLDER/s/.*=~\/\([^\n]*\).*/\1/p' "$DESKBIN")
            [ -d "$DATA_FOLDER" ] && rm -r "$DATA_FOLDER"
            rm "$DESKBIN" "$LOCAL_DIR"
        fi
    ;;

    *)  if [ ! -e "$LOCAL_DIR" ];then
            cp "$PWD"/webapps/"$1" ~/.local/share/applications
        else
            rm "$LOCAL_DIR"
        fi
    ;;
esac


update-desktop-database -q ~/.local/share/applications
nohup kbuildsycoca5 &>/dev/null &
exit
