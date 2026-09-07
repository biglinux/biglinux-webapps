use gtk::prelude::*;
use gtk4 as gtk;
use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

#[derive(Clone)]
struct Raster {
    width: i32,
    height: i32,
    alpha: bool,
    stride: usize,
    pixels: glib::Bytes,
}

type CacheKey = (PathBuf, SystemTime, u64, i32);
thread_local! {
    static REQUESTS: RefCell<HashMap<usize, u64>> = RefCell::new(HashMap::new());
}

pub fn load_icon(image: &gtk::Image, icon_ref: &str) {
    static NEXT_REQUEST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let request = NEXT_REQUEST.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let key = image.as_ptr() as usize;
    REQUESTS.with_borrow_mut(|requests| {
        requests.insert(key, request);
    });
    let target = image.pixel_size().max(32) * image.scale_factor().max(2);
    let cancelled = std::rc::Rc::new(std::cell::Cell::new(false));
    let changed = cancelled.clone();
    let notification = image.connect_notify_local(None, move |_, property| {
        if matches!(property.name(), "file" | "paintable" | "icon-name") {
            changed.set(true);
        }
    });
    let weak_image = image.downgrade();
    let icon_ref = icon_ref.to_owned();
    crate::ui_async::run_with_result(
        move || read_icon(&icon_ref, target),
        move |(resolved, raster)| {
            let current = REQUESTS.with_borrow_mut(|requests| {
                if requests.get(&key) == Some(&request) {
                    requests.remove(&key);
                    true
                } else {
                    false
                }
            });
            let Some(image) = weak_image.upgrade() else {
                return;
            };
            image.disconnect(notification);
            if !current || cancelled.get() {
                return;
            }
            if let Some(raster) = raster {
                let format = if raster.alpha {
                    gtk::gdk::MemoryFormat::R8g8b8a8
                } else {
                    gtk::gdk::MemoryFormat::R8g8b8
                };
                let texture = gtk::gdk::MemoryTexture::new(
                    raster.width,
                    raster.height,
                    format,
                    &raster.pixels,
                    raster.stride,
                );
                image.set_paintable(Some(&texture));
            } else {
                image.set_icon_name(Some(&resolved));
            }
        },
    );
}

fn read_icon(icon_ref: &str, target: i32) -> (String, Option<Raster>) {
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, Raster>>> = OnceLock::new();
    let resolved = crate::service::resolve_icon_path(icon_ref);
    let path = PathBuf::from(&resolved);
    let raster = (|| {
        if !path.is_absolute() {
            return None;
        }
        let metadata = std::fs::metadata(&path).ok()?;
        let key = (
            path.clone(),
            metadata.modified().ok()?,
            metadata.len(),
            target,
        );
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        if let Some(raster) = cache.lock().ok()?.get(&key).cloned() {
            return Some(raster);
        }
        let pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale(&path, target, target, true).ok()?;
        let raster = Raster {
            width: pixbuf.width(),
            height: pixbuf.height(),
            alpha: pixbuf.has_alpha(),
            stride: pixbuf.rowstride() as usize,
            pixels: pixbuf.read_pixel_bytes(),
        };
        let mut cache = cache.lock().ok()?;
        if cache.len() >= 128 {
            cache.clear();
        }
        cache.insert(key, raster.clone());
        Some(raster)
    })();
    (resolved, raster)
}
