#!/bin/bash

# The output of this script is the path of the icon selected by the user

# Persist last-used directory
LAST_DIR_FILE="$HOME/.config/biglinux-webapps/last_icon_dir"
last_dir=""
if [[ -f "$LAST_DIR_FILE" ]]; then
    last_dir=$(<"$LAST_DIR_FILE")
fi

# Use type to check if a command exists
if type kdialog >/dev/null 2>&1; then
    icon=$(kdialog --geticon Applications 2> /dev/null)
elif type zenity >/dev/null 2>&1; then
    icon=$(zenity --file-selection ${last_dir:+--filename="$last_dir/"} --file-filter="image|*.[Jj][Pp][Gg] *.[Jj][Pp][Ee][Gg] *.[Pp][Nn][Gg] *.[Ss][Vv][Gg] *.[Ss][Vv][Gg][Zz] *.[Ww][Ee][Bb][Pp]")
elif type yad >/dev/null 2>&1; then
    icon=$(cd "${last_dir:-$HOME}"; yad --file --add-preview --large-preview --file-filter="image|*.[Jj][Pp][Gg] *.[Jj][Pp][Ee][Gg] *.[Pp][Nn][Gg] *.[Ss][Vv][Gg] *.[Ss][Vv][Gg][Zz] *.[Ww][Ee][Bb][Pp]")
fi

# Save chosen directory for next time
if [[ $icon =~ / ]]; then
    mkdir -p "$(dirname "$LAST_DIR_FILE")"
    dirname "$icon" > "$LAST_DIR_FILE"
fi

# If icon don't have a path, get the icon with path
if [[ $icon =~ / ]]; then
    echo "$icon"
elif [[ -n "$icon" ]]; then
    for sz in 128 64 48 32; do
        icon_address=$(geticons -s "$sz" "$icon" 2>/dev/null)
        if [[ -n "$icon_address" ]]; then
            echo "$icon_address"
            exit
        fi
    done
    geticons "$icon" 2>/dev/null
fi
