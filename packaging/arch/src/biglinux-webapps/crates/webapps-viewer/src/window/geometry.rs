//! Window geometry: persistence of size/position across sessions.
use std::path::PathBuf;

#[allow(unused_imports)]
use adw::prelude::*;
use libadwaita as adw;

/// Load window geometry from JSON config. Silently uses defaults on missing/invalid file.
pub(super) fn load_geometry(window: &adw::ApplicationWindow, config_path: &PathBuf) {
    let data = match std::fs::read_to_string(config_path) {
        Ok(d) => d,
        Err(_) => return, // no config yet → use defaults
    };
    match serde_json::from_str::<serde_json::Value>(&data) {
        Ok(geo) => {
            let w = geo.get("width").and_then(|v| v.as_i64()).unwrap_or(1024) as i32;
            let h = geo.get("height").and_then(|v| v.as_i64()).unwrap_or(720) as i32;
            window.set_default_size(w, h);

            if geo
                .get("maximized")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                window.maximize();
            }
        }
        Err(e) => log::warn!("Geometry parse fail: {e}"),
    }
}

/// Save window geometry to JSON config. Skips while in fullscreen.
pub(super) fn save_geometry(window: &adw::ApplicationWindow, config_path: &PathBuf) {
    if window.is_fullscreen() {
        return;
    }

    // use actual allocation → default_size() only returns initial set value
    let (w, h) = if window.is_maximized() {
        // when maximized, fall back to default_size (last unmaximized value)
        let (dw, dh) = window.default_size();
        (
            if dw > 0 { dw } else { 1024 },
            if dh > 0 { dh } else { 720 },
        )
    } else {
        (window.width().max(200), window.height().max(200))
    };
    let geo = serde_json::json!({
        "width": w,
        "height": h,
        "maximized": window.is_maximized(),
    });

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Err(e) = std::fs::write(config_path, geo.to_string()) {
        log::error!("Failed to save geometry: {e}");
    }
}
