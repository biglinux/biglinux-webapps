#!/usr/bin/env bash

WEBAPPS=($(find ~/.local/share/applications -iname "*-webapp-biglinux-custom.desktop"))

if [ -z "$WEBAPPS" ];then
    exit
fi

TMP_DIR="/tmp/backup-webapps"
TMP_DIR_BIN="$TMP_DIR/bin"
TMP_DIR_DATA="$TMP_DIR/data"
TMP_DIR_ICON="$TMP_DIR/icons"
TMP_DIR_EPIHANY="$TMP_DIR/epiphany"
TMP_DIR_PORTAL="$TMP_DIR_EPIHANY/xdg-desktop-portal"
TMP_DIR_EPIHANY_DATA="$TMP_DIR_EPIHANY/data"

for w in "${WEBAPPS[@]}";do
    if grep -q '.local.bin' "$w";then
        mkdir -p "$TMP_DIR_BIN"
        BIN=$(awk -F'=' '/^Exec/{print $2}' "$w")
        cp "$BIN" "$TMP_DIR_BIN"
        DATA_FOLDER=$(sed -n '/^FOLDER/s/.*=\([^\n]*\).*/\1/p' "$BIN")
        if grep -q '.bigwebapps' <<< "$DATA_FOLDER";then
            mkdir -p "$TMP_DIR_DATA"
            cp -r "$DATA_FOLDER" "$TMP_DIR_DATA"
        else
            DATA_FOLDER_COPY="$TMP_DIR/flatpak/$(awk -F'/' '{print $6"/"$7}' <<< "$DATA_FOLDER")"
            mkdir -p "$DATA_FOLDER_COPY"
            cp -r "$DATA_FOLDER" "$DATA_FOLDER_COPY"
        fi
    fi

    if grep -q '..no.first.run' "$w";then
        DATA_DIR=$(awk '/Exec/{sub(/--user-data-dir=/,"");print $2}' "$w")
        if grep -q '.bigwebapps' <<< "$DATA_DIR";then
            mkdir -p "$TMP_DIR_DATA"
            cp -r "$DATA_DIR" "$TMP_DIR_DATA"
        else
            DATA_DIR_COPY="$TMP_DIR/flatpak/$(awk -F'/' '{print $6"/"$7}' <<< "$DATA_DIR")"
            mkdir -p "$DATA_DIR_COPY"
            cp -r "$DATA_DIR" "$DATA_DIR_COPY"
        fi
    fi

    if grep -q '..profile=' "$w";then
        mkdir -p "$TMP_DIR_PORTAL"
        cp -r ~/.local/share/xdg-desktop-portal/* "$TMP_DIR_PORTAL"
        EPI_DATA=$(awk '/Exec/{sub(/--profile=/,"");print $3}' "$w")
        mkdir -p "$TMP_DIR_EPIHANY_DATA"
        cp -r "$EPI_DATA" "$TMP_DIR_EPIHANY_DATA"
    else
        mkdir -p "$TMP_DIR_ICON"
        ICON=$(awk -F'=' '/^Icon/{print $2}' "$w")
        cp "$ICON" "$TMP_DIR_ICON"
        cp "$w" "$TMP_DIR"
    fi
done

NAME_FILE="backup-webapps_$(date +%Y-%m-%d).tar.gz"
SAVE_DIR=$(kdialog --getexistingdirectory ~ 2>/dev/null)

if [ ! "$SAVE_DIR" ];then
    exit
fi

cd /tmp
tar -czf "${SAVE_DIR}/${NAME_FILE}" backup-webapps
rm -r backup-webapps

printf "${SAVE_DIR}/${NAME_FILE}"
exit
