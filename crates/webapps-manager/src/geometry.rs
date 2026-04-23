//! Persist and restore window/dialog size across sessions.
//!
//! Each call site picks its own JSON filename inside
//! [`webapps_core::config::config_dir`]. `gtk::Window` is the common ancestor
//! of `adw::ApplicationWindow` and `adw::Window`, so the helpers work for both
//! the main window and dialogs.

use std::path::PathBuf;

#[allow(unused_imports)]
use adw::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;

use webapps_core::config;

const DEFAULT_WIDTH: i32 = 800;
const DEFAULT_HEIGHT: i32 = 650;
const MIN_SAVED: i32 = 200;

/// Build the JSON path for a given filename component.
pub fn geometry_path(filename: &str) -> PathBuf {
    config::config_dir().join(filename)
}

/// Load geometry from JSON into `window`, silently falling back to
/// `(default_width, default_height)` when no state is available.
pub fn load_geometry<W: IsA<gtk::Window>>(
    window: &W,
    path: &PathBuf,
    default_width: i32,
    default_height: i32,
) {
    window.set_default_size(default_width, default_height);

    let Ok(data) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(geo) = serde_json::from_str::<serde_json::Value>(&data) else {
        log::warn!("Geometry parse fail at {}", path.display());
        return;
    };

    let w = geo
        .get("width")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(default_width);
    let h = geo
        .get("height")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(default_height);
    window.set_default_size(w, h);

    if geo
        .get("maximized")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        window.upcast_ref::<gtk::Window>().maximize();
    }
}

/// Save current geometry to JSON. Skips while the window is in fullscreen.
pub fn save_geometry<W: IsA<gtk::Window>>(window: &W, path: &PathBuf) {
    let window_ref = window.upcast_ref::<gtk::Window>();
    if window_ref.is_fullscreen() {
        return;
    }

    let (w, h) = if window_ref.is_maximized() {
        // maximized width/height reflect monitor size; fall back to the last
        // unmaximized default so restoring doesn't trap the user full-screen.
        let (dw, dh) = window_ref.default_size();
        (
            if dw > 0 { dw } else { DEFAULT_WIDTH },
            if dh > 0 { dh } else { DEFAULT_HEIGHT },
        )
    } else {
        (
            window_ref.width().max(MIN_SAVED),
            window_ref.height().max(MIN_SAVED),
        )
    };

    let geo = serde_json::json!({
        "width": w,
        "height": h,
        "maximized": window_ref.is_maximized(),
    });

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(path, geo.to_string()) {
        log::error!("Failed to save geometry to {}: {e}", path.display());
    }
}

/// Attach load-on-build + save-on-close to an `adw::Window` dialog.
pub fn bind_dialog(window: &adw::Window, filename: &str, default_width: i32, default_height: i32) {
    let path = geometry_path(filename);
    load_geometry(window, &path, default_width, default_height);

    let path_for_close = path.clone();
    window.connect_close_request(move |win| {
        save_geometry(win, &path_for_close);
        gtk::glib::Propagation::Proceed
    });
}

/// Attach load-on-build + save-on-close to an `adw::Dialog`.
///
/// `AdwDialog` does not expose the "maximized" concept; only content size
/// is persisted. Floating dialogs are resizable, so we read the allocated
/// size from the widget at close time.
pub fn bind_adw_dialog(
    dialog: &adw::Dialog,
    filename: &str,
    default_width: i32,
    default_height: i32,
) {
    let path = geometry_path(filename);
    load_adw_dialog(dialog, &path, default_width, default_height);

    let path_for_close = path.clone();
    dialog.connect_closed(move |d| {
        save_adw_dialog(d, &path_for_close);
    });
}

fn load_adw_dialog(dialog: &adw::Dialog, path: &PathBuf, default_width: i32, default_height: i32) {
    dialog.set_content_width(default_width);
    dialog.set_content_height(default_height);

    let Ok(data) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(geo) = serde_json::from_str::<serde_json::Value>(&data) else {
        log::warn!("Dialog geometry parse fail at {}", path.display());
        return;
    };
    if let Some(w) = geo.get("width").and_then(|v| v.as_i64()) {
        dialog.set_content_width(w as i32);
    }
    if let Some(h) = geo.get("height").and_then(|v| v.as_i64()) {
        dialog.set_content_height(h as i32);
    }
}

fn save_adw_dialog(dialog: &adw::Dialog, path: &PathBuf) {
    let w = dialog.width().max(MIN_SAVED);
    let h = dialog.height().max(MIN_SAVED);
    let geo = serde_json::json!({ "width": w, "height": h });

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(path, geo.to_string()) {
        log::error!("Failed to save dialog geometry to {}: {e}", path.display());
    }
}
