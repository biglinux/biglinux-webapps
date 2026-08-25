//! Header-level image inspection for downloaded icon candidates.
//!
//! Icon *ranking* used to trust the `sizes=""` attribute in the page HTML and
//! the `sizes` field in `manifest.json`. Both are author-supplied hints and
//! routinely wrong or absent: a `<link rel="icon">` with no `sizes` scored 0 and
//! lost to a declared `32x32`, even when the unlabelled asset was a 512 px PNG.
//! The result was a blurry launcher icon whenever the honest high-res asset
//! happened to be the undeclared one.
//!
//! So we measure instead of trust: every candidate is decoded far enough to read
//! its real pixel dimensions, and ranking runs on that. Parsing the handful of
//! container headers by hand keeps this dependency-free — pulling in `image`
//! would add megabytes to a binary with a 7 MB budget for a job that needs
//! roughly twenty bytes from each file.
use std::path::Path;

/// Raster formats we can measure, plus `Svg` for resolution-independent input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ImageFormat {
    Png,
    Svg,
    Ico,
    Jpeg,
    WebP,
    Gif,
    Unknown,
}

impl ImageFormat {
    /// File extension to store the asset under. Matching the real container
    /// matters because `desktop::persist_icon` derives the persisted icon's
    /// extension from the cached filename — a JPEG saved as `.png` would be
    /// copied to `Icon=/…/webapp-foo.png`, leaving a lie in the `.desktop`.
    pub(super) fn extension(self) -> &'static str {
        match self {
            Self::Png | Self::Unknown => "png",
            Self::Svg => "svg",
            Self::Ico => "ico",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Gif => "gif",
        }
    }

    pub(super) fn is_vector(self) -> bool {
        self == Self::Svg
    }
}

/// Effective resolution used for ranking. Vectors get `VECTOR_SIDE` so they
/// outrank every realistic raster without being treated as literally infinite.
pub(super) const VECTOR_SIDE: u32 = 4096;

pub(super) fn detect_format(bytes: &[u8]) -> ImageFormat {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return ImageFormat::Png;
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return ImageFormat::Jpeg;
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return ImageFormat::Gif;
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return ImageFormat::WebP;
    }
    if is_ico(bytes) {
        return ImageFormat::Ico;
    }
    if looks_like_svg(bytes) {
        return ImageFormat::Svg;
    }
    ImageFormat::Unknown
}

/// An ICO/CUR header is `00 00 <01|02> 00 <count:u16>` with a non-zero count.
/// The magic is weak (four of the six bytes are zero), so the count and the
/// directory length are checked too — otherwise arbitrary binary blobs starting
/// with zeros would be handed to the ICO frame extractor.
fn is_ico(bytes: &[u8]) -> bool {
    if bytes.len() < 22 || bytes[0] != 0 || bytes[1] != 0 || bytes[3] != 0 {
        return false;
    }
    if bytes[2] != 1 && bytes[2] != 2 {
        return false;
    }
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
    count > 0 && bytes.len() >= 6 + count * 16
}

/// SVG detection has to tolerate a leading XML declaration, a doctype, comments
/// and a BOM before the root element, so we scan a prefix for the `<svg` token
/// rather than only matching the first bytes.
fn looks_like_svg(bytes: &[u8]) -> bool {
    let probe_len = bytes.len().min(1024);
    let head = String::from_utf8_lossy(&bytes[..probe_len]).to_lowercase();
    head.contains("<svg")
}

/// Measured extent of an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// How far from 1:1 an asset may be and still count as an icon.
///
/// Aspect ratio is what separates an app icon from an `og:image` share card.
/// Ranking on pixel count alone made a 1200×630 Open Graph banner beat a
/// 512×512 app icon on Discord and Slack — technically the bigger image,
/// visually a wide screenshot squeezed into a launcher tile. Slightly
/// non-square logos (a 1.2:1 wordmark) are still legitimate icons, so the gate
/// is loose rather than exact.
const MAX_ICON_ASPECT_RATIO: f32 = 1.25;

impl Dimensions {
    fn new(width: u32, height: u32) -> Option<Self> {
        (width > 0 && height > 0).then_some(Self { width, height })
    }

