#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
from os.path import splitext, basename, isdir
from os import mkdir
from shutil import rmtree
import requests
import favicon
from random import randint
from io import BytesIO
from PIL import Image, UnidentifiedImageError


class GetFavicon(object):
    """Get favicon from site"""

    def __init__(self, url):
        self.path = "/tmp/.bigwebicons"
        if isdir(self.path):
            rmtree(self.path)
        mkdir(self.path)
        html = self.get_favicon_site(url)
        print(html, end="")

    def get_favicon_site(self, url):
        try:
            icons = favicon.get(url)
            htm = ""
            num = 0
            if len(icons) > 1:
                for icon in icons:
                    iconSave = self.saveImg(icon.url, 0)
                    htm += """
                    <button class="btn-img-favicon" id="btn-icon-%s">
                      <img src="%s" class="img-max"/>
                    </button>""" % (
                        num,
                        iconSave,
                    )
                    num += 1
                return htm
            else:
                url_icon = icons[0].url
                imgSave = self.saveImg(url_icon, 1)
                return imgSave
        except Exception:
            img = self.img_fallback(url)
            return img

    def img_fallback(self, uri):
        from pycurl import Curl

        random_img = f"{self.path}/favicon-{randint(0, 10000000)}.png"
        crl = Curl()
        url_icon = f"https://t0.gstatic.com/faviconV2?client=SOCIAL&type=FAVICON&fallback_opts=TYPE,SIZE,URL&url={uri}&size=256"
        crl.setopt(crl.URL, url_icon)
        crl.setopt(crl.WRITEDATA, open(random_img, "wb"))
        crl.perform()
        crl.close()
        return random_img

    def saveImg(self, link, qtd):
        base_name = basename(link)
        string, extension = splitext(base_name)
        name = "".join(c for c in string if c.isalnum())
        name_file = f"{self.path}/{name}-{randint(0, 10000000)}.png"
        if qtd == 1:
            name_file = f"/tmp/{name}-{randint(0, 10000000)}.png"

        headers = {
            "User-Agent": "Mozilla/5.0 (X11; Linux x86_64)"
            "AppleWebKit/537.36 (KHTML, like Gecko)"
            "Chrome/107.0.0.0 Safari/537.36"
        }

        try:
            resp = requests.get(link, stream=True, headers=headers)
        except requests.exceptions.RequestException:
            img = self.img_fallback(link)
            return img

        if resp.status_code >= 400:
            img = self.img_fallback(link)
            return img

        try:
            with Image.open(BytesIO(resp.content)) as img:
                if img.verify:
                    img.save(name_file)
                    return name_file
        except UnidentifiedImageError:
            img = self.img_fallback(link)
            return img


if __name__ == "__main__":
    url = sys.argv[1].strip()
    if "http" not in url:
        url = "https://" + url
    GetFavicon(url)
