//! big-webapps-exec — Webapp launcher for BigLinux WebApps.
//!
//! Replaces the former Bash launcher script. Reads browser definitions from
//! `/usr/share/biglinux-webapps/browsers.toml` (embedded fallback when absent),
//! so adding a new browser to the TOML file is sufficient — no recompilation,
//! no changes to this binary.
//!
//! Called by the `Exec=` line of each webapp `.desktop` file:
//! ```text
//! big-webapps-exec filename=<file> <browser-id> \
//!     --class=<class> --profile-directory=<profile> --app=<url>
//! ```
//!
//! X11 and Wayland are both supported. The Wayland desktop-file icon-swap
//! workaround is applied automatically when `XDG_SESSION_TYPE=wayland`.
mod icon;
mod launch;
mod wayland;

use std::process;

const USAGE: &str = "Usage: big-webapps-exec filename=<file> <browser-id> \
    --class=<class> --profile-directory=<profile> --app=<url>";

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args = parse_args().unwrap_or_else(|e| {
        eprintln!("big-webapps-exec: {e}\n{USAGE}");
        process::exit(1);
    });

    // Normalize icon (non-fatal: launch proceeds even if icon handling fails)
    let icon = icon::normalize(&args.filename).unwrap_or_default();

    // Look up browser definition from browsers.toml (native id, flatpak_id, or legacy id)
    let def = webapps_core::browsers::find_def(&args.browser);
    let is_flatpak = args.browser.starts_with("flatpak-");

    // Determine engine family: TOML `firefox_like` field; fallback to string matching
    let is_firefox = def.map_or_else(
        || args.browser.contains("firefox") || args.browser.contains("librewolf"),
        |d| d.firefox_like,
    );

    // Grant Flatpak filesystem access to this browser's data directory
    if is_flatpak {
        if let Some(app_id) = def.and_then(|d| d.flatpak_app_id.as_deref()) {
            launch::grant_flatpak_access(&args.browser, app_id);
        }
    }

    if is_firefox {
        // exec() replaces this process — the function never returns
        launch::firefox(&args, &args.browser, def, &icon, is_flatpak);
    } else {
        launch::chromium(&args, &args.browser, def, is_flatpak);
    }
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

/// Parsed arguments from the `.desktop` `Exec=` line.
pub struct Args {
    /// Desktop file name (e.g. `"Gmail-BigWebApp1.desktop"`).
    pub filename: String,
    /// Browser identifier (e.g. `"brave"`, `"flatpak-firefox"`).
    pub browser: String,
    /// WM class / app-id for the webapp window.
    pub class: String,
    /// Profile name (`"Default"`, `"Browser"`, or a custom name).
    pub profile: String,
    /// Target URL (normalized: always has a scheme).
    pub url: String,
}

fn parse_args() -> Result<Args, String> {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() < 6 {
        return Err(format!(
            "expected 5 arguments, got {}",
            argv.len().saturating_sub(1)
        ));
    }

    let filename = argv[1]
        .strip_prefix("filename=")
        .ok_or("argument 1 must be filename=<file>")?
        .to_string();

    let browser = argv[2].clone();

    let class = argv[3]
        .strip_prefix("--class=")
        .ok_or("argument 3 must be --class=<value>")?
        .to_string();

    let profile = argv[4]
        .strip_prefix("--profile-directory=")
        .ok_or("argument 4 must be --profile-directory=<value>")?
        .to_string();

    let url_raw = argv[5]
        .strip_prefix("--app=")
        .ok_or("argument 5 must be --app=<url>")?;
    let url = normalize_url(url_raw);

    Ok(Args {
        filename,
        browser,
        class,
        profile,
        url,
    })
}

/// Prepend `https://` to URLs that have no scheme.
fn normalize_url(raw: &str) -> String {
    if raw.starts_with("http:") || raw.starts_with("https:") || raw.starts_with("file:") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    }
}
