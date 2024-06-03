#!/usr/bin/env bash
#shellcheck disable=SC2155,SC2034
#shellcheck source=/dev/null

#  /usr/share/bigbashview/bcc/apps/biglinux-webapps/restore.sh
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

function sh_webapp_restore_main() {
	local SUBTITLE="$(gettext $"Selecionar o arquivo de backup para restaurar: ")"
	local BKP_FOLDER="$TMP_FOLDER/backup-webapps"
	local FLATPAK_FOLDER_DATA="$HOME"/.var/app
	local backup_file
	local cancel=1

	cd "$HOME_FOLDER" || return
	backup_file=$(
		yad --title="$SUBTITLE" \
			--file \
			--window-icon="$WEBAPPS_PATH/icons/webapp.svg" \
			--width=900 \
			--height=600 \
			--center \
			--mime-filter=$"Backup WebApps""|application/gzip"
	)
	if [[ "$?" -eq "$cancel" ]] || [[ -z "$backup_file" ]]; then
		exit
	fi

	if tar -xzf "$backup_file" -C "$TMP_FOLDER"; then
		if [[ -d "$BKP_FOLDER" ]]; then
			cp -a "$BKP_FOLDER"/*.desktop "$HOME_LOCAL"/share/applications
			cp -a "$BKP_FOLDER"/icons/* "$HOME_LOCAL"/share/icons

			if [[ -d "$BKP_FOLDER"/bin ]]; then
				cp -a "$BKP_FOLDER"/bin/* "$HOME_LOCAL"/bin
			fi

			if [[ -d "$BKP_FOLDER"/data ]]; then
				cp -a "$BKP_FOLDER"/data/* "$HOME_FOLDER"
			fi

			if [[ -d "$BKP_FOLDER"/epiphany ]]; then
				cp -a "$BKP_FOLDER"/epiphany/data "$FLATPAK_FOLDER_DATA"/org.gnome.Epiphany
				cp -a "$BKP_FOLDER"/epiphany/xdg-desktop-portal "$HOME_LOCAL"/share
				ln -sf "$HOME"/.local/share/xdg-desktop-portal/applications/*.desktop "$HOME_LOCAL"/share/applications
			fi

			if [[ -d "$BKP_FOLDER"/flatpak ]]; then
				cp -a "$BKP_FOLDER"/flatpak/* "$FLATPAK_FOLDER_DATA"
			fi

			if [[ -d "$BKP_FOLDER"/desktop ]]; then
				cp -a "$BKP_FOLDER"/desktop/* "$(xdg-user-dir DESKTOP)"
			fi

			rm -r "$BKP_FOLDER"
			update-desktop-database -q "$HOME_LOCAL"/share/applications
			nohup kbuildsycoca5 &>/dev/null &

			printf 0
			exit
		fi
	fi
}

sh_webapp_restore_main "$@"
