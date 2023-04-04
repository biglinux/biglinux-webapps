#!/usr/bin/env bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

PROG_DIR="/usr/share/bigbashview/bcc/apps/biglinux-webapps"
cd "$(xdg-user-dir PICTURES)"
FILENAME=$(yad --file --center --width=900 --height=600   \
               --window-icon="$PROG_DIR/icons/webapp.svg" \
               --title $"Selecione o arquivo de imagem"   \
               --mime-filter=$"Arquivos de Imagem""|image/bmp image/jpeg image/png image/x-icon")

if [ ! "$FILENAME" ];then
    exit
fi

NEW_FILE=$("$PROG_DIR"/resize_favicon.sh.py "$FILENAME")
printf "$NEW_FILE"
exit
