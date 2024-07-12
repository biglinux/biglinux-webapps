#!/usr/bin/env bash
#shellcheck disable=SC2155,SC2034
#shellcheck source=/dev/null

#  /usr/bin/webapp-manager-biglinux
#  Description: WebApps installing programs for BigLinux
#
#  Created: 2020/01/11
#  Altered: 2024/06/24
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
_VERSION_="1.0.0-20240624"
LIBRARY=${LIBRARY:-'/usr/share/bigbashview/bcc/shell'}
[[ -f "${LIBRARY}/bcclib.sh" ]] && source "${LIBRARY}/bcclib.sh"
[[ -f "${LIBRARY}/tinilib.sh" ]] && source "${LIBRARY}/tinilib.sh"
[[ -f "${LIBRARY}/weblib.sh" ]] && source "${LIBRARY}/weblib.sh"

function sh_unset_config() {
	unset QT_QPA_PLATFORM
	unset SDL_VIDEODRIVER
	unset WINIT_UNIX_BACKEND
	unset GDK_BACKEND
	unset MOZ_ENABLE_WAYLAND
	unset QT_QPA_PLATFORM
	unset MOZ_ENABLE_WAYLAND
}

function sh_webapps_main {
	# Define o tamanho padrão da janela
	local default_size='960x720'
	local BROWSER
	local width
	local height
	local half_width
	local half_height
	local _session

	# Verifica se o arquivo .ini existe no HOME_FOLDER, senão executa um script de checagem dos navegadores
	[[ -r "$INI_FILE_WEBAPPS" ]] || sh_webapp_check_browser
	# Verifica se o arquivo .ini tem algum browser default, senão executa um script de checagem dos navegadores
	if BROWSER=$(TIni.Get "$INI_FILE_WEBAPPS" "browser" "short_name") && [[ -z "$BROWSER" ]]; then sh_webapp_check_browser; fi
	if ! sh_webapp_verify_browser "$BROWSER"; then
		#		notify-send -u critical --icon=webapps --app-name "$0" "$TITLE" "${Amsg[error_browser]}" --expire-time=2000
		yadmsg "${Amsg[error_browser_config]}"
		sh_webapp_check_browser
	fi

	# verifica que existem webapp antigos para converter para novo formato wayland
	sh_webapp_convert

	cd "$WEBAPPS_PATH" || {
		notify-send --icon=webapp.svg --app-name "$0" "$TITLE" "${Amsg[error_access_dir]}\n$WEBAPP_PATH" --expire-time=2000
		return 1
	}

	# Obtém a largura da tela primária usando xrandr
	if width=$(xrandr | grep -oP 'primary \K[0-9]+(?=x)') && [[ -n "$width" ]]; then
		# Se a largura foi obtida, tenta obter a altura da tela primária
		if height=$(xrandr | grep -oP 'primary \K[0-9]+x\K[0-9]+') && [[ -n "$height" ]]; then
			# Calcula metade da largura e altura
			half_width=$((width / 2))
			half_height=$((height / 2 * 3 / 2))
			# Atualiza o tamanho padrão com metade da largura e altura da tela
			default_size="${half_width}x${half_height}"
		fi
	fi

	# Define várias variáveis de ambiente para usar a plataforma x11
	# -s Define o tamanho da janela
	# -n Define o título da janela
	# -p Define o nome do aplicativo
	# -d Define o caminho das webapps
	# -i Define o ícone do aplicativo

	sh_unset_config
	_session="$(sh_get_desktop_session)"
	case "${_session^^}" in
	X11)
		export QT_QPA_PLATFORM=xcb
		export SDL_VIDEODRIVER=x11
		export WINIT_UNIX_BACKEND=x11
		export GDK_BACKEND=x11
		;;
	WAYLAND)
		export MOZ_ENABLE_WAYLAND=1
		:
		;;
	esac

	bigbashview index.sh.htm \
		-s "${default_size}" \
		-n "$TITLE" \
		-p "$APP" \
		-d "$WEBAPPS_PATH" \
		-i "icons/big-webapps.svg"
	sh_unset_config
}

#sh_debug
sh_check_webapp_is_running
sh_webapp_check_dirs
sh_webapps_main "$@"
