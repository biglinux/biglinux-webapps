#!/usr/bin/env bash

# Function to check if a browser is installed
check_browser() {
    local browser_path=$1
    if [ -e "$browser_path" ]; then
        return 0 # true
    else
        return 1 # false
    fi
}

# Declare an associative array to map browser names to their paths
declare -A browsers
browsers=(
    ["firefox"]="/usr/lib/firefox/firefox"
    ["brave"]="/usr/lib/brave-browser/brave /opt/brave-bin/brave"
    ["brave-beta"]="/usr/bin/brave-beta"
    ["brave-nightly"]="/usr/bin/brave-nightly"
    ["librewolf"]="/usr/lib/librewolf/librewolf"
    ["chromium"]="/usr/lib/chromium/chromium"
    ["google-chrome-stable"]="/opt/google/chrome/google-chrome"
    ["google-chrome-beta"]="/opt/google/chrome-beta/google-chrome"
    ["google-chrome-unstable"]="/opt/google/chrome-unstable/google-chrome"
    ["microsoft-edge-stable"]="/opt/microsoft/msedge/microsoft-edge"
    ["vivaldi-stable"]="/opt/vivaldi/vivaldi"
    ["vivaldi-beta"]="/opt/vivaldi-beta/vivaldi"
    ["vivaldi-snapshot"]="/opt/vivaldi-snapshot/vivaldi"
    ["flatpak-brave"]="$HOME/.local/share/flatpak/app/com.brave.Browser /var/lib/flatpak/exports/bin/com.brave.Browser"
    ["flatpak-chrome"]="$HOME/.local/share/flatpak/app/com.google.Chrome /var/lib/flatpak/exports/bin/com.google.Chrome"
    ["flatpak-chrome-unstable"]="$HOME/.local/share/flatpak/app/com.google.ChromeDev /var/lib/flatpak/exports/bin/com.google.ChromeDev"
    ["flatpak-chromium"]="$HOME/.local/share/flatpak/app/org.chromium.Chromium /var/lib/flatpak/exports/bin/org.chromium.Chromium"
    ["flatpak-edge"]="$HOME/.local/share/flatpak/app/com.microsoft.Edge /var/lib/flatpak/exports/bin/com.microsoft.Edge"
    ["flatpak-ungoogled-chromium"]="$HOME/.local/share/flatpak/app/com.github.Eloston.UngoogledChromium /var/lib/flatpak/exports/bin/com.github.Eloston.UngoogledChromium"
    ["flatpak-firefox"]="$HOME/.local/share/flatpak/app/org.mozilla.firefox /var/lib/flatpak/exports/bin/org.mozilla.firefox"
    ["flatpak-librewolf"]="$HOME/.local/share/flatpak/app/io.gitlab.librewolf-community /var/lib/flatpak/exports/bin/io.gitlab.librewolf-community"
)

# Array to maintain the order of browser names
browser_order=(
    "brave"
    "brave-beta"
    "brave-nightly"
    "firefox"
    "chromium"
    "google-chrome-stable"
    "google-chrome-beta"
    "google-chrome-unstable"
    "microsoft-edge-stable"
    "vivaldi-stable"
    "librewolf"
    "vivaldi-beta"
    "vivaldi-snapshot"
    "flatpak-brave"
    "flatpak-chrome"
    "flatpak-chrome-unstable"
    "flatpak-chromium"
    "flatpak-edge"
    "flatpak-ungoogled-chromium"
    "flatpak-firefox"
    "flatpak-librewolf"
)

# Function to show the default browser
show_default_browser() {
    for browser in "${browser_order[@]}"; do
        IFS=' ' read -r -a paths <<< "${browsers[$browser]}"
        for path in "${paths[@]}"; do
            if check_browser "$path"; then
                if [ "$1" == "--json" ]; then
                    echo "{\"default_browser\": \"$browser\"}"
                else
                    echo "$browser"
                fi
                return
            fi
        done
    done
}

# Function to list all installed browsers
list_installed_browsers() {
    local json_output="["
    local first=true
    for browser in "${browser_order[@]}"; do
        IFS=' ' read -r -a paths <<< "${browsers[$browser]}"
        for path in "${paths[@]}"; do
            if check_browser "$path"; then
                if [ "$1" == "--json" ]; then
                    if [ "$first" = true ]; then
                        json_output="$json_output{\"browser\": \"$browser\"}"
                        first=false
                    else
                        json_output="$json_output, {\"browser\": \"$browser\"}"
                    fi
                else
                    echo "$browser"
                fi
                break
            fi
        done
    done
    if [ "$1" == "--json" ]; then
        json_output="$json_output]"
        echo "$json_output"
    fi
}

# Function to list all compatible browsers
list_compatible_browsers() {
    local json_output="["
    local first=true
    for browser in "${browser_order[@]}"; do
        if [ "$1" == "--json" ]; then
            if [ "$first" = true ]; then
                json_output="$json_output{\"browser\": \"$browser\"}"
                first=false
            else
                json_output="$json_output, {\"browser\": \"$browser\"}"
            fi
        else
            echo "$browser"
        fi
    done
    if [ "$1" == "--json" ]; then
        json_output="$json_output]"
        echo "$json_output"
    fi
}

# Function to display help message
show_help() {
    echo "Usage: $0 [--default] [--list] [--list-json] [--list-all] [--list-all-json] [--help]"
    echo ""
    echo "Options:"
    echo "  --default           Show the default browser."
    echo "  --list              List all installed browsers."
    echo "  --list-json         List all installed browsers in JSON format."
    echo "  --list-all          List all compatible browsers."
    echo "  --list-all-json     List all compatible browsers in JSON format."
    echo "  --help              Display this help message."
}

# Main script logic to handle arguments
case "$1" in
    --default)
        show_default_browser
        ;;
    --list)
        list_installed_browsers
        ;;
    --list-json)
        list_installed_browsers "--json"
        ;;
    --list-all)
        list_compatible_browsers
        ;;
    --list-all-json)
        list_compatible_browsers "--json"
        ;;
    --help|*)
        show_help
        ;;
esac
