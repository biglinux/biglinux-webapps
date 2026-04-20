use std::path::PathBuf;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
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
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("applications")
}

/// System icons base: /usr/share/biglinux/webapps/icons/
pub fn system_icons_dir() -> PathBuf {
    PathBuf::from("/usr/share/biglinux/webapps/icons")
}

/// Browser profile storage: ~/.local/share/biglinux-webapps/profiles/
pub fn profiles_dir() -> PathBuf {
    data_dir().join("profiles")
}
