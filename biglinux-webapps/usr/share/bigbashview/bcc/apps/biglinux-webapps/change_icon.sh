#!/usr/bin/env bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

cd "$(xdg-user-dir PICTURES)"
FILENAME="$(yad --file --image-filter                \
                --add-preview --large-preview        \
                --width=700 --height=500             \
                --center --title=$"Selecionar imagem" \
                --window-icon=image --skip-taskbar)"
echo -n "$FILENAME"
exit
