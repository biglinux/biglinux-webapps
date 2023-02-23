#!/usr/bin/env bash

WEBAPPS=($(find ~/.local/share/applications -iname "*-webapp-biglinux-custom.desktop"))

if [ -z "$WEBAPPS" ];then
    exit
fi

TMP_DIR="/tmp/backup-webapps"
TMP_DIR_ICON="$TMP_DIR/icons"
TMP_DIR_BIN="$TMP_DIR/bin"

mkdir -p "$TMP_DIR"/{icons,bin}

for w in "${WEBAPPS[@]}";do
    if grep -q '.local.bin' "$w";then
        BIN=$(awk -F'=' '/^Exec/{print $2}' "$w")
        cp "$BIN" "$TMP_DIR_BIN"
    fi

    ICON=$(awk -F'=' '/^Icon/{print $2}' "$w")
    cp "$ICON" "$TMP_DIR_ICON"

    cp "$w" "$TMP_DIR"
done

NAME_FILE="backup-webapps_$(date +%Y-%m-%d).tar.gz"
SAVE_DIR=$(kdialog --getexistingdirectory ~ 2>/dev/null)

if [ ! "$SAVE_DIR" ];then
    exit
fi

cd /tmp
tar -czf "${SAVE_DIR}/${NAME_FILE}" backup-webapps
rm -r backup-webapps

printf "%s/%s" "${SAVE_DIR}" "${NAME_FILE}"
exit