    /// Square side the asset can fill: the shorter edge, since a launcher tile
    /// is square and the excess on the long edge is padding, not resolution.
    pub(super) fn side(self) -> u32 {
        self.width.min(self.height)
    }

    pub(super) fn is_square_ish(self) -> bool {
        let long = self.width.max(self.height) as f32;
        let short = self.width.min(self.height) as f32;
        long / short <= MAX_ICON_ASPECT_RATIO
    }
}

/// Pixel extent of the asset, or `None` when the header is unparseable.
/// `None` sorts below every measured candidate.
pub(super) fn measure(bytes: &[u8], format: ImageFormat) -> Option<Dimensions> {
    match format {
        ImageFormat::Svg => Some(svg_dimensions(bytes)),
        ImageFormat::Png => png_dimensions(bytes),
        ImageFormat::Ico => {
            largest_ico_frame(bytes).and_then(|frame| Dimensions::new(frame.side, frame.side))
        }
        ImageFormat::Jpeg => jpeg_dimensions(bytes),
        ImageFormat::WebP => webp_dimensions(bytes),
        ImageFormat::Gif => gif_dimensions(bytes),
        ImageFormat::Unknown => None,
    }
}

/// Convenience wrapper for the common "how big is the square" question.
#[cfg(test)]
fn measure_side(bytes: &[u8], format: ImageFormat) -> Option<u32> {
    measure(bytes, format).map(Dimensions::side)
}

/// A vector renders at any size, so resolution is reported as `VECTOR_SIDE`.
/// The *aspect ratio* still has to come from the document, though: an SVG
/// og:image banner is as wrong for a launcher tile as a PNG one, and reporting
/// every SVG as square would let it through the icon gate.
fn svg_dimensions(bytes: &[u8]) -> Dimensions {
    let probe_len = bytes.len().min(2048);
    let head = String::from_utf8_lossy(&bytes[..probe_len]);
    match svg_aspect_ratio(&head) {
        // Scale the reported extent so `side()` stays at VECTOR_SIDE while the
        // long edge carries the real proportions.
        Some(ratio) if ratio > 1.0 => Dimensions {
            width: (VECTOR_SIDE as f32 * ratio) as u32,
            height: VECTOR_SIDE,
        },
        Some(ratio) if ratio > 0.0 && ratio < 1.0 => Dimensions {
            width: VECTOR_SIDE,
            height: (VECTOR_SIDE as f32 / ratio) as u32,
        },
        // No usable width/height or viewBox: assume square, which is what an
        // SVG with only a `viewBox`-less root and CSS sizing usually is.
        _ => Dimensions {
            width: VECTOR_SIDE,
            height: VECTOR_SIDE,
        },
    }
}

/// Width-to-height ratio from the root `<svg>`'s `width`/`height`, falling back
/// to the `viewBox`. Units (`px`, `pt`, `%`) are ignored — only the ratio
/// matters, and mixed units on the two axes are vanishingly rare.
fn svg_aspect_ratio(head: &str) -> Option<f32> {
    let leading_number = |value: &str| -> Option<f32> {
        let digits: String = value
            .trim()
            .chars()
            .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
            .collect();
        digits.parse::<f32>().ok().filter(|number| *number > 0.0)
    };

    let attribute = |name: &str| -> Option<String> {
        let at = head.find(&format!("{name}=\""))? + name.len() + 2;
        let rest = head.get(at..)?;
        Some(rest[..rest.find('"')?].to_string())
    };

    if let (Some(width), Some(height)) = (
        attribute("width").as_deref().and_then(leading_number),
        attribute("height").as_deref().and_then(leading_number),
    ) {
        return Some(width / height);
    }

    let view_box = attribute("viewBox")?;
    let values: Vec<f32> = view_box
        .split([' ', ','])
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<f32>().ok())
        .collect();
    let [_, _, width, height] = values[..] else {
        return None;
    };
    (width > 0.0 && height > 0.0).then(|| width / height)
}

