#!/usr/bin/env bash

FILE=~/.local/share/applications/"$1"
if grep -q '.local.bin' "$FILE";then
    EXEC=$(grep -E -m1 '^Exec' "$FILE"|sed "s|^Exec=||;s|~|$HOME|")
    "$EXEC"
else
    gtk-launch "$1"
fi
