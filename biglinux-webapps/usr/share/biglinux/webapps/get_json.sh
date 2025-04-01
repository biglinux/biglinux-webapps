#!/bin/bash

cache_filename="bigwebapps"

# If the first argument is "cache", then cache the json file
if [[ $1 = "cache" ]]; then
    python get_app_icon_url.py <(big-webapps json) > ~/.cache/$cache_filename.json
    rm ~/.cache/pre-cache-$cache_filename.json
    exit
fi


if [ -f "$HOME/.cache/pre-cache-$cache_filename.json" ] || [ -f "$HOME/.cache/$cache_filename.json" ]; then
    # Wait for pre-cache file is deleted, if it is not deleted in 5 seconds, display without cache
    for i in {1..50}; do
        if [ ! -f "$HOME/.cache/pre-cache-$cache_filename.json" ]; then
            echo "$(< ~/.cache/$cache_filename.json)"
            rm ~/.cache/$cache_filename.json
            exit 0
        fi
        sleep 0.1
    done
fi

# Only call cache fail or not using cache
python get_app_icon_url.py <(big-webapps json)
