#!/bin/bash

#Translation
export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

kdialog --getopenfilename ~ $"Ícones(*.png *.svg *.ico *.xpm)"
