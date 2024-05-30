from sys import argv
import json

import gettext
lang_translations = gettext.translation(
    'biglinux-webapps',
    localedir='/usr/share/locale',
    fallback=True
)
lang_translations.install()


if __name__ == '__main__':
    ARGS = argv[1:]

    strings_to_return: list[str] = json.loads(ARGS[0])

    TRANSLATIONS = {}

    for string in strings_to_return:
        TRANSLATIONS[string] = lang_translations.gettext(string)

    print(json.dumps(TRANSLATIONS))
