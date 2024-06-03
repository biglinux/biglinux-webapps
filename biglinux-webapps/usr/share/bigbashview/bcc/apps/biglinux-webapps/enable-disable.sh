#!/usr/bin/env bash
#shellcheck disable=SC2155,SC2034
#shellcheck source=/dev/null

#  /usr/share/bigbashview/bcc/apps/biglinux-webapps/enable-disable.sh
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

function sh_webapp_enable_disable_main() {
	local LOCAL_DIR="$HOME_LOCAL/share/applications/$1"

	case "$2" in
	firefox | org.mozilla.firefox | librewolf | io.gitlab.librewolf-community)
		if [[ ! -e "$LOCAL_DIR" ]]; then
			cp "$WEBAPPS_PATH/assets/$2/desk/$1" "$HOME_LOCAL"/share/applications
			cp "$WEBAPPS_PATH/assets/$2/bin/${1%%.*}-$2" "$HOME_LOCAL"/bin
		else
			rm "$LOCAL_DIR"
		fi
		;;

	*)
		if [[ ! -e "$LOCAL_DIR" ]]; then
			cp "$WEBAPP_PATH/webapps/$1" "$HOME_LOCAL"/share/applications
		else
			rm "$LOCAL_DIR"
		fi
		;;
	esac

	update-desktop-database -q "$HOME_LOCAL"/share/applications
	nohup kbuildsycoca5 &>/dev/null &
	exit
}

sh_webapp_enable_disable_main "$@"
