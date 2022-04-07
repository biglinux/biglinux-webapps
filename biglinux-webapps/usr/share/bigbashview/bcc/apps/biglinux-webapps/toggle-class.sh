#!/usr/bin/env bash

[ ! -e ~/.bigwebapps/LIGHT ] && {
    > ~/.bigwebapps/LIGHT
} || {
    rm ~/.bigwebapps/LIGHT
}

exit
