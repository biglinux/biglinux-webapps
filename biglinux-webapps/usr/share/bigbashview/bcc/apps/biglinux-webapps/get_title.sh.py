#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
import requests
import re
from bs4 import BeautifulSoup
from urllib.parse import urlparse


def parse_url(uri):
    html_title = urlparse(uri)
    return html_title.hostname.split('.')[0]


def get_title(url):
    headers = {
        'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64)'
        'AppleWebKit/537.36 (KHTML, like Gecko)'
        'Chrome/107.0.0.0 Safari/537.36'
    }

    try:
        resp = requests.get(url, headers=headers, timeout=10)
    except requests.exceptions.RequestException:
        title = parse_url(url)
        return title

    if resp.status_code >= 400:
        title = parse_url(url)
        return title

    try:
        soup = BeautifulSoup(resp.text, features='html.parser')
        html_title_str = soup.title.string.strip()
    except Exception:
        title = parse_url(url)
        return title

    try:
        html_title_enc = html_title_str.encode('latin-1')
        html_title_dec = html_title_enc.decode('utf-8')
        html_title_only_word = re.sub(r'[^\w]', ' ', html_title_dec)
        html_title_no_space = re.sub(r'\s+', ' ', html_title_only_word)
        return html_title_no_space
    except (UnicodeEncodeError, UnicodeDecodeError):
        html_title_only_word = re.sub(r'[^\w]', ' ', html_title_str)
        html_title_no_space = re.sub(r'\s+', ' ', html_title_only_word)
        return html_title_no_space


if __name__ == '__main__':
    url = sys.argv[1].strip()
    if 'http' not in url:
        url = 'https://'+url
    print(get_title(url), end='')
