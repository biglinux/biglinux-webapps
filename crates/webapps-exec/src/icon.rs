//! Icon path normalization for webapp launchers.
//!
//! Reads the `Icon=` field from the webapp's `.desktop` file. When the value
//! is an absolute path, copies the file to the user icons directory and
//! rewrites the `.desktop` entry to use only the stem name, so icon themes
//! can locate the icon without knowing the full path.

use std::{fs, path::Path};

/// Normalize the icon for a webapp desktop file.
///
/// Returns the icon name (stem, no path, no extension) to pass to the browser
/// via `XAPP_FORCE_GTKWINDOW_ICON`. Returns an empty string on any error.
pub fn normalize(filename: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let desktop_path = format!("{home}/.local/share/applications/{filename}");
    let content = fs::read_to_string(&desktop_path).ok()?;

    let icon_line = content.lines().find(|l| l.starts_with("Icon="))?;
    let icon = icon_line.strip_prefix("Icon=")?.trim();

    // Absolute path: copy to the icons dir and rewrite .desktop to use the stem
    if icon.contains('/') && Path::new(icon).is_file() {
        let icon_path = Path::new(icon);
        let file_name = icon_path.file_name()?.to_string_lossy();
        let stem = icon_path.file_stem()?.to_string_lossy().to_string();
        let dest = format!("{home}/.local/share/icons/{file_name}");

        if should_copy(icon, &dest) {
            if let Err(e) = fs::copy(icon, &dest) {
                eprintln!("big-webapps-exec: cannot copy icon: {e}");
            }
        }

        // Rewrite Icon= to stem only so icon themes can locate it by name
        let new_content = content.replacen(icon_line, &format!("Icon={stem}"), 1);
        let _ = fs::write(&desktop_path, new_content);

        return Some(stem);
    }

    Some(icon.to_string())
}

/// `true` when `src` should be copied to `dst` (dst missing or sizes differ).
///
/// Uses file size as an approximation of `cmp -s` to avoid reading full files.
fn should_copy(src: &str, dst: &str) -> bool {
    let Ok(dst_meta) = fs::metadata(dst) else {
        return true; // dst does not exist
    };
    let src_len = fs::metadata(src).map(|m| m.len()).unwrap_or(0);
    src_len != dst_meta.len()
}
