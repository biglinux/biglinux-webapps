#!/usr/bin/env python3

import requests
import favicon
import sys

domain = str(sys.argv[1])
if 'http' not in domain:
	domain = 'https://'+domain

icons = favicon.get(domain)
icon = icons[0]

https, nameurl = domain.split('://')

if nameurl.split('.')[0] == 'www':
	namefile = nameurl.split('.')[1]
elif nameurl.split('.')[0] == 'web':
	namefile = nameurl.split('.')[1]
else:
	namefile = nameurl.split('.')[0]

response = requests.get(icon.url, stream=True)
with open('/tmp/{}.{}'.format(namefile, icon.format), 'wb') as image:
    image.write(response.content)

print('/tmp/{}.{}'.format(namefile, icon.format))