fn read_u32_be(bytes: &[u8], at: usize) -> Option<u32> {
    let slice = bytes.get(at..at + 4)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

/// PNG: fixed 8-byte signature, then the IHDR chunk whose width/height are
/// big-endian u32 at offsets 16 and 20.
fn png_dimensions(bytes: &[u8]) -> Option<Dimensions> {
    if bytes.get(12..16)? != b"IHDR" {
        return None;
    }
    Dimensions::new(read_u32_be(bytes, 16)?, read_u32_be(bytes, 20)?)
}

fn png_side(bytes: &[u8]) -> Option<u32> {
    png_dimensions(bytes).map(Dimensions::side)
}

fn gif_dimensions(bytes: &[u8]) -> Option<Dimensions> {
    let width = u16::from_le_bytes([*bytes.get(6)?, *bytes.get(7)?]);
    let height = u16::from_le_bytes([*bytes.get(8)?, *bytes.get(9)?]);
    Dimensions::new(u32::from(width), u32::from(height))
}

/// WebP stores dimensions differently per chunk type: `VP8 ` (lossy) packs them
/// as 14-bit fields in the keyframe header, `VP8L` (lossless) as 14-bit fields
/// in a packed u32, and `VP8X` (extended) as two 24-bit "minus one" values.
fn webp_dimensions(bytes: &[u8]) -> Option<Dimensions> {
    let chunk = bytes.get(12..16)?;
    let (width, height) = match chunk {
        b"VP8 " => {
            let width = u16::from_le_bytes([*bytes.get(26)?, *bytes.get(27)?]) & 0x3fff;
            let height = u16::from_le_bytes([*bytes.get(28)?, *bytes.get(29)?]) & 0x3fff;
            (u32::from(width), u32::from(height))
        }
        b"VP8L" => {
            let slice = bytes.get(21..25)?;
            let packed = u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]);
            ((packed & 0x3fff) + 1, ((packed >> 14) & 0x3fff) + 1)
        }
        b"VP8X" => {
            let width = u32::from_le_bytes([*bytes.get(24)?, *bytes.get(25)?, *bytes.get(26)?, 0]);
            let height = u32::from_le_bytes([*bytes.get(27)?, *bytes.get(28)?, *bytes.get(29)?, 0]);
            (width + 1, height + 1)
        }
        _ => return None,
    };
    Dimensions::new(width, height)
}

/// JPEG: walk the marker chain until a Start-Of-Frame carrying the dimensions.
/// Segments without a payload (`D0`–`D9`, `01`) must not consume a length word,
/// and fill bytes (`FF` runs) are skipped.
fn jpeg_dimensions(bytes: &[u8]) -> Option<Dimensions> {
    let mut cursor = 2usize;
    loop {
        while bytes.get(cursor) == Some(&0xff) {
            cursor += 1;
        }
        let marker = *bytes.get(cursor)?;
        cursor += 1;
        if marker == 0x01 || (0xd0..=0xd9).contains(&marker) {
            continue;
        }
        let length = u16::from_be_bytes([*bytes.get(cursor)?, *bytes.get(cursor + 1)?]) as usize;
        // SOF0/1/2/3, 5/6/7, 9/10/11, 13/14/15 — every marker in 0xC0..=0xCF
        // except DHT (0xC4), JPG (0xC8) and DAC (0xCC).
        let is_sof =
            (0xc0..=0xcf).contains(&marker) && marker != 0xc4 && marker != 0xc8 && marker != 0xcc;
        if is_sof {
            let height = u16::from_be_bytes([*bytes.get(cursor + 3)?, *bytes.get(cursor + 4)?]);
            let width = u16::from_be_bytes([*bytes.get(cursor + 5)?, *bytes.get(cursor + 6)?]);
            return Dimensions::new(u32::from(width), u32::from(height));
        }
        if length < 2 {
            return None;
        }
        cursor += length;
    }
}

/// One entry of an ICO directory, resolved to its payload.
pub(super) struct IcoFrame<'a> {
    /// Square side in pixels.
    pub side: u32,
    /// Zero-based directory index, for `magick "ICO:-[index]"`.
    pub index: usize,
    /// Payload when it is a whole embedded PNG (modern ICOs do this for the
    /// larger frames); `None` for BMP-encoded frames, which need a decoder.
    pub embedded_png: Option<&'a [u8]>,
}

