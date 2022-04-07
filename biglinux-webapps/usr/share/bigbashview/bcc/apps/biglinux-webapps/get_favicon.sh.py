#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
import os
import favicon
import subprocess

def get_favicon_site(url):
    try:
        icons = favicon.get(url)
        html = ''
        num=0
        if len(icons) > 1:
            for i in icons:
                html += '''
                <button class="btn-img-favicon" id="btn-icon-%s">
                  <img src="%s" class="img-max"/>
                </button>''' % (num, i.url)
                num+=1
        else:
            url_icon = icons[0].url
            html = subprocess.getoutput('./save_favicon.sh.py %s' % url_icon)

        print(html, end='')
    except:
        return

url = sys.argv[1].strip()
if 'https' not in url:
    url = 'https://'+url
get_favicon_site(url)
