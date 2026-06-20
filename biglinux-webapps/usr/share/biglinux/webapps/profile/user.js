// BigLinux WebApps — Firefox-family profile overrides.
//
// Copied verbatim into every Firefox/LibreWolf webapp profile on first launch
// (see big-webapps-exec `setup_firefox_profile`). Keep this minimal: it only
// turns a stock Firefox profile into a single-site app window.
//
// Sibling: chrome/userChrome.css collapses the tab strip and nav bar. It loads
// ONLY because of the legacyUserProfileCustomizations pref below — removing
// that line brings the full browser chrome back.

// --- Chrome customization (required for userChrome.css) ---------------------
user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);
user_pref("browser.tabs.inTitlebar", 0);

// --- App-window behaviour ---------------------------------------------------
user_pref("browser.tabs.warnOnClose", false);
user_pref("browser.tabs.warnOnOpen", false);
user_pref("browser.shell.checkDefaultBrowser", false);

// --- First-run / onboarding noise off ---------------------------------------
user_pref("browser.startup.homepage_override.mstone", "ignore");
user_pref("browser.aboutwelcome.enabled", false);
user_pref("browser.messaging-system.whatsNewPanel.enabled", false);
user_pref("browser.aboutConfig.showWarning", false);

// --- Recommendation prompts off ---------------------------------------------
user_pref("browser.newtabpage.activity-stream.asrouter.userprefs.cfr.addons", false);
user_pref("browser.newtabpage.activity-stream.asrouter.userprefs.cfr.features", false);
user_pref("browser.discovery.enabled", false);

// --- Privacy + media --------------------------------------------------------
user_pref("browser.contentblocking.category", "strict");
// Widevine/EME so DRM webapps (Spotify, streaming) play in the external browser.
user_pref("media.eme.enabled", true);
