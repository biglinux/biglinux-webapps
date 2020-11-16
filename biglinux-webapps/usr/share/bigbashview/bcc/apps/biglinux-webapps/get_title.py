#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
import requests
from bs4 import BeautifulSoup

url = str(sys.argv[1])
if 'http' not in url:
	url = 'https://'+url
r = requests.get(url)
html = BeautifulSoup(r.content, 'html.parser')
print(html.title.string)