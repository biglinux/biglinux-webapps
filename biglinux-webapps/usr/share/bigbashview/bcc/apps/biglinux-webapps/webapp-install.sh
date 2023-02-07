#!/usr/bin/env bash

FLATPAK_BIN=/var/lib/flatpak/exports/bin
SNAPD_BIN=/var/lib/snapd/snap/bin

_NAMEDESK=$(sed 's|https\:\/\/||;s|www\.||;s|\/.*||;s|\.|-|g' <<<"$urldesk")
USER_DESKTOP=$(xdg-user-dir DESKTOP)

NAME="${_NAMEDESK}-$RANDOM"
DIR_PROF="$HOME/.bigwebapps/$NAME"
BASENAME="$NAME-webapp-biglinux-custom"
BROWSER_NAME="${browser##*/}"
case "$browser" in
*epiphany)
	BASENAME="$BROWSER_NAME.WebApp_$BASENAME"
	DIR_PROF="$HOME/.bigwebapps/$BASENAME"
	;;
esac

BASENAME_APP="$BASENAME.desktop"
LINK_APP="$HOME/.local/share/applications/$BASENAME_APP"
DESKTOP_LINK="$USER_DESKTOP/$BASENAME_APP"

if [[ "$browser" = "$FLATPAK_BIN/"* ]]; then
	DIR_PROF="$HOME/.var/app/$BROWSER_NAME/data/$NAME"
	FLATPAK_LINE="X-Flatpak=$BROWSER_NAME"
fi

BASENAME_ICON="${icondesk##*/}"
NAME_FILE="${BASENAME_ICON// /-}"
ICON_FILE=~/.local/share/icons/"$NAME_FILE"

if [[ "$urldesk" != *"://"* ]]; then
	urldesk="https://$urldesk"
fi

gen_launcher() {
	cat <<EOF >"$1"
[Desktop Entry]
Version=1.0
Terminal=false
Type=Application
Name=$namedesk
Exec=$2
Icon=$ICON_FILE
Categories=$category;
StartupNotify=true
X-KDE-StartupNotify=true
$3
$FLATPAK_LINE
EOF
}

cp_icon() {
	if [ "${icondesk##*/}" = "default-webapps.png" ]; then
		cp "$icondesk" "$1"
	else
		mv "$icondesk" "$1"
	fi
}

shopt -s nocasematch
case "$browser" in
*firefox | *firedragon | *librewolf*)

	cp_icon "$ICON_FILE"

	DESKBIN="$HOME/.local/bin/$NAME"

	cat >"$DESKBIN" <<EOF
#!/usr/bin/env sh
#
# Amofi - App mode for Firefox
# Copyright (C) 2017-2019  SEPBIT
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# dash version 0.5
#
# @author    Vitor Guia <contato@vitor.guia.nom.br>
# @Modified by Bruno Goncalves <www.biglinux.com.br>
# @copyright 2017-2019 SEPBIT
# @license   http://www.gnu.org/licenses GPL-3.0-or-later
# @see       https://notabug.org/sepbit/amofi Repository of Amofi

FOLDER=$DIR_PROF

if ! grep -q 'toolkit.legacyUserProfileCustomizations.stylesheets' "\$FOLDER/prefs.js" 2>/dev/null;then
	[ -d "\$FOLDER" ] && rm -r "\$FOLDER"
	mkdir -p "\$FOLDER/chrome"
	echo 'user_pref("media.eme.enabled", true);' >> "\$FOLDER/prefs.js"
	echo 'user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);' >> "\$FOLDER/prefs.js"
	sed -i 's|user_pref("browser.urlbar.placeholderName.*||g' "\$FOLDER/prefs.js"

	# Custom profile
	echo '#nav-bar{visibility: collapse;} #TabsToolbar{visibility: collapse;}' >> "\$FOLDER/chrome/userChrome.css"
	echo 'user_pref("browser.tabs.warnOnClose", false);' >> "\$FOLDER/user.js"