/// Pick the highest-resolution frame of a multi-size ICO.
///
/// This is the difference between a crisp and a mushy launcher icon on sites
/// that only ship `favicon.ico`: those files commonly hold 16/32/48 px frames
/// (and sometimes 256 px), and whichever frame a decoder happens to return
/// first is usually the 16 px one. Reading the directory ourselves lets us name
/// the largest frame explicitly.
pub(super) fn largest_ico_frame(bytes: &[u8]) -> Option<IcoFrame<'_>> {
    if !is_ico(bytes) {
        return None;
    }
    let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;

    (0..count)
        .filter_map(|index| {
            let entry = bytes.get(6 + index * 16..6 + index * 16 + 16)?;
            // A stored 0 means 256: the field is a single byte, so 256 does not
            // fit and the format spec overloads zero for it.
            let width = if entry[0] == 0 {
                256
            } else {
                u32::from(entry[0])
            };
            let height = if entry[1] == 0 {
                256
            } else {
                u32::from(entry[1])
            };
            let size = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
            let offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;
            let payload = bytes.get(offset..offset.checked_add(size)?);
            Some(IcoFrame {
                side: width.min(height),
                index,
                embedded_png: payload
                    .filter(|data| data.starts_with(b"\x89PNG\r\n\x1a\n"))
                    .and_then(|data| {
                        // Trust the declared side only for BMP frames, where the
                        // directory is the only source. An embedded PNG carries
                        // its own IHDR, and some encoders leave the directory
                        // byte at a stale value.
                        png_side(data).map(|_| data)
                    }),
            })
        })
        .max_by_key(|frame| {
            frame
                .embedded_png
                .and_then(png_side)
                .unwrap_or(frame.side)
                .max(frame.side)
        })
}

