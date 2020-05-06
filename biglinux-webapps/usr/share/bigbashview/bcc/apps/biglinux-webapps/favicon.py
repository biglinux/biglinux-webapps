#!/usr/bin/env python3
import sys
import os
import bs4
import requests
from urllib.parse import urlparse, urlunparse

def find_icon(domain):
    resp = requests.get("https://{}/".format(domain))
    page = bs4.BeautifulSoup(resp.text, 'html.parser')
    res = "https://{}/favicon.ico".format(domain)
    icons = [e for e in page.find_all(name='link') if 'icon' in e.attrs.get('rel')]
    if icons:
        res = icons[0].attrs.get('href')
    url = urlparse(res, scheme='https')
    if not url.netloc:
        res = urlunparse((url.scheme, domain, url.path, '', '', ''))
    return res

def download(domain, icon_url):
    i = icon_url.find('.', len(icon_url)-4)
    if i>=0:
        ext = icon_url[i+1:]
    else:
        ext = 'ico'
    if domain.split('.')[0] == 'www':
        main = domain.split('.')[1]
    else:
        main = domain.split('.')[0]
    datadir = os.popen('xdg-user-dir PICTURES').read().replace('\n', '')
    fname = "{}/{}.{}".format(datadir, main, ext)
    resp = requests.get(icon_url)
    with open(fname, 'wb') as out:
        out.write(resp.content)
        print(fname)

if __name__=='__main__':
    if len(sys.argv)<2:
        print('Need Url!')
        sys.exit()
    site = sys.argv[1]
    download(site, find_icon(site))
