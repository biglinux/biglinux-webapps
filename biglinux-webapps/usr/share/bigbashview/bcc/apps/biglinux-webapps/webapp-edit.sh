#!/usr/bin/env bash
#shellcheck disable=SC2155,SC2034
#shellcheck source=/dev/null

#  /usr/share/bigbashview/bcc/apps/biglinux-webapps/webapp-edit.sh
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

CHANGE=false
EDIT=false

if [ "$browserOld" != "$browserNew" ]; then
	name_file="$RANDOM-${icondesk##*/}"
	cp -f "$icondesk" /tmp/"$name_file"
	icondesk=/tmp/"$name_file"
	CHANGE=true
fi

if [ "$newperfil" = "on" ]; then
	if ! grep -q '..user.data.dir.' "$filedesk"; then
		name_file="$RANDOM-${icondesk##*/}"
		cp -f "$icondesk" /tmp/"$name_file"
		icondesk=/tmp/"$name_file"
		CHANGE=true
	fi
fi

if [ "$CHANGE" = "true" ]; then
	JSON="{
  \"browser\"   : \"$browserNew\",
  \"category\"  : \"$category\",
  \"filedesk\"  : \"$filedesk\",
  \"icondesk\"  : \"$icondesk\",
  \"namedesk\"  : \"$namedesk\",
  \"newperfil\" : \"$newperfil\",
  \"shortcut\"  : \"$shortcut\",
  \"urldesk\"   : \"$urldesk\"
}"
	printf "%s" "$JSON"
	exit
fi

if [ "$icondesk" != "$icondeskOld" ]; then
	mv -f "$icondesk" "$icondeskOld"
	EDIT=true
fi

if [ "$namedeskOld" != "$namedesk" ]; then
	sed -i "s|Name=$namedeskOld|Name=$namedesk|" "$filedesk"
	EDIT=true
fi

if [ "$categoryOld" != "$category" ]; then
	sed -i "s|Categories=$categoryOld;|Categories=$category;|" "$filedesk"
	EDIT=true
fi

if [ ! "$newperfil" ]; then
	if grep -q '..user.data.dir.' "$filedesk"; then
		FIELD=$(awk '/Exec/{print $2}' "$filedesk")
		FOLDER=$(awk -F'=' '{print $2}' <<<"$FIELD")
		rm -r "$FOLDER"
		sed -i "s|$FIELD --no-first-run ||" "$filedesk"
		EDIT=true
	fi
fi

USER_DESKTOP=$(xdg-user-dir DESKTOP)
DESKNAME=${filedesk##*/}
if [ "$shortcut" = "on" ]; then
	if [ ! -L "$USER_DESKTOP/$DESKNAME" ]; then
		ln -sf "$filedesk" "$USER_DESKTOP/$DESKNAME"
		chmod 755 "$USER_DESKTOP/$DESKNAME"
		gio set "$USER_DESKTOP/$DESKNAME" -t string metadata::trust "true"
		EDIT=true
	else
		ln -sf "$filedesk" "$USER_DESKTOP/$DESKNAME"
		chmod 755 "$USER_DESKTOP/$DESKNAME"
		gio set "$USER_DESKTOP/$DESKNAME" -t string metadata::trust "true"
	fi
else
	if [ -L "$USER_DESKTOP/$DESKNAME" ]; then
		unlink "$USER_DESKTOP/$DESKNAME"
		EDIT=true
	fi
fi

if [ "$EDIT" = "true" ]; then
	nohup update-desktop-database -q ~/.local/share/applications &
	nohup kbuildsycoca5 &>/dev/null &
	rm -f /tmp/*.png
	printf '{ "return" : "0" }'
	exit
fi

if [ "$EDIT" = "false" ] && [ "$CHANGE" = "false" ]; then
	printf '{ "return" : "1" }'
	exit
fi
