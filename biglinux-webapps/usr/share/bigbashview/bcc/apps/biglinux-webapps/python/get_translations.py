from os import system
import subprocess

LANG = subprocess.call(["echo", "${LANG:0:2}"])

print(LANG)
