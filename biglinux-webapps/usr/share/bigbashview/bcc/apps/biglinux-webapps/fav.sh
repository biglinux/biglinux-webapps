#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

domain="$(echo "$1" | sed 's|http://||;s|https://||')"
path="$(python3 favicon.py "$domain")"
if [ "$path" != "" ];then
	echo "$path"
else
	kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --error $"Não foi possível detectar o ícone deste site!"

	echo "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
fi
exit
