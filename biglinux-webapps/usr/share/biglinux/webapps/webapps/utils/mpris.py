"""Optional MPRIS2 D-Bus integration for media webapps.

Bridges web Media Session API → MPRIS2 D-Bus interface so
desktop media keys (play/pause/next/prev) work with webapps
like Spotify, YouTube Music, Tidal, etc.

Requires: dbus-python (python3-dbus). Degrades gracefully if missing.
"""

import threading

# flag for conditional import
MPRIS_AVAILABLE = False

try:
    import dbus
    import dbus.service
    import dbus.mainloop.glib
    from gi.repository import GLib

    MPRIS_AVAILABLE = True
except ImportError:
    pass


# JS injected into web pages to poll Media Session state
MEDIA_SESSION_JS = """
(function() {
    if (window._bigwebapp_mpris) return;
    window._bigwebapp_mpris = true;

    var channel = null;
    if (typeof QWebChannel !== 'undefined') {
        new QWebChannel(qt.webChannelTransport, function(ch) {
            channel = ch.objects.mpris;
        });
    }

    function send(data) {
        if (channel && channel.updateState) {
            channel.updateState(JSON.stringify(data));
        }
    }

    // poll navigator.mediaSession
    setInterval(function() {
        var ms = navigator.mediaSession;
        if (!ms) return;
        var meta = ms.metadata;
        send({
            state: ms.playbackState || 'none',
            title: meta ? meta.title : '',
            artist: meta ? meta.artist : '',
            album: meta ? meta.album : '',
            artwork: (meta && meta.artwork && meta.artwork.length)
                ? meta.artwork[meta.artwork.length - 1].src : ''
        });
    }, 1500);
})();
"""

if MPRIS_AVAILABLE:

    MPRIS_IFACE = "org.mpris.MediaPlayer2"
    PLAYER_IFACE = "org.mpris.MediaPlayer2.Player"

    class MprisService(dbus.service.Object):
        """Minimal MPRIS2 D-Bus service for a webapp."""

        def __init__(self, app_id: str, app_name: str):
            dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
            bus = dbus.SessionBus()
            bus_name = dbus.service.BusName(
                f"org.mpris.MediaPlayer2.bigwebapp.{app_id}", bus
            )
            super().__init__(bus_name, "/org/mpris/MediaPlayer2")

            self.app_name = app_name
            self.app_id = app_id
            self._state = "Stopped"
            self._metadata: dict = {}
            self._play_cb = None
            self._pause_cb = None
            self._next_cb = None
            self._prev_cb = None

        def set_callbacks(self, play=None, pause=None, next_=None, prev=None):
            self._play_cb = play
            self._pause_cb = pause
            self._next_cb = next_
            self._prev_cb = prev

        def update_from_web(self, state_json: str):
            """Called from QWebChannel with JSON media state."""
            import json

            try:
                data = json.loads(state_json)
            except (json.JSONDecodeError, TypeError):
                return

            web_state = data.get("state", "none")
            new_state = {
                "playing": "Playing",
                "paused": "Paused",
            }.get(web_state, "Stopped")

            changed = new_state != self._state
            self._state = new_state

            meta = {}
            title = data.get("title", "")
            if title:
                meta["xesam:title"] = title
            artist = data.get("artist", "")
            if artist:
                meta["xesam:artist"] = dbus.Array([artist], signature="s")
            album = data.get("album", "")
            if album:
                meta["xesam:album"] = album
            artwork = data.get("artwork", "")
            if artwork:
                meta["mpris:artUrl"] = artwork

            if meta != self._metadata or changed:
                self._metadata = meta
                self.PropertiesChanged(
                    PLAYER_IFACE,
                    {
                        "PlaybackStatus": self._state,
                        "Metadata": dbus.Dictionary(self._metadata, signature="sv"),
                    },
                    [],
                )

        # --- org.mpris.MediaPlayer2 ---

        @dbus.service.method(MPRIS_IFACE)
        def Raise(self):
            pass

        @dbus.service.method(MPRIS_IFACE)
        def Quit(self):
            pass

        # --- org.mpris.MediaPlayer2.Player ---

        @dbus.service.method(PLAYER_IFACE)
        def Play(self):
            if self._play_cb:
                self._play_cb()

        @dbus.service.method(PLAYER_IFACE)
        def Pause(self):
            if self._pause_cb:
                self._pause_cb()

        @dbus.service.method(PLAYER_IFACE)
        def PlayPause(self):
            if self._state == "Playing":
                self.Pause()
            else:
                self.Play()

        @dbus.service.method(PLAYER_IFACE)
        def Next(self):
            if self._next_cb:
                self._next_cb()

        @dbus.service.method(PLAYER_IFACE)
        def Previous(self):
            if self._prev_cb:
                self._prev_cb()

        @dbus.service.method(PLAYER_IFACE)
        def Stop(self):
            self.Pause()

        # --- Properties ---

        @dbus.service.method(
            dbus.PROPERTIES_IFACE, in_signature="ss", out_signature="v"
        )
        def Get(self, interface, prop):
            return self.GetAll(interface).get(prop)

        @dbus.service.method(
            dbus.PROPERTIES_IFACE, in_signature="s", out_signature="a{sv}"
        )
        def GetAll(self, interface):
            if interface == MPRIS_IFACE:
                return {
                    "CanQuit": False,
                    "CanRaise": False,
                    "HasTrackList": False,
                    "Identity": self.app_name,
                    "DesktopEntry": f"br.com.biglinux.webapp.{self.app_id}",
                    "SupportedUriSchemes": dbus.Array([], signature="s"),
                    "SupportedMimeTypes": dbus.Array([], signature="s"),
                }
            if interface == PLAYER_IFACE:
                return {
                    "PlaybackStatus": self._state,
                    "Metadata": dbus.Dictionary(self._metadata, signature="sv"),
                    "CanGoNext": True,
                    "CanGoPrevious": True,
                    "CanPlay": True,
                    "CanPause": True,
                    "CanControl": True,
                }
            return {}

        @dbus.service.signal(dbus.PROPERTIES_IFACE, signature="sa{sv}as")
        def PropertiesChanged(self, interface, changed, invalidated):
            pass

    def start_glib_loop():
        """Run GLib main loop in background thread for D-Bus signals."""
        loop = GLib.MainLoop()
        t = threading.Thread(target=loop.run, daemon=True)
        t.start()
        return loop

else:
    # stubs when dbus not available
    class MprisService:
        def __init__(self, *a, **kw):
            pass

        def set_callbacks(self, **kw):
            pass

        def update_from_web(self, s):
            pass

    MEDIA_SESSION_JS = ""

    def start_glib_loop():
        return None
