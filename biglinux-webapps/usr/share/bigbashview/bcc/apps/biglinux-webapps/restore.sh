#!/usr/bin/env bash

FILENAME=$(kdialog --getopenfilename ~ 'application/gzip' 2>/dev/null)

if [ ! "$FILENAME" ];then
    exit
fi

tar -xzf "$FILENAME" -C /tmp

if [ -d /tmp/backup-webapps ];then
    cp -f /tmp/backup-webapps/*.desktop ~/.local/share/applications
    cp -f /tmp/backup-webapps/icons/* ~/.local/share/icons
    cp -f /tmp/backup-webapps/bin/* ~/.local/bin

    rm -r /tmp/backup-webapps

    update-desktop-database -q ~/.local/share/applications
    nohup kbuildsycoca5 &>/dev/null &
fi

printf 0
exit
