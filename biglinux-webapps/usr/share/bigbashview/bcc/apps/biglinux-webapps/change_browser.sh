#!/usr/bin/env bash
#shellcheck disable=SC2155,SC2034
#shellcheck source=/dev/null

#  /usr/share/bigbashview/bcc/apps/biglinux-webapps/change_browser.sh
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

function sh_webapp_change_browser_main() {
	local DESKTOP_FILES
	local -i CHANGE=0
	local -i nDesktop_Files_Found

	mapfile -t DESKTOP_FILES < <(find "$HOME_LOCAL"/share/applications -iname '*-webapp-biglinux.desktop')
	nDesktop_Files_Found="${#DESKTOP_FILES[@]}"

	if ((nDesktop_Files_Found)); then
		echo "$2" >"$HOME_FOLDER"/BROWSER
		exit
	fi

	function ChromeToFirefox() {
		local filename
		local w

		for w in "${DESKTOP_FILES[@]}"; do
			filename="${w##*/}"
			cp -f "$webapps_path"/assets/"$1"/bin/"${filename%%.*}-$1" "$HOME_LOCAL"/bin
			cp -f "$webapps_path"/assets/"$1"/desk/"$filename" "$HOME_LOCAL"/share/applications
		done
	}

	function FireTofoxChrome() {
		local w

		for w in "${DESKTOP_FILES[@]}"; do
			cp -f "$webapps_path"/webapps/"${w##*/}" "$HOME_LOCAL"/share/applications
		done
	}

	case "$1" in
	firefox | org.mozilla.firefox | librewolf | io.gitlab.librewolf-community)
		case "$2" in
		firefox | org.mozilla.firefox | librewolf | io.gitlab.librewolf-community)
			ChromeToFirefox "$2"
			CHANGE=1
			;;

		*)
			FirefoxToChrome
			CHANGE=1
			;;
		esac
		;;

	*)
		case "$2" in
		firefox | org.mozilla.firefox | librewolf | io.gitlab.librewolf-community)
			ChromeToFirefox "$2"
			CHANGE=1
			;;

		*) : ;;
		esac
		;;
	esac

	if ((CHANGE)); then
		update-desktop-database -q "$HOME"/.local/share/applications
		nohup kbuildsycoca5 &>/dev/null &
	fi
	echo "$2" >"$HOME_FOLDER"/BROWSER
	exit
}

sh_webapp_change_browser_main "$@"
