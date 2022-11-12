browser_bin_list=( "brave" "brave-browser" "google-chrome-stable" "vivaldi" "microsoft-edge-beta" "microsoft-edge-dev" "microsoft-edge-stable" "epiphany" "firefox" "yandex-browser-stable" "yandex-browser-beta" )

for browser in "${browser_bin_list[@]}";do
  if which "$browser" &>/dev/null; then
    browser_name="${browser^^}"
    case "$browser" in
      "microsoft-edge-"*)
        browser_name="${browser_name:10}"
        browser_name="${browser_name//-/' '}"
        [[ "$browser" = *"-stable" ]] && browser_name="${browser_name::-7}"
        ;;
      "google-chrome-stable")
        browser_name="CHROME"
        ;;
      "vivaldi")
        browser_name="VIVALDI"
        ;;
      "yandex-browser-"*)
        browser_name="${browser_name//-browser-/' '}"
        [[ "$browser" = *"-stable" ]] && browser_name="${browser_name::-7}"
        ;;
      "brave-browser")
        browser_name="BRAVE"
        ;;
    esac
    # echo $brower_name
    echo "<option value=\"${browser}\">${browser_name}</option>"
  fi
done
