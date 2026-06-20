//! Browser command construction and execution.
//!
//! Firefox-family uses `exec()` to replace the launcher process, ensuring
//! session managers and taskbars track the correct PID.
//! Chromium-family spawns the browser in the background and exits.

use std::{os::unix::process::CommandExt, path::Path, process::Command};

use big_os_kit::subprocess::BigSubprocessSpec;
use webapps_core::{browsers::BrowserDef, config};

use crate::{wayland, Args};

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Launch a Firefox-like browser. Calls `exec()` — never returns normally.
///
/// Sets up the per-webapp profile directory (userChrome.css + user.js) on
/// first run, then replaces the current process with the browser. The Wayland
/// `app_id` is pinned to the `.desktop` basename via `MOZ_APP_REMOTINGNAME` so
/// the window carries the webapp icon and groups under its own taskbar entry.
pub fn firefox(
    args: &Args,
    browser_id: &str,
    def: Option<&'static BrowserDef>,
    icon: &str,
    is_flatpak: bool,
) -> ! {
    let profile_name = args.filename.trim_end_matches(".desktop");
    let profile_dir = config::profiles_dir().join(browser_id).join(profile_name);

    setup_firefox_profile(&profile_dir);

    // XApp accepts either a theme name or an absolute path here; pass the
    // raw Icon= value through so the manager's absolute path (the common
    // case after icon persistence at save time) is honored as-is.
    let Some((program, prefix_args)) = base_spec(def, is_flatpak) else {
        eprintln!(
            "big-webapps-exec: refusing to launch {browser_id:?} — no registered \
             native_paths/flatpak_app_id resolved. Defense-in-depth: not using \
             browser_id as program name."
        );
        std::process::exit(1);
    };
    // bigagents: app-local-subprocess — Firefox path uses exec() to REPLACE the
    // launcher process image (session managers/taskbars must track the browser
    // PID, not a wrapper). BigSubprocessSpec models spawn/run, not execve, so
    // this site stays on std::process::Command by design.
    let mut cmd = Command::new(&program);
    cmd.args(&prefix_args)
        .env("XAPP_FORCE_GTKWINDOW_ICON", icon)
        // Wayland taskbar icon + grouping: the compositor (KDE/GNOME) maps a
        // window back to its `.desktop` (and thus its icon) only when the
        // Wayland `app_id` equals the `.desktop` *basename*. Firefox sets the
        // app_id from MOZ_APP_REMOTINGNAME (Mozilla bug 1859546; app_id must
        // match the desktop name, bug 1826330), so it must be `profile_name`
        // (= the installed `.desktop` basename), NOT the WM_CLASS. `--class`
        // below still carries the WM_CLASS for X11 grouping/StartupWMClass.
        .env("MOZ_APP_REMOTINGNAME", profile_name)
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
    let Some((program, cmd_args)) = build_chromium_spec(args, browser_id, def, is_flatpak) else {
        eprintln!(
            "big-webapps-exec: refusing to launch {browser_id:?} — no registered \
             native_paths/flatpak_app_id resolved. Defense-in-depth: not using \
             browser_id as program name."
        );
        return;
    };

    let spawn = move || {
        if let Err(e) = BigSubprocessSpec::builder()
            .program(program.as_str())
            .args(&cmd_args)
            .build()
            .spawn_detached()
        {
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
    let data_dir = config::profiles_dir().join(browser_id);
    let status = BigSubprocessSpec::builder()
        .program("flatpak")
        .args([
            "override",
            "--user",
            &format!("--filesystem={}", data_dir.display()),
            app_id,
        ])
        .build()
        .run();
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
///
/// Defense-in-depth: never falls back to `browser_id` as the program
/// name. If `def` does not resolve a known program (registered
/// `flatpak_app_id` for Flatpak, or an existing `native_paths` entry
/// for native), the function returns `None`. The caller must reject
/// the launch — passing `browser_id` to `Command::new` would let a
/// malicious or stale registry entry control the executed binary.
fn base_spec(def: Option<&BrowserDef>, is_flatpak: bool) -> Option<(String, Vec<String>)> {
    if is_flatpak {
        let app_id = def.and_then(|d| d.flatpak_app_id.as_deref())?;
        Some((
            "flatpak".to_string(),
            vec!["run".to_string(), app_id.to_string()],
        ))
    } else {
        let exec = def.and_then(|d| d.native_paths.iter().find(|p| Path::new(p).exists()))?;
        Some((exec.clone(), Vec::new()))
    }
}

/// Assemble the complete Chromium argv as `(program, args)`.
///
/// Always uses an isolated `--user-data-dir` so the webapp runs in its own
/// browser process. Sharing the browser's main user-data-dir routes the new
/// window through IPC into the existing process, causing the taskbar to group
/// the webapp under the browser's own entry instead of its own `.desktop`.
///
/// For sentinel profiles "Default" / "Browser" the data dir is per-webapp
/// (keyed by `args.filename`); custom names share their named dir across
/// webapps that opted in.
fn build_chromium_spec(
    args: &Args,
    browser_id: &str,
    def: Option<&BrowserDef>,
    is_flatpak: bool,
) -> Option<(String, Vec<String>)> {
    let (program, mut cmd_args) = base_spec(def, is_flatpak)?;

    let profile_key = if args.profile == "Default" || args.profile == "Browser" {
        // Per-webapp dir so each Default/Browser webapp gets its own process.
        args.filename.trim_end_matches(".desktop")
    } else {
        args.profile.as_str()
    };
    let profile_dir = config::profiles_dir().join(browser_id).join(profile_key);
    let _ = std::fs::create_dir_all(&profile_dir);

    cmd_args.extend([
        "--no-default-browser-check".to_string(),
        "--no-first-run".to_string(),
        format!("--user-data-dir={}", profile_dir.display()),
        format!("--profile-directory={}", args.profile),
        format!("--class={}", args.class),
        format!("--app={}", args.url),
    ]);
    Some((program, cmd_args))
}
