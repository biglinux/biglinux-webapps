#!/usr/bin/env bash
#shellcheck disable=SC2155,SC2034
#shellcheck source=/dev/null

#  /usr/share/bigbashview/bcc/apps/biglinux-webapps/backup.sh
#  Description: WebApps installing programs for BigLinux
#
#  Created: 2020/01/11
#  Altered: 2024/06/01
#
#  Copyright (c) 2023-2024, Vilmar Catafesta <vcatafesta@gmail.com>
#                2022-2023, Bruno Gonçalves <www.biglinux.com.br>
#                2022-2023, Rafael Ruscher <rruscher@gmail.com>
#                2020-2023, eltonff <www.biglinux.com.br>
#  All rights reserved.
#
#  Redistribution and use in source and binary forms, with or without
#  modification, are permitted provided that the following conditions
#  are met:
#  1. Redistributions of source code must retain the above copyright
#     notice, this list of conditions and the following disclaimer.
#  2. Redistributions in binary form must reproduce the above copyright
#     notice, this list of conditions and the following disclaimer in the
#     documentation and/or other materials provided with the distribution.
#
#  THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
#  IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
#  OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
#  IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT,
#  INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
#  NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
#  DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
#  THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
#  (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
#  THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

APP="${0##*/}"
_VERSION_="1.0.0-20240601"
LIBRARY=${LIBRARY:-'/usr/share/bigbashview/bcc/shell'}
[[ -f "${LIBRARY}/bcclib.sh" ]] && source "${LIBRARY}/bcclib.sh"
[[ -f "${LIBRARY}/tinilib.sh" ]] && source "${LIBRARY}/tinilib.sh"
[[ -f "${LIBRARY}/weblib.sh" ]] && source "${LIBRARY}/weblib.sh"

function sh_webapp_backup_main() {
	local SUBTITLE="$(gettext $"Não existem WebApps instalados para backup!")"
	local SAVE_DIR_TITLE="$(gettext $"Selecione o diretório para salvar:")"
	local backup_file="backup-webapps_$(date +%Y-%m-%d).tar.gz"
	local TMP_DIR="$TMP_FOLDER/backup-webapps"
	local TMP_DIR_BIN="$TMP_DIR/bin"
	local TMP_DIR_DATA="$TMP_DIR/data"
	local TMP_DIR_ICON="$TMP_DIR/icons"
	local TMP_DIR_EPIHANY="$TMP_DIR/epiphany"
	local TMP_DIR_PORTAL="$TMP_DIR_EPIHANY/xdg-desktop-portal"
	local TMP_DIR_EPIHANY_DATA="$TMP_DIR_EPIHANY/data"
	local TMP_DIR_DESKTOP="$TMP_DIR/desktop"
	local WEBAPPS
	local SAVE_DIR
	local DATA_FOLDER
	local DATA_FOLDER_COPY
	local DATA_DIR_COPY
	local TMP_FILE_ICON
	local TMP_FILE_BIN
	local -i nDesktop_Files_Found
	local -i cancel=1

	mapfile -t WEBAPPS < <(find "$HOME"/.local/share/applications -iname "*-webapp-biglinux-custom.desktop")
	nDesktop_Files_Found="${#WEBAPPS[@]}"

	if ! ((nDesktop_Files_Found)); then
		yad --title="$TITLE" \
			--image=emblem-warning \
			--image-on-top \
			--form \
			--width=500 \
			--height=100 \
			--fixed \
			--align=center \
			--text="$SUBTITLE" \
			--button="$BUTTON_CLOSE" \
			--on-top \
			--center \
			--borders=20 \
			--window-icon="$WEBAPPS_PATH/icons/webapp.svg"
		exit 1
	fi

	cd "$HOME_FOLDER" || return
	SAVE_DIR=$(
		yad --title="$SAVE_DIR_TITLE" \
			--file \
			--directory \
			--center \
			--window-icon="$WEBAPPS_PATH/icons/webapp.svg" \
			--width=900 --height=600
	)

	if [[ "$?" -eq "$cancel" ]] || [[ -z "$SAVE_DIR" ]]; then
		exit
	fi

	for w in "${WEBAPPS[@]}"; do
		if grep -q '.local.bin' "$w"; then
			mkdir -p "$TMP_DIR_BIN"
			TMP_FILE_BIN=$(awk -F'=' '/^Exec/{print $2}' "$w")
			cp -a "$BIN" "$TMP_DIR_BIN"
			DATA_FOLDER=$(sed -n '/^FOLDER/s/.*=\([^\n]*\).*/\1/p' "$TMP_FILE_BIN")
			if grep -q '.bigwebapps' <<<"$DATA_FOLDER"; then
				mkdir -p "$TMP_DIR_DATA"
				cp -a "$DATA_FOLDER" "$TMP_DIR_DATA"
			else
				DATA_FOLDER_COPY="$TMP_DIR/flatpak/$(awk -F'/' '{print $6"/"$7}' <<<"$DATA_FOLDER")"
				mkdir -p "$DATA_FOLDER_COPY"
				cp -a "$DATA_FOLDER" "$DATA_FOLDER_COPY"
			fi
		fi

		if grep -q '..no.first.run' "$w"; then
			DATA_DIR=$(awk '/Exec/{sub(/--user-data-dir=/,"");print $2}' "$w")
			if grep -q '.bigwebapps' <<<"$DATA_DIR"; then
				mkdir -p "$TMP_DIR_DATA"
				cp -a "$DATA_DIR" "$TMP_DIR_DATA"
			else
				DATA_DIR_COPY="$TMP_DIR/flatpak/$(awk -F'/' '{print $6"/"$7}' <<<"$DATA_DIR")"
				mkdir -p "$DATA_DIR_COPY"
				cp -a "$DATA_DIR" "$DATA_DIR_COPY"
			fi
		fi

		if grep -q '..profile=' "$w"; then
			mkdir -p "$TMP_DIR_PORTAL"
			cp -a "$HOME"/.local/share/xdg-desktop-portal/* "$TMP_DIR_PORTAL"
			EPI_DATA=$(awk '/Exec/{sub(/--profile=/,"");print $3}' "$w")
			mkdir -p "$TMP_DIR_EPIHANY_DATA"
			cp -a "$EPI_DATA" "$TMP_DIR_EPIHANY_DATA"
		else
			mkdir -p "$TMP_DIR_ICON"
			TMP_FILE_ICON=$(awk -F'=' '/^Icon/{print $2}' "$w")
			cp -a "$ICON" "$TMP_DIR_ICON"
			cp -a "$w" "$TMP_DIR"
		fi

		if [ -L "$(xdg-user-dir DESKTOP)/${w##*/}" ]; then
			mkdir -p "$TMP_DIR_DESKTOP"
			cp -a "$(xdg-user-dir DESKTOP)/${w##*/}" "$TMP_DIR_DESKTOP"
		fi
	done

	cd "$TMP_FOLDER" || return
	tar -czf "${SAVE_DIR}/${backup_file}" backup-webapps
	rm -r backup-webapps
	echo "${SAVE_DIR}/${backup_file}"
	exit
}

sh_webapp_backup_main "$@"
