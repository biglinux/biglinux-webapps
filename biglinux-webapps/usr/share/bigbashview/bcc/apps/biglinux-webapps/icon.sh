#!/usr/bin/env bash
#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps


icon=$(kdialog --title "BigLinux WebApps" --icon "internet-web-browser" --getopenfilename ~ $"Arquivos de Imagem(*.bmp *.png *.ico *.xpm *.jpg)")
if [ "$icon" != "" ];then
	echo "$icon"
else
	echo "/usr/share/bigbashview/bcc/apps/biglinux-webapps/default.png"
fi

exit