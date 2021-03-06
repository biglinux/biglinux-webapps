#!/usr/bin/env bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

domain="$(echo "$1" | sed 's|http://||;s|https://||')"
path="$(python3 get_favicon.py "$domain")"
if [ "$path" != "" ];then
	identify "$path" > /dev/null
	if [ "$?" = "0" ]; then
    	echo "$path"
	else
    	domain_try="$(echo "$1" | sed 's|http://||;s|https://||;s|/.*||')"
		path_try="$(python3 get_favicon.py "$domain_try")"
		identify "$path_try" > /dev/null
		if [ "$?" = "0" ]; then
    		echo "$path_try"
    	else
			kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --error $"Não foi possível detectar o ícone deste site!"
			echo "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
		fi
	fi
else
	domain_try="$(echo "$1" | sed 's|http://||;s|https://||;s|/.*||')"
	path_try="$(python3 get_favicon.py "$domain_try")"
	if [ "$path_try" != "" ];then
		echo "$path_try"
	else
		kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --error $"Não foi possível detectar o ícone deste site!"
		echo "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
	fi
fi
exit
