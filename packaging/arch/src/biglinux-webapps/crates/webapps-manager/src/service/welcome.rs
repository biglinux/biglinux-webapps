use std::fs;

use webapps_core::config;

pub fn should_show_welcome() -> bool {
    let flag = config::config_dir().join("welcome_shown.json");
    !flag.exists()
}

pub fn mark_welcome_shown() {
    let dir = config::config_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(dir.join("welcome_shown.json"), "true");
}
