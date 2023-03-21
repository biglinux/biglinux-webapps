#!/usr/bin/env bash

CHANGE=false
FILES=($(find ~/.local/share/applications -iname '*-webapp-biglinux.desktop'))
BASEDIR=/usr/share/bigbashview/bcc/apps/biglinux-webapps

if [ ! "$FILES" ];then
    printf "$2" > ~/.bigwebapps/BROWSER
    exit
fi

function ChromeToFire(){
    for w in "${FILES[@]}";do
        filename="${w##*/}"
        cp -f "$BASEDIR"/assets/"$1"/bin/"${filename%%.*}-$1" ~/.local/bin
        cp -f "$BASEDIR"/assets/"$1"/desk/"$filename" ~/.local/share/applications
    done
}


function FireToChrome(){
    for w in "${FILES[@]}";do
        cp -f "$BASEDIR"/webapps/"${w##*/}" ~/.local/share/applications
    done
}


function ClearFiles(){
    for w in "${FILES[@]}";do
        EXEC_BIN=~/.local/bin/$(sed -n '/^Exec/s/.*\/\([^\/]*\)$/\1/p' "$w")
        DATA_DIR=~/$(sed -n '/^FOLDER/s/.*=~\/\([^\n]*\).*/\1/p' "$EXEC_BIN")

        [ -d "$DATA_DIR" ] && rm -r "$DATA_DIR"
        [ -e "$EXEC_BIN" ] && rm "$EXEC_BIN"
    done
}


case "$1" in
    firefox|org.mozilla.firefox|librewolf|io.gitlab.librewolf-community)
        case "$2" in
            firefox|org.mozilla.firefox|librewolf|io.gitlab.librewolf-community)
                ClearFiles
                ChromeToFire "$2"
                CHANGE=true
            ;;

            *)  ClearFiles
                FireToChrome
                CHANGE=true
            ;;
        esac
    ;;

    *)  case "$2" in
            firefox|org.mozilla.firefox|librewolf|io.gitlab.librewolf-community)
                ChromeToFire "$2"
                CHANGE=true
            ;;

            *):;;
        esac
    ;;
esac


if [ "$CHANGE" = "true" ];then
    update-desktop-database -q ~/.local/share/applications
    nohup kbuildsycoca5 &>/dev/null &
fi


printf "$2" > ~/.bigwebapps/BROWSER
exit
