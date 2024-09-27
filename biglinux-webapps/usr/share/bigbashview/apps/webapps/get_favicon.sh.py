#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
import os
import requests
import favicon
from random import randint
from io import BytesIO
from PIL import Image, UnidentifiedImageError
import json
import shutil
import pycurl


class GetFavicon:
    """Class to retrieve favicons from a website"""

    def __init__(self, url):
        self.path = "/tmp/.bigwebicons"
        self.prepare_directory()
        favicons = self.get_favicons(url)
        print(json.dumps(favicons))

    def prepare_directory(self):
        """Prepare the temporary directory for storing favicons"""
        if os.path.isdir(self.path):
            shutil.rmtree(self.path)
        os.mkdir(self.path)

    def get_favicons(self, url):
        """Get favicons from the given URL"""
        favicons = [self.save_image(f"{url.rstrip('/')}/favicon.ico")]
        try:
            icons = favicon.get(url)
            for icon in icons[:5]:  # Limita a quantidade de ícones a 5 além do favicon.ico
                favicons.append(self.save_image(icon.url))
            return favicons
        except Exception:
            fallback_icon = self.fetch_fallback_icon(url)
            return [fallback_icon]

    def fetch_fallback_icon(self, url):
        """Fetch a fallback icon using Google's favicon service"""
        fallback_url = (
            f"https://t0.gstatic.com/faviconV2?client=SOCIAL&type=FAVICON&fallback_opts=TYPE,SIZE,URL&url={url}&size=128"
        )
        random_img_path = os.path.join(self.path, f"favicon-{randint(0, 10000000)}.png")
        with open(random_img_path, 'wb') as f:
            self.download_with_pycurl(fallback_url, f)
        return random_img_path

    def download_with_pycurl(self, url, file):
        """Download a file using pycurl"""
        curl = pycurl.Curl()
        curl.setopt(curl.URL, url)
        curl.setopt(curl.WRITEDATA, file)
        curl.perform()
        curl.close()

    def save_image(self, url):
        """Save the image from the URL to the local filesystem"""
        headers = {
            'User-Agent': 'Mozilla/5.0 (X11; Linux x86_64) '
                          'AppleWebKit/537.36 (KHTML, like Gecko) '
                          'Chrome/107.0.0.0 Safari/537.36'
        }
        try:
            response = requests.get(url, stream=True, headers=headers)
            response.raise_for_status()
        except requests.RequestException:
            return self.fetch_fallback_icon(url)

        try:
            with Image.open(BytesIO(response.content)) as img:
                img.verify()
                img_path = os.path.join(self.path, f"{self.clean_filename(url)}-{randint(0, 10000000)}.png")
                img.save(img_path)
                return img_path
        except (UnidentifiedImageError, Exception):
            return self.fetch_fallback_icon(url)

    def clean_filename(self, url):
        """Clean the URL to create a valid filename"""
        base_name = os.path.basename(url)
        name, _ = os.path.splitext(base_name)
        return ''.join(c for c in name if c.isalnum())


if __name__ == '__main__':
    url = sys.argv[1].strip()
    if not url.startswith(('http://', 'https://')):
        url = 'https://' + url
    GetFavicon(url)
