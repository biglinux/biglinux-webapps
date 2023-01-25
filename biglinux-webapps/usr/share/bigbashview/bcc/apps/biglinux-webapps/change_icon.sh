#!/usr/bin/env bash

FILENAME=$(kdialog --getopenfilename "$(xdg-user-dir PICTURES)" 'image/bmp image/jpeg image/png image/x-icon' 2>/dev/null)

if [ ! "$FILENAME" ];then
    exit
fi

NEW_FILE=$(./resize_favicon.sh.py "$FILENAME")
printf '%s' "$NEW_FILE"
exit
