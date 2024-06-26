#!/usr/bin/env bash

# Change directory to the local user's applications directory
cd ~/.local/share/applications

# Get the default browser
defaultBrowser=$(/usr/share/bigbashview/apps/webapps/check_browser.sh --default)

# Iterate over all webapps
for file in *webapp-biglinux*; do

    if [[ "$file" = '*webapp-biglinux*' ]]; then
        exit
    fi

    # Clean info before verify next file
    unset browser name url icon categories profile

    # Read any file and get the name, url, icon and categories
    while IFS= read -r line; do
        case $line in
            "Name="*)
                name=${line#*Name=}
                ;;
            "Exec="*)
                browser=${line%% *}
                browser=${browser#Exec=}
                if [[ "$browser" =~ biglinux-webapp ]]; then
                    browser=$defaultBrowser

                # In older versions of biglinux-webapp, Firefox and browsers based in Firefox use script in ~/.local/bin
                elif [[ "$browser" =~ \.local/bin ]]; then
                    oldScript=$browser
                    browser=$(grep -m1 '\--new-instance' "$oldScript" | grep -o '^[^ ]*')
                    url=$(grep -m1 '\--new-instance' "$oldScript" | grep -o 'http[^" ]*')
                    profile='Default'
                fi

                # Extract the URL and profile directory if they are not already set
                url=${url:-${line#*--app=}}
                profile=${profile:-${line#*--profile-directory=}}
                profile=${profile% *}
                ;;
            "Icon="*)
                icon=${line#*Icon=}
                icon=$(echo $icon | sed "s|$HOME/.local/share/icons/||g;s|~/.local/share/icons/||g")
                ;;
            "Categories="*)
                categories=${line#*Categories=}
                ;;
        esac
    done <<<$(grep -m4 -e '^Name=' -e '^Exec=' -e '^Icon=' -e '^Categories=' $file)

    # Create a new desktop file compatible with wayland
    big-webapps create "$browser" "$name" "$url" "$icon" "$categories" "$profile"

    # Remove the old desktop file
    rm $file

done
