//! Shared icon loader for webapp rows and dialogs.
//!
//! The imperative row builder (`build_row` + `RowCallbacks`) was retired by the
//! onda6 migration to the typed Relm4 factory (`relm4_window::row`). Only the
//! icon loader survives here because every surface (factory rows, dialog
//! previews, browser/template pickers) resolves icons the same way.

use gtk4 as gtk;
use gtk4::gdk as gdk4;

use crate::service;

/// Load icon into GtkImage — resolves via theme or file path with crisp SVG
pub fn load_icon(image: &gtk::Image, icon_ref: &str) {
    let resolved = service::resolve_icon_path(icon_ref);
    let p = std::path::Path::new(&resolved);
    if p.is_absolute() && p.exists() {
        if resolved.ends_with(".svg") {
            let target = image.pixel_size().max(32) * 4;
            match gdk_pixbuf::Pixbuf::from_file_at_size(p, target, target) {
                Ok(pixbuf) => {
                    // Build the texture from the pixbuf's own buffer — byte-for-byte
                    // what the deprecated `Texture::for_pixbuf` did internally.
                    let format = if pixbuf.has_alpha() {
                        gdk4::MemoryFormat::R8g8b8a8
                    } else {
                        gdk4::MemoryFormat::R8g8b8
                    };
                    let tex = gdk4::MemoryTexture::new(
                        pixbuf.width(),
                        pixbuf.height(),
                        format,
                        &pixbuf.read_pixel_bytes(),
                        pixbuf.rowstride() as usize,
                    );
                    image.set_paintable(Some(&tex));
                }
                Err(_) => image.set_from_file(Some(p)),
            }
        } else {
            image.set_from_file(Some(p));
        }
    } else {
        image.set_icon_name(Some(&resolved));
    }
}
