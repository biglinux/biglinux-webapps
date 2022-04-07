#!/usr/bin/env bash

if [ ! -e "$HOME/.local/share/applications/$1" ]; then
    cp /usr/share/bigbashview/bcc/apps/biglinux-webapps/webapps/$1 $HOME/.local/share/applications
    update-desktop-database -q $HOME/.local/share/applications
    kbuildsycoca5 &> /dev/null
    exit
else
    rm "$HOME/.local/share/applications/$1"
    update-desktop-database -q $HOME/.local/share/applications
    kbuildsycoca5 &> /dev/null
    exit
fi