fi

CLASS="$BROWSER_NAME-webapp-$_NAMEDESK"

MOZ_DISABLE_GMP_SANDBOX=1 MOZ_DISABLE_CONTENT_SANDBOX=1 \
$browser --class="\$CLASS" --profile "\$FOLDER" --no-remote --new-instance "$urldesk" &

count=0
while [ \$count -lt 100 ];do
	wininfo=\$(xwininfo -root -children -all | grep \\"Navigator\\"\\ \\"\$CLASS\\")
	if [ "\$wininfo" ];then
		xseticon -id "\$(awk '{print \$1}' <<< \$wininfo)" $ICON_FILE
		count=100
	else
		let count=count+1;
	fi
	sleep 0.5
done
EOF

	chmod +x "$DESKBIN"

	gen_launcher "$LINK_APP" "$DESKBIN"
	;;
*firefoxpwa)

	cp_icon "$ICON_FILE"

	FPWA_PROFILE="00000000000000000000000000"

	APP_ID=$($browser site install --document-url "$urldesk" --icon-url "file://$ICON_FILE" --profile "$FPWA_PROFILE" --no-system-integration "data:application/manifest+json;base64,$(base64 <<<"{'start_url':'$urldesk','name':'$namedesk'}")" | tail -1 | awk '{print $6}')

	gen_launcher "$LINK_APP" "$browser site launch --url $urldesk $APP_ID"
	;;

*epiphany)
	EPI_LINK="$DIR_PROF/$BASENAME_APP"
	ICON_FILE="$DIR_PROF/app-icon.png"

	if [[ "$browser" = "$FLATPAK_BIN/"* ]]; then
		DIR_PORTAL="$HOME/.local/share/xdg-desktop-portal"
		DIR_PORTAL_APP="$DIR_PORTAL/applications"
		DIR_PORTAL_ICON="$DIR_PORTAL/icons/64x64"

		mkdir -p "$DIR_PORTAL_APP"
		mkdir -p "$DIR_PORTAL_ICON"

		EPI_LINK="$DIR_PORTAL_APP/$BASENAME_APP"
		ICON_FILE="$DIR_PORTAL_ICON/$BASENAME.png"
	fi

	DESKTOP_LINK="$USER_DESKTOP/$BASENAME_APP"
	mkdir -p "$DIR_PROF"
	true >"$DIR_PROF/.app"
	echo -n 37 >"$DIR_PROF/.migrated"

	cp_icon "$ICON_FILE"

	gen_launcher "$EPI_LINK" "$browser --application-mode --profile=$DIR_PROF $urldesk" "X-Purism-FormFactor=Workstation;Mobile;"

	ln -s "$EPI_LINK" "$LINK_APP"
	;;

*)
	if [ "$newperfil" = "on" ]; then
		browser="$browser --user-data-dir=$DIR_PROF --no-first-run"
	fi

	cp_icon "$ICON_FILE"

	CUT_HTTP=$(sed 's|https://||;s|/|_|g;s|_|__|1;s|_$||;s|_$||;s|&|_|g;s|?||g;s|=|_|g' <<<"$urldesk")

	gen_launcher "$LINK_APP" "$browser --class=$CUT_HTTP,Chromium-browser --profile-directory=Default --app=$urldesk" "StartupWMClass=$CUT_HTTP"
	;;
esac

chmod +x "$LINK_APP"

if [ "$shortcut" = "on" ]; then
	ln -s "$LINK_APP" "$DESKTOP_LINK"
	chmod 755 "$DESKTOP_LINK"
	gio set "$DESKTOP_LINK" -t string metadata::trust "true"
fi

nohup update-desktop-database -q ~/.local/share/applications &
nohup kbuildsycoca5 &>/dev/null &

rm -f /tmp/*.png
rm -rf /tmp/.bigwebicons
exit
