"""
Translation utility module to ensure consistent translations throughout the application
"""

import gettext

gettext.textdomain("biglinux-webapps")
_ = gettext.gettext
