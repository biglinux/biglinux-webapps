#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

icon=$(kdialog --getopenfilename ~ $"Ícones(*.png *.ico *.xpm)")
echo $icon
exit
