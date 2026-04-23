//! Wayland taskbar icon workaround for BigLinux WebApps.
//!
//! On Wayland, compositors identify windows by `app-id`, derived from the
//! desktop file name. When a browser launches a webapp it reports the
//! browser's own app-id, not the webapp-specific one. The workaround
//! temporarily replaces the browser's generic desktop file with the
//! webapp-specific file so the compositor picks up the correct icon.
//!
//! A POSIX advisory lock (`flock`) prevents races when multiple webapp
//! instances start at the same time. The OS releases the lock automatically
//! if the process exits unexpectedly, so no stale locks are possible.

use std::{
    fs,
    os::unix::io::{AsRawFd, IntoRawFd},
    path::Path,
    thread,
    time::Duration,
};

/// Settle time given to the compositor to re-read the desktop file after swap.
/// GNOME and KDE pick the new icon up well under this limit on current hardware;
/// slower compositors can override via `BIG_WEBAPPS_SWAP_SETTLE_MS`.
const DEFAULT_SWAP_SETTLE_MS: u64 = 500;
const ENV_SWAP_SETTLE_MS: &str = "BIG_WEBAPPS_SWAP_SETTLE_MS";

fn swap_settle() -> Duration {
    let ms = std::env::var(ENV_SWAP_SETTLE_MS)
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(DEFAULT_SWAP_SETTLE_MS);
    Duration::from_millis(ms)
}

/// Perform the desktop-file icon swap, call `launch`, then restore.
///
/// Falls back to calling `launch` without swapping when the lock is held
/// by another instance of `big-webapps-exec`.
pub fn swap_and_launch<F: FnOnce()>(filename: &str, launch: F) {
    let home = std::env::var("HOME").unwrap_or_default();
    let apps_dir = format!("{home}/.local/share/applications");

    let full_path = format!("{apps_dir}/{filename}");
    let orig_name = original_desktop_name(filename);
    let orig_path = format!("{apps_dir}/{orig_name}");
    let bkp_stem = orig_path.strip_suffix(".desktop").unwrap_or(&orig_path);
    let bkp_path = format!("{bkp_stem}-bkp.desktop");
    let lock_path = format!("{orig_path}.lock");

    // Restore backup left behind by a crashed previous instance
    if Path::new(&bkp_path).exists() {
        let _ = fs::rename(&bkp_path, &orig_path);
    }

    match try_lock(&lock_path) {
        Some(fd) => {
            // Swap: replace generic desktop file with webapp-specific content
            let _ = fs::rename(&orig_path, &bkp_path);
            let _ = fs::copy(&full_path, &orig_path);
            thread::sleep(swap_settle());

            launch();

            thread::sleep(swap_settle());
            let _ = fs::rename(&bkp_path, &orig_path);
            release_lock(fd, &lock_path);
        }
        None => {
            // Another instance holds the lock — launch without icon swap
            launch();
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Derive the "original" desktop file name by stripping the `-BigWebApp<N>` suffix.
///
/// `"Gmail-BigWebApp1.desktop"` → `"Gmail.desktop"`
fn original_desktop_name(filename: &str) -> String {
    if let Some(pos) = filename.find("-BigWebApp") {
        let after = &filename[pos + "-BigWebApp".len()..];
        if after.starts_with(|c: char| c.is_ascii_digit()) {
            return format!("{}.desktop", &filename[..pos]);
        }
    }
    filename.to_string()
}

/// Try to acquire an exclusive advisory lock on `path` (non-blocking).
///
/// Returns the raw fd on success so the caller can release the lock later.
/// Returns `None` if the lock is held by another process (`EWOULDBLOCK`).
fn try_lock(path: &str) -> Option<i32> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .ok()?;

    let fd = file.as_raw_fd();
    // SAFETY: fd is valid and open; flock is async-signal-safe
    let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };

    if ret == 0 {
        // Take ownership of the fd so File's Drop won't close it (which would
        // release the lock prematurely). IntoRawFd consumes self without
        // running Drop, the canonical alternative to mem::forget here.
        Some(file.into_raw_fd())
    } else {
        // File is dropped → fd is closed → no leak. The lock is released as a
        // side-effect of the close, which is the desired behaviour on failure.
        None
    }
}

/// Release the advisory lock, close the fd, and remove the lock file.
fn release_lock(fd: i32, path: &str) {
    // SAFETY: fd ownership was transferred via File::into_raw_fd in try_lock,
    // so closing it here is the unique close of that file descriptor.
    unsafe {
        libc::flock(fd, libc::LOCK_UN);
        libc::close(fd);
    }
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_desktop_name_strips_big_webapp_suffix() {
        assert_eq!(
            original_desktop_name("Gmail-BigWebApp1.desktop"),
            "Gmail.desktop"
        );
    }

    #[test]
    fn original_desktop_name_keeps_unrelated_filenames() {
        assert_eq!(original_desktop_name("Gmail.desktop"), "Gmail.desktop");
    }

    #[test]
    fn original_desktop_name_requires_digit_after_suffix() {
        // "Gmail-BigWebAppish" is not a generated suffix
        assert_eq!(
            original_desktop_name("Gmail-BigWebAppish.desktop"),
            "Gmail-BigWebAppish.desktop"
        );
    }

    #[test]
    fn swap_settle_falls_back_to_default_when_env_missing() {
        // Use a serial guard via a unique env name so other tests don't interfere.
        std::env::remove_var(ENV_SWAP_SETTLE_MS);
        assert_eq!(swap_settle(), Duration::from_millis(DEFAULT_SWAP_SETTLE_MS));
    }

    #[test]
    fn swap_settle_honours_env_override() {
        std::env::set_var(ENV_SWAP_SETTLE_MS, "1234");
        assert_eq!(swap_settle(), Duration::from_millis(1234));
        std::env::remove_var(ENV_SWAP_SETTLE_MS);
    }

    #[test]
    fn swap_settle_ignores_non_numeric_env() {
        std::env::set_var(ENV_SWAP_SETTLE_MS, "not-a-number");
        assert_eq!(swap_settle(), Duration::from_millis(DEFAULT_SWAP_SETTLE_MS));
        std::env::remove_var(ENV_SWAP_SETTLE_MS);
    }
}
