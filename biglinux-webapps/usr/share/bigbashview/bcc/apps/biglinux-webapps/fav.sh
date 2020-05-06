#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

domain="$(echo "$1" | sed 's|.*://||;s|/.*||')"
path="$(python3 favicon.py "$domain")"
if [ "$path" != "" ];then
	echo "$path" 
else
	echo $"Não foi possível pesquisar neste site!"
fi
exit
