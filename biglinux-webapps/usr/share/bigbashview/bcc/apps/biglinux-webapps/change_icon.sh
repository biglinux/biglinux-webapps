#!/usr/bin/env bash

FILENAME=$(kdialog --getopenfilename "$(xdg-user-dir PICTURES)" 'image/bmp image/jpeg image/png image/x-icon' 2>/dev/null)

[ "$FILENAME" ] || exit 1

printf '%s' "$(./resize_favicon.sh.py "$FILENAME")"
exit 0
