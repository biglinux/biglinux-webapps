#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
import os
from PIL import Image


def resize(img_tmp):
    file_tmp = os.path.basename(img_tmp)
    filename = f"/tmp/{file_tmp.split('.')[0]}.png"
    with Image.open(img_tmp) as img:
        width, height = img.size
        if width > 64:
            size = (64, 64)
            img_new = img.resize(size)
            img_new.save(filename)
        else:
            img.save(filename)
    print(filename, end='')


if __name__ == '__main__':
    im = sys.argv[1].strip()
    resize(im)
