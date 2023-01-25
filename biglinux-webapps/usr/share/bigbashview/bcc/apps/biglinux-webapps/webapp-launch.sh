#!/usr/bin/env bash

FILE=~/.local/share/applications/"$1"
if grep -q '.local.bin' "$FILE";then
    EXEC=~/$(sed -n '/^Exec/s/.*=~\/\([^\n]*\).*/\1/p' "$FILE")
    "${EXEC}"
else
    gtk-launch "$1"
fi
