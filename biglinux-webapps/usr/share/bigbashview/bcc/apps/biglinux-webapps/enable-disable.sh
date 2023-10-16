#!/usr/bin/env bash

LOCAL_DIR=~/.local/share/applications/"$1"
BASEDIR=/usr/share/bigbashview/bcc/apps/biglinux-webapps

case "$2" in
    firefox|org.mozilla.firefox|librewolf|io.gitlab.librewolf-community)
        if [ ! -e "$LOCAL_DIR" ];then
            cp "$BASEDIR"/assets/"$2"/desk/"$1" ~/.local/share/applications
            cp "$BASEDIR"/assets/"$2"/bin/"${1%%.*}-$2" ~/.local/bin
        else
            rm "$LOCAL_DIR"
        fi
    ;;

    *)  if [ ! -e "$LOCAL_DIR" ];then
            cp "$BASEDIR"/webapps/"$1" ~/.local/share/applications
        else
            rm "$LOCAL_DIR"
        fi
    ;;
esac


update-desktop-database -q ~/.local/share/applications
nohup kbuildsycoca5 &>/dev/null &
exit
