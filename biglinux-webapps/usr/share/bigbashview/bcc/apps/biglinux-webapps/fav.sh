#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

domain="$(echo "$1" | sed 's|http://||;s|https://||')"
path="$(python3 favicon.py "$domain")"
if [ "$path" != "" ];then
	identify "$path" > /dev/null
	if [ "$?" = "0" ]; then
    	echo "$path"
	else
    	domain_try="$(echo "$1" | sed 's|http://||;s|https://||;s|/.*||')"
		path_try="$(python3 favicon.py "$domain_try")"
		echo "$path_try"
	fi
else
	kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --error $"Não foi possível detectar o ícone deste site!"

	echo "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
fi
exit
