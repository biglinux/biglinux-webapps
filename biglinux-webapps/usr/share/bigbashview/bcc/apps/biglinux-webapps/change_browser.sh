#!/usr/bin/env bash

CHANGE=false
mapfile -t FILES < <(find ~/.local/share/applications -iname "*-webapp-biglinux.desktop")

# shellcheck disable=SC2128
if [ ! "${FILES}" ]; then
    printf "%s" "$2" >~/.bigwebapps/BROWSER
    exit 0
fi

blink_to_gecko() {
    for w in "${FILES[@]}"; do
        filename="${w##*/}"
        cp -f "${PWD}/assets/${1}/bin/${filename%%.*}-$1" ~/.local/bin/
        cp -f "${PWD}/assets/${1}/desk/${filename}" ~/.local/share/applications/
    done
}

gecko_to_blink() {
    for w in "${FILES[@]}"; do
        cp -f "$PWD"/webapps/"${w##*/}" ~/.local/share/applications
    done
}

clear_files() {
    for w in "${FILES[@]}"; do
        EXEC_BIN=~/.local/bin/$(sed -n '/^Exec/s/.*\/\([^\/]*\)$/\1/p' "$w")
        DATA_DIR=~/$(sed -n '/^FOLDER/s/.*=~\/\([^\n]*\).*/\1/p' "$EXEC_BIN")

        [ -d "$DATA_DIR" ] && rm -r "$DATA_DIR"
        [ -e "$EXEC_BIN" ] && rm "$EXEC_BIN"
    done
}

case "$2" in
*firefox* | *firedragon | *librewolf*)
    blink_to_gecko "$2"
    CHANGE=true
    ;;
*) notgecko=true ;;
esac

case "$1" in
*firefox* | *firedragon | *librewolf*)
    clear_files
    [ "$notgecko" ] && gecko_to_blink
    CHANGE=true
    ;;
esac

if [ "$CHANGE" = "true" ]; then
    update-desktop-database -q ~/.local/share/applications
    nohup kbuildsycoca5 &>/dev/null &
fi

printf "%s" "$2" >~/.bigwebapps/BROWSER
exit 0
