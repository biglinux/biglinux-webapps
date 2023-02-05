#!/usr/bin/env bash

CHANGE=false
EDIT=false

if [ "$browserOld" != "$browserNew" ]; then
    name_file="$RANDOM-${icondesk##*/}"
    cp -f "$icondesk" /tmp/"$name_file"
    icondesk=/tmp/"$name_file"
    CHANGE=true
fi

if [ "$newperfil" = "on" ]; then
    if ! grep -q '..user.data.dir.' "$filedesk"; then
        name_file="$RANDOM-${icondesk##*/}"
        cp -f "$icondesk" /tmp/"$name_file"
        icondesk=/tmp/"$name_file"
        CHANGE=true
    fi
fi

if [ "$CHANGE" = "true" ]; then
    JSON=$(
        tr -d "\ \n\r" <<EOF
{
  "browser"   : "$browserNew",
  "category"  : "$category",
  "filedesk"  : "$filedesk",
  "icondesk"  : "$icondesk",
  "namedesk"  : "$namedesk",
  "newperfil" : "$newperfil",
  "shortcut"  : "$shortcut",
  "urldesk"   : "$urldesk"
}
EOF
    )
    printf "%s" "$JSON"
    exit
fi

if [ "$icondesk" != "$icondeskOld" ]; then
    mv -f "$icondesk" "$icondeskOld"
    EDIT=true
fi

if [ "$namedeskOld" != "$namedesk" ]; then
    sed -i "s|Name=$namedeskOld|Name=$namedesk|" "$filedesk"
    EDIT=true
fi

if [ "$categoryOld" != "$category" ]; then
    sed -i "s|Categories=$categoryOld;|Categories=$category;|" "$filedesk"
    EDIT=true
fi

if [ ! "$newperfil" ]; then
    if grep -q '..user.data.dir.' "$filedesk"; then
        FIELD=$(awk '/Exec/{print $2}' "$filedesk")
        FOLDER=$(awk -F'=' '{print $2}' <<<"$FIELD")
        rm -r "$FOLDER"
        sed -i "s|$FIELD --no-first-run ||" "$filedesk"
        EDIT=true
    fi
fi

USER_DESKTOP=$(xdg-user-dir DESKTOP)
DESKNAME=${filedesk##*/}
if [ "$shortcut" = "on" ]; then
    if [ ! -L "$USER_DESKTOP/$DESKNAME" ]; then
        ln -sf "$filedesk" "$USER_DESKTOP/$DESKNAME"
        chmod 755 "$USER_DESKTOP/$DESKNAME"
        gio set "$USER_DESKTOP/$DESKNAME" -t string metadata::trust "true"
        EDIT=true
    else
        ln -sf "$filedesk" "$USER_DESKTOP/$DESKNAME"
        chmod 755 "$USER_DESKTOP/$DESKNAME"
        gio set "$USER_DESKTOP/$DESKNAME" -t string metadata::trust "true"
    fi
else
    if [ -L "$USER_DESKTOP/$DESKNAME" ]; then
        unlink "$USER_DESKTOP/$DESKNAME"
        EDIT=true
    fi
fi

if [ "$EDIT" = "true" ]; then

    nohup update-desktop-database -q ~/.local/share/applications &
    nohup kbuildsycoca5 &>/dev/null &
    rm -f /tmp/*.png
    printf '{ "return" : "0" }'
    exit
fi

if [ "$EDIT" = "false" ] && [ "$CHANGE" = "false" ]; then
    printf '{ "return" : "1" }'
    exit
fi
