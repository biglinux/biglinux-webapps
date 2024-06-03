#!/usr/bin/env bash
#shellcheck disable=SC2155,SC2034
#shellcheck source=/dev/null

#  /usr/share/bigbashview/bcc/apps/biglinux-webapps/check_browser.sh
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

function sh_webapps_check_browser_main() {
	local default_browser=
	local NOT_COMPATIBLE=0 # Flag para indicar se navegador é compatível

	# Verifica a existência de navegadores instalados e define o navegador padrão
	if [ -e /usr/lib/brave-browser/brave ] || [ -e /opt/brave-bin/brave ]; then
		default_browser='brave'
	elif [ -e /opt/google/chrome/google-chrome ]; then
		default_browser='google-chrome-stable'
	elif [ -e /usr/lib/chromium/chromium ]; then
		default_browser='chromium'
	elif [ -e /opt/microsoft/msedge/microsoft-edge ]; then
		default_browser='microsoft-edge-stable'
	elif [ -e /usr/lib/firefox/firefox ]; then
		sh_webapp_change_browser 'brave' 'firefox'
	elif [ -e /usr/lib/librewolf/librewolf ]; then
		sh_webapp_change_browser 'brave' 'librewolf'
	elif [ -e /usr/bin/falkon ]; then
		default_browser='falkon'
	elif [ -e /opt/vivaldi/vivaldi ]; then
		default_browser='vivaldi-stable'
	elif [ -e /var/lib/flatpak/exports/bin/com.brave.Browser ]; then
		default_browser='com.brave.Browser'
	elif [ -e /var/lib/flatpak/exports/bin/com.google.Chrome ]; then
		default_browser='com.google.Chrome'
	elif [ -e /var/lib/flatpak/exports/bin/org.chromium.Chromium ]; then
		default_browser='org.chromium.Chromium'
	elif [ -e /var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium ]; then
		default_browser='com.github.Eloston.UngoogledChromium'
	elif [ -e /var/lib/flatpak/exports/bin/com.microsoft.Edge ]; then
		default_browser='com.microsoft.Edge'
	elif [ -e /var/lib/flatpak/exports/bin/org.gnome.Epiphany ]; then
		default_browser=
		NOT_COMPATIBLE=1
	elif [ -e /var/lib/flatpak/exports/bin/org.mozilla.firefox ]; then
		sh_webapp_change_browser 'brave' 'org.mozilla.firefox'
	elif [ -e /var/lib/flatpak/exports/bin/io.gitlab.librewolf-community ]; then
		sh_webapp_change_browser 'brave' 'io.gitlab.librewolf-community'
	fi

	if ((NOT_COMPATIBLE)); then
		# Exibe uma mensagem de erro se nenhum navegador compatível for encontrado
		yad --image=emblem-warning \
			--image-on-top \
			--form \
			--width=500 \
			--height=100 \
			--fixed \
			--align=center \
			--text="$(gettext $"Não existem navegadores compatíveis com WebApps instalados!")" \
			--button="$(gettext $"Fechar")" \
			--on-top \
			--center \
			--borders=20 \
			--title="$TITLE" \
			--window-icon="$WEBAPPS_PATH/icons/webapp.svg"
		exit 1
	fi

	# Atualiza a configuração do navegador se necessário
	[[ "$(<~/.bigwebapps/BROWSER)" = "brave-browser" ]] && default_browser='brave'

	# Salva o navegador padrão no arquivo de configuração
	echo "$default_browser" >"$HOME_FOLDER"/BROWSER
}

sh_webapps_check_browser_main "$@"
