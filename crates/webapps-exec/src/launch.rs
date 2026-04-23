//! Browser command construction and execution.
//!
//! Firefox-family uses `exec()` to replace the launcher process, ensuring
//! session managers and taskbars track the correct PID.
//! Chromium-family spawns the browser in the background and exits.

use std::{os::unix::process::CommandExt, path::Path, process::Command};

use webapps_core::browsers::BrowserDef;

use crate::{wayland, Args};

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Launch a Firefox-like browser. Calls `exec()` — never returns normally.
///
/// Sets up the per-webapp profile directory (userChrome.css + user.js) on
/// first run, then replaces the current process with the browser.
pub fn firefox(
    args: &Args,
    browser_id: &str,
    def: Option<&'static BrowserDef>,
    icon: &str,
    is_flatpak: bool,
) -> ! {
    let home = std::env::var("HOME").unwrap_or_default();
    let profile_name = args.filename.trim_end_matches(".desktop");
    let profile_dir = format!("{home}/.bigwebapps/{browser_id}/{profile_name}");

    setup_firefox_profile(Path::new(&profile_dir));

    let icon_stem = Path::new(icon)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| icon.to_string());

    let (program, prefix_args) = base_spec(def, browser_id, is_flatpak);
    let mut cmd = Command::new(&program);
    cmd.args(&prefix_args)
        .env("XAPP_FORCE_GTKWINDOW_ICON", &icon_stem)
        .env("MOZ_APP_REMOTINGNAME", &args.class)
        .arg(format!("--class={}", args.class))
        .arg(format!("--name={profile_name}"))
        .arg("--profile")
        .arg(&profile_dir)
        .arg("--no-remote")
        .arg(&args.url);

    let err = cmd.exec(); // replaces the current process image
    eprintln!("big-webapps-exec: exec failed: {err}");
    std::process::exit(1)
}

/// Launch a Chromium-family browser in the background.
///
/// On Wayland with a `-BigWebApp`-suffixed desktop file, performs the
/// compositor icon-swap workaround before spawning the browser.
pub fn chromium(args: &Args, browser_id: &str, def: Option<&'static BrowserDef>, is_flatpak: bool) {
    let home = std::env::var("HOME").unwrap_or_default();
    let (program, cmd_args) = build_chromium_spec(args, &home, browser_id, def, is_flatpak);

    let spawn = move || {
        if let Err(e) = Command::new(&program).args(&cmd_args).spawn() {
            eprintln!("big-webapps-exec: failed to spawn browser: {e}");
        }
    };

    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    if args.filename.contains("-BigWebApp") && session == "wayland" {
        wayland::swap_and_launch(&args.filename, spawn);
    } else {
        spawn();
    }
}

/// Grant a Flatpak app access to the webapp's data directory.
///
/// Runs `flatpak override --user --filesystem=<path> <app_id>` so the
/// sandboxed browser can read and write its profile data.
pub fn grant_flatpak_access(browser_id: &str, app_id: &str) {
    let home = std::env::var("HOME").unwrap_or_default();
    let data_dir = format!("{home}/.bigwebapps/{browser_id}");
    let status = Command::new("flatpak")
        .args([
            "override",
            "--user",
            &format!("--filesystem={data_dir}"),
            app_id,
        ])
        .status();
    if let Err(e) = status {
        eprintln!("big-webapps-exec: flatpak override failed: {e}");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a per-webapp Firefox profile with the webapp appearance CSS/JS.
///
/// Files are copied from `/usr/share/biglinux/webapps/profile/` only when
/// the `chrome/` subdirectory does not yet exist (first launch).
fn setup_firefox_profile(profile_dir: &Path) {
    let chrome_dir = profile_dir.join("chrome");
    if chrome_dir.exists() {
        return;
    }
    if let Err(e) = std::fs::create_dir_all(&chrome_dir) {
        eprintln!("big-webapps-exec: cannot create Firefox profile: {e}");
        return;
    }
    copy_profile_file(
        "/usr/share/biglinux/webapps/profile/userChrome.css",
        &chrome_dir.join("userChrome.css"),
    );
    copy_profile_file(
        "/usr/share/biglinux/webapps/profile/user.js",
        &profile_dir.join("user.js"),
    );
}

fn copy_profile_file(src: &str, dst: &Path) {
    if let Err(e) = std::fs::copy(src, dst) {
        eprintln!("big-webapps-exec: cannot copy {src}: {e}");
    }
}

/// Build `(program, argv_prefix)` for any browser (Flatpak or native).
///
/// For Flatpak: `("flatpak", ["run", "<app_id>"])`
/// For native: `("<binary_path>", [])`
fn base_spec(
    def: Option<&BrowserDef>,
    browser_id: &str,
    is_flatpak: bool,
) -> (String, Vec<String>) {
    if is_flatpak {
        let app_id = def
            .and_then(|d| d.flatpak_app_id.as_deref())
            .unwrap_or_else(|| browser_id.strip_prefix("flatpak-").unwrap_or(browser_id));
        (
            "flatpak".to_string(),
            vec!["run".to_string(), app_id.to_string()],
        )
    } else {
        let exec = def
            .and_then(|d| d.native_paths.iter().find(|p| Path::new(p).exists()))
            .map(String::as_str)
            .unwrap_or(browser_id);
        (exec.to_string(), Vec::new())
    }
}

/// Assemble the complete Chromium argv as `(program, args)`.
///
/// Profile "Default" / "Browser" → reuse native browser profile.
/// Any other profile name → isolated `--user-data-dir` session.
fn build_chromium_spec(
    args: &Args,
    home: &str,
    browser_id: &str,
    def: Option<&BrowserDef>,
    is_flatpak: bool,
) -> (String, Vec<String>) {
    let (program, mut cmd_args) = base_spec(def, browser_id, is_flatpak);

    let profile_args = if args.profile == "Default" || args.profile == "Browser" {
        vec![
            "--no-default-browser-check".to_string(),
            "--profile-directory=Default".to_string(),
            format!("--app={}", args.url),
        ]
    } else {
        let profile_dir = format!("{home}/.bigwebapps/{browser_id}/{}", args.profile);
        let _ = std::fs::create_dir_all(&profile_dir);
        vec![
            "--no-default-browser-check".to_string(),
            format!("--user-data-dir={profile_dir}"),
            format!("--app={}", args.url),
        ]
    };

    cmd_args.extend(profile_args);
    (program, cmd_args)
}
