use std::path::PathBuf;

/// App version shown in the About dialog and elsewhere.
///
/// At build time, the PKGBUILD or distro maintainer can override this by setting
/// the `BIGLINUX_WEBAPPS_VERSION` env variable — useful so that the date-based
/// `pkgver` (e.g. `26.04.22`) is reflected in the UI instead of the Cargo manifest
/// version, which is updated less frequently.
pub const APP_VERSION: &str = match option_env!("BIGLINUX_WEBAPPS_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};
pub const APP_ID: &str = "br.com.biglinux.webapps";

/// Config dir: ~/.config/biglinux-webapps/
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("biglinux-webapps")
}

/// Data dir: ~/.local/share/biglinux-webapps/
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("biglinux-webapps")
}

/// Cache dir: ~/.cache/biglinux-webapps/
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("biglinux-webapps")
}

/// Desktop files dir: ~/.local/share/applications/
pub fn applications_dir() -> PathBuf {
    host_data_dir().join("applications")
}

/// System icons base: /usr/share/biglinux/webapps/icons/
pub fn system_icons_dir() -> PathBuf {
    share_dir().join("biglinux/webapps/icons")
}

/// Browser profile storage: ~/.bigwebapps/
pub fn profiles_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".bigwebapps")
}

pub fn is_flatpak() -> bool {
    std::path::Path::new("/.flatpak-info").is_file()
}

pub fn share_dir() -> PathBuf {
    if let Some(prefix) = std::env::var_os("BIGLINUX_WEBAPPS_PREFIX") {
        return PathBuf::from(prefix).join("share");
    }
    if is_flatpak() {
        return PathBuf::from("/app/share");
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(prefix) = executable.parent().and_then(std::path::Path::parent) {
            let share = prefix.join("share");
            if share.join("biglinux-webapps/browsers.toml").is_file() {
                return share;
            }
        }
    }
    PathBuf::from("/usr/share")
}

pub fn host_data_dir() -> PathBuf {
    if is_flatpak() {
        if let Some(path) = std::env::var_os("HOST_XDG_DATA_HOME") {
            return PathBuf::from(path);
        }
    }
    dirs::data_dir().unwrap_or_else(|| PathBuf::from("~/.local/share"))
}

pub fn host_command(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    if is_flatpak() {
        let mut command = std::process::Command::new("flatpak-spawn");
        command.args(["--host", "--watch-bus"]).arg(program);
        command
    } else {
        std::process::Command::new(program)
    }
}

pub fn desktop_command(binary: &str) -> String {
    if is_flatpak() {
        format!("flatpak run --command={binary} {APP_ID}")
    } else {
        let share = share_dir();
        if share != std::path::Path::new("/usr/share") {
            if let Some(prefix) = share.parent() {
                let executable = prefix.join("bin").join(binary);
                if executable.is_file() {
                    return format!("\"{}\"", executable.display());
                }
            }
        }
        binary.to_string()
    }
}