/// Measured extent of a file already on disk, used to re-rank candidates after
/// download and to decide whether the winner needs upscaling.
pub(super) fn measure_file(path: &Path) -> Option<Dimensions> {
    // Only the header is needed; reading the whole file keeps this simple and
    // the inputs are capped at a few MB by the downloader anyway.
    let bytes = std::fs::read(path).ok()?;
    measure(&bytes, detect_format(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid PNG header: signature + IHDR length/type + 64x48.
    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes
    }

    #[test]
    fn detects_and_measures_png() {
        let bytes = png_header(512, 512);
        assert_eq!(detect_format(&bytes), ImageFormat::Png);
        assert_eq!(measure_side(&bytes, ImageFormat::Png), Some(512));
    }

    #[test]
    fn png_side_uses_shorter_edge() {
        let bytes = png_header(1024, 512);
        assert_eq!(measure_side(&bytes, ImageFormat::Png), Some(512));
    }

    #[test]
    fn png_with_zero_dimension_is_rejected() {
        // A 0-px "image" must not win a ranking round as a measured candidate.
        assert_eq!(measure_side(&png_header(0, 32), ImageFormat::Png), None);
    }

    #[test]
    fn truncated_png_header_is_none() {
        let bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR\x00\x00".to_vec();
        assert_eq!(measure_side(&bytes, ImageFormat::Png), None);
    }

    #[test]
    fn detects_svg_behind_xml_declaration() {
        let bytes = br#"<?xml version="1.0"?><!-- c --><svg width="16"></svg>"#;
        assert_eq!(detect_format(bytes), ImageFormat::Svg);
        // Vectors rank as effectively unlimited resolution.
        assert_eq!(measure_side(bytes, ImageFormat::Svg), Some(VECTOR_SIDE));
        assert!(ImageFormat::Svg.is_vector());
    }

    #[test]
    fn square_ish_gate_separates_icons_from_share_banners() {
        let square = Dimensions::new(512, 512).unwrap();
        let wordmark = Dimensions::new(120, 100).unwrap();
        let og_banner = Dimensions::new(1200, 630).unwrap();
        let tall = Dimensions::new(400, 1200).unwrap();

        assert!(square.is_square_ish());
        assert!(wordmark.is_square_ish(), "1.2:1 logos are still icons");
        assert!(!og_banner.is_square_ish(), "1.9:1 is an og:image card");
        assert!(!tall.is_square_ish(), "the gate is direction-agnostic");

        // `side()` is the shorter edge: a launcher tile is square, so the extra
        // width of a banner is padding, not usable resolution.
        assert_eq!(og_banner.side(), 630);
        assert_eq!(tall.side(), 400);
    }

    #[test]
    fn svg_aspect_comes_from_width_height_then_viewbox() {
        let square = br#"<svg width="24" height="24" viewBox="0 0 24 24"/>"#;
        assert!(measure(square, ImageFormat::Svg).unwrap().is_square_ish());

        // Units are stripped; only the ratio matters.
        let square_px = br#"<svg width="48px" height="48px"/>"#;
        assert!(measure(square_px, ImageFormat::Svg)
            .unwrap()
            .is_square_ish());

        // A wide SVG banner must fail the icon gate just like a wide PNG one.
        let wide = br#"<svg width="1200" height="630"/>"#;
        assert!(!measure(wide, ImageFormat::Svg).unwrap().is_square_ish());

        // No width/height: fall back to the viewBox.
        let via_view_box = br#"<svg viewBox="0 0 1200 400" fill="none"/>"#;
        assert!(!measure(via_view_box, ImageFormat::Svg)
            .unwrap()
            .is_square_ish());

        // Neither present: assume square rather than discarding the candidate.
        let bare = br#"<svg xmlns="http://www.w3.org/2000/svg"><path/></svg>"#;
        let bare = measure(bare, ImageFormat::Svg).unwrap();
        assert!(bare.is_square_ish());
        // Resolution stays unlimited regardless of the declared extent.
        assert_eq!(bare.side(), VECTOR_SIDE);
    }

    #[test]
    fn svg_reports_vector_resolution_even_when_declared_tiny() {
        // A 16x16 declaration is a rendering hint, not a resolution limit — the
        // path data scales losslessly, so this must not lose to a 32 px PNG.
        let tiny = br#"<svg width="16" height="16" viewBox="0 0 16 16"/>"#;
        assert_eq!(measure_side(tiny, ImageFormat::Svg), Some(VECTOR_SIDE));
    }

    #[test]
    fn xml_that_is_not_svg_is_unknown() {
        let bytes = br#"<?xml version="1.0"?><rss><channel/></rss>"#;
        assert_eq!(detect_format(bytes), ImageFormat::Unknown);
    }

    #[test]
    fn detects_and_measures_gif() {
        let mut bytes = b"GIF89a".to_vec();
        bytes.extend_from_slice(&128u16.to_le_bytes());
        bytes.extend_from_slice(&96u16.to_le_bytes());
        assert_eq!(detect_format(&bytes), ImageFormat::Gif);
        assert_eq!(measure_side(&bytes, ImageFormat::Gif), Some(96));
    }

    #[test]
    fn detects_and_measures_webp_vp8x() {
        let mut bytes = b"RIFF\x00\x00\x00\x00WEBPVP8X".to_vec();
        bytes.extend_from_slice(&[0; 8]); // chunk size + flags
        bytes.truncate(24);
        bytes.extend_from_slice(&[255, 0, 0]); // canvas width - 1 = 255
        bytes.extend_from_slice(&[127, 0, 0]); // canvas height - 1 = 127
        assert_eq!(detect_format(&bytes), ImageFormat::WebP);
        assert_eq!(measure_side(&bytes, ImageFormat::WebP), Some(128));
    }

    #[test]
    fn detects_and_measures_jpeg_skipping_app_segments() {
        // SOI, APP0 (16-byte payload), then SOF0 declaring 200x300.
        let mut bytes = vec![0xff, 0xd8];
        bytes.extend_from_slice(&[0xff, 0xe0, 0x00, 0x10]);
        bytes.extend_from_slice(&[0; 14]);
        bytes.extend_from_slice(&[0xff, 0xc0, 0x00, 0x11, 0x08]);
        bytes.extend_from_slice(&300u16.to_be_bytes()); // height
        bytes.extend_from_slice(&200u16.to_be_bytes()); // width
        assert_eq!(detect_format(&bytes), ImageFormat::Jpeg);
        assert_eq!(measure_side(&bytes, ImageFormat::Jpeg), Some(200));
    }

    #[test]
    fn jpeg_without_sof_terminates() {
        // Pins the loop against running past the buffer on malformed input.
        let bytes = vec![0xff, 0xd8, 0xff, 0xe0, 0x00, 0x04, 0x00, 0x00];
        assert_eq!(measure_side(&bytes, ImageFormat::Jpeg), None);
    }

    /// ICO with `sides` frames, each pointing at a tiny BMP-ish payload.
    fn ico_with_sides(sides: &[u8]) -> Vec<u8> {
        let count = sides.len();
        let mut bytes = vec![0, 0, 1, 0];
        bytes.extend_from_slice(&(count as u16).to_le_bytes());
        let payload_start = 6 + count * 16;
        for (index, side) in sides.iter().enumerate() {
            bytes.extend_from_slice(&[*side, *side, 0, 0, 1, 0, 32, 0]);
            bytes.extend_from_slice(&4u32.to_le_bytes());
            bytes.extend_from_slice(&((payload_start + index * 4) as u32).to_le_bytes());
        }
        bytes.extend_from_slice(&vec![0u8; count * 4]);
        bytes
    }

    #[test]
    fn largest_ico_frame_picks_biggest_not_first() {
        // Directory order is 16, 48, 32 — the 48 px frame must win even though
        // it is neither the first nor the last entry.
        let bytes = ico_with_sides(&[16, 48, 32]);
        assert_eq!(detect_format(&bytes), ImageFormat::Ico);
        let frame = largest_ico_frame(&bytes).expect("frame");
        assert_eq!(frame.side, 48);
        assert_eq!(frame.index, 1);
        assert_eq!(measure_side(&bytes, ImageFormat::Ico), Some(48));
    }

    #[test]
    fn ico_zero_width_byte_means_256() {
        let bytes = ico_with_sides(&[48, 0]);
        let frame = largest_ico_frame(&bytes).expect("frame");
        assert_eq!(frame.side, 256, "a stored 0 encodes 256 px");
        assert_eq!(frame.index, 1);
    }

    #[test]
    fn ico_embedded_png_is_exposed_and_trusted_over_directory() {
        // Directory claims 32 px but the embedded PNG is really 256 px; the
        // ranking must follow the PNG's own IHDR.
        let png = png_header(256, 256);
        let mut bytes = vec![0, 0, 1, 0, 1, 0];
        bytes.extend_from_slice(&[32, 32, 0, 0, 1, 0, 32, 0]);
        bytes.extend_from_slice(&(png.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&22u32.to_le_bytes());
        bytes.extend_from_slice(&png);
        let frame = largest_ico_frame(&bytes).expect("frame");
        assert_eq!(frame.embedded_png.map(<[u8]>::len), Some(png.len()));
    }

    #[test]
    fn ico_entry_with_out_of_range_offset_is_skipped() {
        let mut bytes = vec![0, 0, 1, 0, 1, 0];
        bytes.extend_from_slice(&[32, 32, 0, 0, 1, 0, 32, 0]);
        bytes.extend_from_slice(&9999u32.to_le_bytes()); // size past EOF
        bytes.extend_from_slice(&22u32.to_le_bytes());
        bytes.extend_from_slice(&[0; 16]);
        // The frame survives (side comes from the directory) but exposes no
        // embedded PNG, so the caller falls back to a real decoder.
        let frame = largest_ico_frame(&bytes).expect("frame");
        assert!(frame.embedded_png.is_none());
    }

    #[test]
    fn zero_count_ico_is_not_an_ico() {
        let bytes = vec![0u8; 32];
        assert_eq!(detect_format(&bytes), ImageFormat::Unknown);
        assert!(largest_ico_frame(&bytes).is_none());
    }

    #[test]
    fn extension_matches_container() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::WebP.extension(), "webp");
        assert_eq!(ImageFormat::Gif.extension(), "gif");
        assert_eq!(ImageFormat::Svg.extension(), "svg");
        assert_eq!(ImageFormat::Ico.extension(), "ico");
        // Unmeasurable bytes default to png: gdk-pixbuf sniffs content, so a
        // wrong-but-plausible extension still renders.
        assert_eq!(ImageFormat::Unknown.extension(), "png");
    }
}
