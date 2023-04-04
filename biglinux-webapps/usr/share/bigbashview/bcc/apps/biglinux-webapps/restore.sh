#!/usr/bin/env bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

PROG_DIR="/usr/share/bigbashview/bcc/apps/biglinux-webapps"
cd ~
FILENAME=$(yad --file --window-icon="$PROG_DIR/icons/webapp.svg"   \
               --title=$"Selecionar o arquivo de backup" \
               --width=900 --height=600 --center         \
               --mime-filter=$"Backup WebApps""|application/gzip")

if [ ! "$FILENAME" ];then
    exit
fi

tar -xzf "$FILENAME" -C /tmp

TMP_FOLDER=/tmp/backup-webapps
FLATPAK_FOLDER_DATA=~/.var/app

if [ -d "$TMP_FOLDER" ];then
    cp -a "$TMP_FOLDER"/*.desktop ~/.local/share/applications
    cp -a "$TMP_FOLDER"/icons/* ~/.local/share/icons

    if [ -d "$TMP_FOLDER"/bin ];then
        cp -a "$TMP_FOLDER"/bin/* ~/.local/bin
    fi

    if [ -d "$TMP_FOLDER"/data ];then
        cp -a "$TMP_FOLDER"/data/* ~/.bigwebapps
    fi

    if [ -d "$TMP_FOLDER"/epiphany ];then
        cp -a "$TMP_FOLDER"/epiphany/data "$FLATPAK_FOLDER_DATA"/org.gnome.Epiphany
        cp -a "$TMP_FOLDER"/epiphany/xdg-desktop-portal ~/.local/share
        ln -s ~/.local/share/xdg-desktop-portal/applications/*.desktop ~/.local/share/applications
    fi

    if [ -d "$TMP_FOLDER"/flatpak ];then
        cp -a "$TMP_FOLDER"/flatpak/* "$FLATPAK_FOLDER_DATA"
    fi

    if [ -d "$TMP_FOLDER"/desktop ];then
        cp -a "$TMP_FOLDER"/desktop/* "$(xdg-user-dir DESKTOP)"
    fi

    rm -r "$TMP_FOLDER"

    update-desktop-database -q ~/.local/share/applications
    nohup kbuildsycoca5 &>/dev/null &

    printf 0
    exit
fi
