#!/usr/bin/env bash

RESP="$(python3 get_title.py $1)"
echo "$(sed "s|[^a-zA-Z0-9 ]||g;s|  | |g" <<< $RESP)"
exit
