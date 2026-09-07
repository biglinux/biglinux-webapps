use super::*;

#[test]
#[serial]
fn preferred_icons_hide_lower_resolution_candidates() {
    let cache = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CACHE_HOME", cache.path());
    let site = Site::new(HashMap::from([
        ("/", response("text/html", br#"<link rel="icon" href="/small.png"><link rel="icon" href="/medium.png"><link rel="icon" href="/preferred.png">"#.to_vec())),
        ("/small.png", response("image/png", png(64, 0xff0000ff))),
        ("/medium.png", response("image/png", png(255, 0x0000ffff))),
        ("/preferred.png", response("image/png", png(256, 0x00ff00ff))),
    ]));
    let info = webapps_manager::favicon::fetch_site_info(&site.address).unwrap();
    assert_eq!(info.icons.len(), 1);
    assert_eq!(info.icons[0].width, 256);
    assert!(info.icons[0].url.ends_with("/preferred.png"));
}

#[test]
#[serial]
fn fallback_uses_native_size_and_rejects_icons_below_64() {
    let cache = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CACHE_HOME", cache.path());
    for side in [64, 128, 255] {
        let site = Site::new(HashMap::from([
            ("/", response("text/html", br#"<link rel="icon" sizes="1024x1024" href="/fallback.png"><link rel="icon" href="/tiny.png"><link rel="icon" sizes="512x512" href="/broken.png">"#.to_vec())),
            ("/fallback.png", response("image/png", png(side, 0xff0000ff))),
            ("/tiny.png", response("image/png", png(63, 0x0000ffff))),
            ("/broken.png", response("image/png", b"broken image".to_vec())),
        ]));
        let info = webapps_manager::favicon::fetch_site_info(&site.address).unwrap();
        assert_eq!(info.icons.len(), 1);
        assert_eq!(
            (info.icons[0].width, info.icons[0].height),
            (side as u32, side as u32)
        );
        let stored = gdk_pixbuf::Pixbuf::from_file(&info.icons[0].path).unwrap();
        assert_eq!((stored.width(), stored.height()), (side, side));
    }
}

#[test]
#[serial]
fn well_known_large_icon_takes_priority_over_declared_fallback() {
    let cache = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CACHE_HOME", cache.path());
    let site = Site::new(HashMap::from([
        (
            "/",
            response(
                "text/html",
                br#"<link rel="icon" href="/small.png">"#.to_vec(),
            ),
        ),
        ("/small.png", response("image/png", png(64, 0xff0000ff))),
        (
            "/apple-touch-icon.png",
            response("image/png", png(512, 0x00ff00ff)),
        ),
    ]));
    let info = webapps_manager::favicon::fetch_site_info(&site.address).unwrap();
    assert_eq!(info.icons.len(), 1);
    assert_eq!(info.icons[0].width, 512);
    assert!(info.icons[0].url.ends_with("/apple-touch-icon.png"));
}

#[test]
#[serial]
fn scalable_icon_takes_priority_over_raster_fallback() {
    let cache = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CACHE_HOME", cache.path());
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" fill="green"/></svg>"#;
    let site = Site::new(HashMap::from([
        (
            "/",
            response(
                "text/html",
                br#"<link rel="icon" href="/small.png"><link rel="icon" href="/vector.svg">"#
                    .to_vec(),
            ),
        ),
        ("/small.png", response("image/png", png(64, 0xff0000ff))),
        ("/vector.svg", response("image/svg+xml", svg.to_vec())),
    ]));
    let info = webapps_manager::favicon::fetch_site_info(&site.address).unwrap();
    assert_eq!(info.icons.len(), 1);
    assert!(info.icons[0].scalable);
}
