#!/usr/bin/env bash

APPSPATH="/usr/share/bigbashview/bcc/apps/biglinux-webapps/webapps"
LOCALPATH="$HOME/.local/share/applications"

[ ! -e "$LOCALPATH/$1" ] && cp $APPSPATH/$1 $LOCALPATH || rm "$LOCALPATH/$1"

update-desktop-database -q $LOCALPATH
kbuildsycoca5 &> /dev/null
exit
