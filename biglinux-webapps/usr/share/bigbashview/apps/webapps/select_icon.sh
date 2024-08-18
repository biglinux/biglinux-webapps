#!/bin/bash

# The output of this script is the path of the icon selected by the user

# Use type to check if a command exists
if type kdialog >/dev/null; then
    icon=$(kdialog --geticon Applications 2> /dev/null)
elif type zenity >/dev/null; then
    icon=$(zenity --file-selection --file-filter="image|*.[Jj][Pp][Gg] *.[Jj][Pp][Ee][Gg] *.[Pp][Nn][Gg] *.[Ss][Vv][Gg] *.[Ss][Vv][Gg][Zz] *.[Ww][Ee][Bb][Pp]")
elif type yad >/dev/null; then
    icon=$(cd ~; yad --file --add-preview --large-preview --file-filter="image|*.[Jj][Pp][Gg] *.[Jj][Pp][Ee][Gg] *.[Pp][Nn][Gg] *.[Ss][Vv][Gg] *.[Ss][Vv][Gg][Zz] *.[Ww][Ee][Bb][Pp]")
fi

# If icon don't have a path, get the icon with path
if [[ $icon =~ / ]]; then
    echo "$icon"
else
    icon_address=$(geticons -s 128 "$icon")
    [ -n "$icon_address" ] && echo "$icon_address"; exit
    icon_address=$(geticons -s 64 "$icon")
    [ -n "$icon_address" ] && echo "$icon_address"; exit
    icon_address=$(geticons -s 48 "$icon")
    [ -n "$icon_address" ] && echo "$icon_address"; exit
    icon_address=$(geticons -s 32 "$icon")
    [ -n "$icon_address" ] && echo "$icon_address"; exit
    geticons "$icon"
fi
