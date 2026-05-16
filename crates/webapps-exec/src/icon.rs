//! Icon lookup for webapp launchers.
//!
//! Reads the `Icon=` field from the webapp's `.desktop` file and returns a
//! value usable by `XAPP_FORCE_GTKWINDOW_ICON` (Firefox-family launches).
//! For absolute paths we pass the path through unchanged so XApp picks it up
//! verbatim; for theme names we pass the name through. The `.desktop` is no
//! longer rewritten at launch time — the manager owns icon persistence and
//! writes a stable absolute path at save time.

use std::fs;

/// Returns the value to pass via `XAPP_FORCE_GTKWINDOW_ICON`, or `None` if
/// the desktop file or its `Icon=` field cannot be read.
pub fn normalize(filename: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let desktop_path = format!("{home}/.local/share/applications/{filename}");
    let content = fs::read_to_string(&desktop_path).ok()?;

    let icon_line = content.lines().find(|l| l.starts_with("Icon="))?;
    let icon = icon_line.strip_prefix("Icon=")?.trim();
    Some(icon.to_string())
}
