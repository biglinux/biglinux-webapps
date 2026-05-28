use std::path::{Path, PathBuf};

use webapps_core::config;

/// Sizes tried when looking for an icon in `hicolor/<size>x<size>/apps/`.
/// Ordered so mid-range sizes (40px row thumbnails, 48px dialog previews)
/// resolve first.
const SIZE_FALLBACK: &[u32] = &[64, 48, 128, 32, 256, 512, 24, 22, 16];

/// Resolve an icon reference to a display target. Returns either an absolute
/// file path (caller loads via `set_from_file` / `Pixbuf::from_file_at_size`)
/// or a plain icon name (caller defers to `set_icon_name`).
///
/// Lookup order:
///   1. Absolute path supplied by the caller.
///   2. Drop-ins at `~/.local/share/icons/<name>.{svg,png,xpm}`.
///   3. Scalable apps under user / system hicolor.
///   4. Sized apps under user / system hicolor, cycling `SIZE_FALLBACK`.
///   5. The bundled package icon directory.
///   6. Progressive suffix stripping — `google-chrome-stable` retries as
///      `google-chrome`, then `google`, so themes that only carry the
///      shorter name still resolve.
pub fn resolve_icon_path(icon: &str) -> String {
    if icon.is_empty() {
        return "webapp-manager-generic".into();
    }

    if let Some(absolute) = absolute_existing(icon) {
        return absolute;
    }

    for candidate in name_fallback_chain(icon) {
        if let Some(hit) = lookup_icon(&candidate) {
            return hit;
        }
    }

    // Nothing on disk matched; hand the original name back so the caller
    // falls through to `set_icon_name` for a last-chance theme lookup.
    icon.to_string()
}

fn absolute_existing(icon: &str) -> Option<String> {
    let path = Path::new(icon);
    (path.is_absolute() && path.exists()).then(|| icon.to_string())
}

/// Progressive suffix stripping: `foo-bar-baz` → `foo-bar-baz`, `foo-bar`, `foo`.
/// Returning the original name first preserves exact-match priority.
fn name_fallback_chain(icon: &str) -> Vec<String> {
    let parts: Vec<&str> = icon.split('-').collect();
    (1..=parts.len())
        .rev()
        .map(|end| parts[..end].join("-"))
        .collect()
}

fn lookup_icon(name: &str) -> Option<String> {
    let user_icons = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("icons");
    let sys_icons = PathBuf::from("/usr/share/icons");

    // Drop-in: `~/.local/share/icons/<name>.<ext>` (no theme hierarchy).
    if let Some(hit) = first_existing(&user_icons, name, &["svg", "png", "xpm"]) {
        return Some(hit);
    }

    // Scalable apps (user then system) — SVGs render cleanly at any target size.
    let scalable = "hicolor/scalable/apps";
    if let Some(hit) = first_existing(&user_icons.join(scalable), name, &["svg", "png"]) {
        return Some(hit);
    }
    if let Some(hit) = first_existing(&sys_icons.join(scalable), name, &["svg", "png"]) {
        return Some(hit);
    }

    // Sized apps — walk `SIZE_FALLBACK`, trying user before system at each size.
    for size in SIZE_FALLBACK {
        let subdir = format!("hicolor/{size}x{size}/apps");
        if let Some(hit) = first_existing(&user_icons.join(&subdir), name, &["png", "svg"]) {
            return Some(hit);
        }
        if let Some(hit) = first_existing(&sys_icons.join(&subdir), name, &["png", "svg"]) {
            return Some(hit);
        }
    }

    // Package icons directory (ships with the BigLinux webapps install).
    if let Some(hit) = first_existing(&config::system_icons_dir(), name, &["svg", "png"]) {
        return Some(hit);
    }

    None
}

fn first_existing(dir: &Path, name: &str, exts: &[&str]) -> Option<String> {
    for ext in exts {
        let candidate = dir.join(format!("{name}.{ext}"));
        if candidate.exists() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    // Tests use synthetic names prefixed with `zzz-nonexistent-` so they can't
    // collide with anything actually installed under `/usr/share/icons` or the
    // package icons dir. `#[serial]` guards the `XDG_DATA_HOME` mutation.

    #[test]
    fn empty_input_returns_generic() {
        assert_eq!(resolve_icon_path(""), "webapp-manager-generic");
    }

    #[test]
    fn absolute_existing_path_is_returned_verbatim() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("custom.png");
        fs::write(&file, b"").unwrap();
        let result = resolve_icon_path(file.to_str().unwrap());
        assert_eq!(result, file.to_string_lossy());
    }

    #[test]
    #[serial]
    fn resolves_sized_icon_via_hicolor_fallback() {
        let tmp = TempDir::new().unwrap();
        let sized_dir = tmp.path().join("icons/hicolor/64x64/apps");
        fs::create_dir_all(&sized_dir).unwrap();
        let icon = sized_dir.join("zzz-nonexistent-webapp.png");
        fs::write(&icon, b"").unwrap();

        std::env::set_var("XDG_DATA_HOME", tmp.path());
        let result = resolve_icon_path("zzz-nonexistent-webapp");
        std::env::remove_var("XDG_DATA_HOME");

        assert_eq!(result, icon.to_string_lossy());
    }

    #[test]
    #[serial]
    fn progressively_strips_suffix_until_match() {
        let tmp = TempDir::new().unwrap();
        let scalable = tmp.path().join("icons/hicolor/scalable/apps");
        fs::create_dir_all(&scalable).unwrap();
        // Only the shorter name exists on disk.
        let icon = scalable.join("zzz-nonexistent-chrome.svg");
        fs::write(&icon, b"").unwrap();

        std::env::set_var("XDG_DATA_HOME", tmp.path());
        let result = resolve_icon_path("zzz-nonexistent-chrome-stable");
        std::env::remove_var("XDG_DATA_HOME");

        assert_eq!(result, icon.to_string_lossy());
    }

    #[test]
    #[serial]
    fn unresolved_name_falls_through_unchanged() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("XDG_DATA_HOME", tmp.path());
        let result = resolve_icon_path("zzz-nonexistent-icon-xyz");
        std::env::remove_var("XDG_DATA_HOME");

        assert_eq!(result, "zzz-nonexistent-icon-xyz");
    }

    #[test]
    fn name_fallback_chain_walks_suffixes() {
        assert_eq!(
            name_fallback_chain("google-chrome-stable"),
            vec!["google-chrome-stable", "google-chrome", "google"]
        );
        assert_eq!(name_fallback_chain("firefox"), vec!["firefox"]);
    }
}
