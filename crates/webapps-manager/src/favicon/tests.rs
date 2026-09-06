use super::*;

fn icon(path: &str, side: Option<u32>, is_vector: bool) -> download::DownloadedIcon {
    sourced_icon(path, side, side, is_vector, html::IconSource::Icon)
}

fn sourced_icon(
    path: &str,
    width: Option<u32>,
    height: Option<u32>,
    is_vector: bool,
    source: html::IconSource,
) -> download::DownloadedIcon {
    download::DownloadedIcon {
        path: PathBuf::from(path),
        url: String::new(),
        dimensions: width
            .zip(height)
            .map(|(width, height)| image::Dimensions { width, height }),
        is_vector,
        source,
    }
}

#[test]
fn ranking_puts_highest_measured_resolution_first() {
    let mut icons = vec![
        icon("small.png", Some(32), false),
        icon("huge.png", Some(1024), false),
        icon("medium.png", Some(180), false),
    ];
    rank_by_measured_resolution(&mut icons);
    assert_eq!(
        icons
            .iter()
            .map(|icon| icon.path.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec!["huge.png", "medium.png", "small.png"]
    );
}

#[test]
fn ranking_sinks_unmeasurable_icons_below_measured_ones() {
    // An asset whose header we could not parse might be anything; a known
    // 16 px PNG is at least a known quantity.
    let mut icons = vec![
        icon("mystery.png", None, false),
        icon("tiny.png", Some(16), false),
    ];
    rank_by_measured_resolution(&mut icons);
    assert_eq!(icons[0].path, PathBuf::from("tiny.png"));
}

#[test]
fn ranking_prefers_vector_when_effective_sides_tie() {
    let mut icons = vec![
        icon("raster.png", Some(image::VECTOR_SIDE), false),
        icon("vector.svg", Some(image::VECTOR_SIDE), true),
    ];
    rank_by_measured_resolution(&mut icons);
    assert_eq!(icons[0].path, PathBuf::from("vector.svg"));
}

#[test]
fn ranking_rejects_og_image_banner_in_favour_of_smaller_square_icon() {
    // Regression pin for Discord/Slack: a 1200x630 share card is the biggest
    // image on the page and the worst possible launcher icon.
    let mut icons = vec![
        sourced_icon(
            "banner.png",
            Some(1200),
            Some(630),
            false,
            html::IconSource::OgImage,
        ),
        sourced_icon(
            "icon-512.png",
            Some(512),
            Some(512),
            false,
            html::IconSource::Manifest,
        ),
    ];
    rank_by_measured_resolution(&mut icons);
    assert_eq!(icons[0].path, PathBuf::from("icon-512.png"));
}

#[test]
fn ranking_rejects_square_og_image_in_favour_of_smaller_real_icon() {
    // Even a perfectly square og:image is a share card, not a logo, so the
    // shape gate alone is not enough — provenance has to outrank pixels.
    let mut icons = vec![
        sourced_icon(
            "square-banner.png",
            Some(1024),
            Some(1024),
            false,
            html::IconSource::OgImage,
        ),
        sourced_icon(
            "icon-64.png",
            Some(64),
            Some(64),
            false,
            html::IconSource::Icon,
        ),
    ];
    rank_by_measured_resolution(&mut icons);
    assert_eq!(icons[0].path, PathBuf::from("icon-64.png"));
}

#[test]
fn ranking_sinks_monochrome_mask_icon_svg_below_a_small_colour_icon() {
    // A mask-icon SVG scores unlimited resolution; its tier must still lose
    // to any real full-colour icon.
    let mut icons = vec![
        sourced_icon(
            "mask.svg",
            Some(image::VECTOR_SIDE),
            Some(image::VECTOR_SIDE),
            true,
            html::IconSource::MaskIcon,
        ),
        sourced_icon(
            "icon-32.png",
            Some(32),
            Some(32),
            false,
            html::IconSource::Icon,
        ),
    ];
    rank_by_measured_resolution(&mut icons);
    assert_eq!(icons[0].path, PathBuf::from("icon-32.png"));
}

#[test]
fn ranking_still_accepts_a_banner_when_it_is_all_there_is() {
    // Demotion must not become exclusion: a share card beats no icon.
    let mut icons = vec![sourced_icon(
        "banner.png",
        Some(1200),
        Some(630),
        false,
        html::IconSource::OgImage,
    )];
    rank_by_measured_resolution(&mut icons);
    assert_eq!(icons[0].path, PathBuf::from("banner.png"));
}

#[test]
fn best_side_ignores_banners_so_probing_still_runs() {
    assert_eq!(best_side(&[]), 0);
    assert_eq!(best_side(&[icon("x.png", None, false)]), 0);
    // A page offering only a 32 px favicon must still be probed for an
    // unlisted apple-touch-icon.
    assert!(best_side(&[icon("x.png", Some(32), false)]) < MIN_ACCEPTABLE_SIDE);
    assert!(best_side(&[icon("x.png", Some(512), false)]) >= MIN_ACCEPTABLE_SIDE);
    // A huge og:image must not count as "we already have a good icon".
    let banner = sourced_icon(
        "banner.png",
        Some(1200),
        Some(630),
        false,
        html::IconSource::OgImage,
    );
    assert_eq!(best_side(&[banner]), 0);
}
