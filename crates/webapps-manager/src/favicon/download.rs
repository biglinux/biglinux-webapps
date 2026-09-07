#[cfg(test)]
use anyhow::Context;
use anyhow::Result;
#[cfg(test)]
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::process::{Command, Stdio};
use std::time::Duration;

use super::html::IconSource;
use super::image::{self, ImageFormat};
use crate::http_client::{http_get_bytes_capped, RequestHeaders};

/// Hard cap on icon byte size.
///
/// The old 1 MB ceiling silently rejected exactly the assets we most want: a
/// 1024×1024 PNG app icon with a photographic background routinely lands between
/// 1 and 3 MB, so the high-res candidate failed the cap and ranking fell back to
/// a 32 px favicon. 8 MB still bounds a hostile server's ability to fill memory
/// (the read is capped mid-stream, not after buffering) while clearing every
/// realistic icon by a wide margin.
const MAX_ICON_BYTES: usize = 8 * 1024 * 1024;

/// An icon fetched to the cache, with its resolution measured from the bytes
/// rather than taken from the page's `sizes` hint.
#[derive(Debug, Clone)]
pub(super) struct DownloadedIcon {
    pub path: PathBuf,
    pub url: String,
    /// Measured extent; `None` when the container header was unparseable.
    pub dimensions: Option<image::Dimensions>,
    pub is_vector: bool,
    /// Where the URL was advertised. Kept past the download because a
    /// `mask-icon` silhouette and an `og:image` banner must stay demoted no
    /// matter how many pixels they turn out to have.
    pub source: IconSource,
}

impl DownloadedIcon {
    /// Resolution used for ranking. Unmeasurable assets sort below everything
    /// measured but still above nothing at all.
    pub(super) fn effective_side(&self) -> u32 {
        self.dimensions.map_or(0, image::Dimensions::side)
    }

    /// Whether the asset is shaped like an icon. Unmeasurable assets are given
    /// the benefit of the doubt — they are usually favicons whose container we
    /// simply do not parse, not share banners.
    pub(super) fn is_icon_shaped(&self) -> bool {
        self.dimensions.is_none_or(image::Dimensions::is_square_ish)
    }
}

pub(super) fn download_icon(
    url: &str,
    cache_dir: &Path,
    index: usize,
    source: IconSource,
    timeout: Duration,
) -> Result<DownloadedIcon> {
    let response = http_get_bytes_capped(
        url,
        &RequestHeaders::browser(),
        MAX_ICON_BYTES as u64,
        timeout,
    )
    .map_err(|error| anyhow::anyhow!(error))?;

    if !(200..300).contains(&response.status) {
        anyhow::bail!("HTTP {}", response.status);
    }

    if let Some(content_type) = response.content_type.as_deref() {
        reject_non_image(content_type)?;
    }

    let bytes = response.bytes;
    if bytes.is_empty() {
        anyhow::bail!("Empty response");
    }

    let format = image::detect_format(&bytes);
    if format == ImageFormat::Ico {
        let mut icon = store_ico(&bytes, cache_dir, index, source)?;
        let bytes = std::fs::read(&icon.path)?;
        icon.dimensions = Some(validate_image(&bytes, image::detect_format(&bytes))?);
        icon.url = response.final_url;
        return Ok(icon);
    }

    let dimensions = Some(validate_image(&bytes, format)?);
    let path = cache_dir.join(format!("icon_{index}.{}", format.extension()));
    std::fs::write(&path, &bytes)?;
    Ok(DownloadedIcon {
        path,
        url: response.final_url,
        dimensions,
        is_vector: format.is_vector(),
        source,
    })
}

/// Strict allowlist: `image/*` covers png/jpeg/webp/svg/x-icon, plus explicit
/// `application/octet-stream` (some CDNs serve favicons that way). Reject
/// anything else — accepting "could be an image" responses expanded the parser
/// attack surface unnecessarily.
fn reject_non_image(content_type: &str) -> Result<()> {
    let content_type = content_type.to_lowercase();
    let acceptable = content_type.starts_with("image/")
        || content_type == "application/octet-stream"
        || content_type.starts_with("application/octet-stream;")
        || content_type.starts_with("application/vnd.microsoft.icon");
    if acceptable {
        Ok(())
    } else {
        anyhow::bail!("Not an image: {content_type}")
    }
}

/// Store an ICO as a PNG of its **largest** frame.
///
/// Writing the `.ico` verbatim leaves frame choice to gdk-pixbuf, which returns
/// whichever frame it decodes first — in practice the 16 px one, even when the
/// same file carries a 48 or 256 px frame. Since `favicon.ico` is the only icon
/// many sites publish, picking the frame ourselves is often the whole difference
/// between a sharp and a blurry launcher entry.
fn store_ico(
    bytes: &[u8],
    cache_dir: &Path,
    index: usize,
    source: IconSource,
) -> Result<DownloadedIcon> {
    let png_path = cache_dir.join(format!("icon_{index}.png"));
    let frame = image::largest_ico_frame(bytes);
    // ICO frames are square by definition, so the frame side describes both axes.
    let square = |side: u32| image::Dimensions {
        width: side,
        height: side,
    };

    // Fast path: modern ICOs embed the large frames as complete PNG streams, so
    // the payload can be lifted out byte-for-byte with no decoder at all.
    if let Some(png) = frame.as_ref().and_then(|frame| frame.embedded_png) {
        std::fs::write(&png_path, png)?;
        return Ok(DownloadedIcon {
            path: png_path,
            url: String::new(),
            dimensions: image::measure(png, ImageFormat::Png),
            is_vector: false,
            source,
        });
    }

    let frame = frame.ok_or_else(|| anyhow::anyhow!("Invalid ICO directory"))?;
    let header = 6 + frame.index * 16;
    let length = u32::from_le_bytes(bytes[header + 8..header + 12].try_into()?) as usize;
    let offset = u32::from_le_bytes(bytes[header + 12..header + 16].try_into()?) as usize;
    let payload = bytes
        .get(
            offset
                ..offset
                    .checked_add(length)
                    .ok_or_else(|| anyhow::anyhow!("ICO overflow"))?,
        )
        .ok_or_else(|| anyhow::anyhow!("Truncated ICO frame"))?;
    let mut single = vec![0, 0, 1, 0, 1, 0];
    single.extend_from_slice(&bytes[header..header + 12]);
    single.extend_from_slice(&22u32.to_le_bytes());
    single.extend_from_slice(payload);
    use gdk_pixbuf::prelude::*;
    let loader = gdk_pixbuf::PixbufLoader::new();
    loader.write(&single)?;
    loader.close()?;
    let pixbuf = loader
        .pixbuf()
        .ok_or_else(|| anyhow::anyhow!("ICO decoding failed"))?;
    pixbuf.savev(&png_path, "png", &[])?;
    Ok(DownloadedIcon {
        path: png_path,
        url: String::new(),
        dimensions: Some(square(frame.side)),
        is_vector: false,
        source,
    })
}

#[cfg(test)]
/// Run `magick` with `args`, feeding `stdin_bytes` when non-empty.
///
/// stdin is closed before `wait_with_output` in both paths: ImageMagick reads
/// its input to EOF, so holding the pipe open while waiting for exit would
/// deadlock. Writing to a closed pipe is also tolerated — `magick` exits early
/// on malformed input, and the resulting `EPIPE` must surface as a conversion
/// failure, not a panic.
fn run_magick(stdin_bytes: &[u8], args: &[String]) -> Result<()> {
    let mut command = Command::new("magick");
    command.args(args);
    command.stdin(if stdin_bytes.is_empty() {
        Stdio::null()
    } else {
        Stdio::piped()
    });
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().context("spawn ImageMagick magick")?;
    if !stdin_bytes.is_empty() {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open magick stdin"))?;
        let write_result = stdin.write_all(stdin_bytes);
        drop(stdin);
        if let Err(err) = write_result {
            // Let the exit status below produce the real diagnostic; a broken
            // pipe here just means magick rejected the input first.
            log::debug!("magick stdin write: {err}");
        }
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("magick failed: {}", stderr.trim())
    }
}

fn validate_image(bytes: &[u8], format: ImageFormat) -> Result<image::Dimensions> {
    use gdk_pixbuf::prelude::*;
    if format == ImageFormat::Unknown {
        anyhow::bail!("Unknown image format");
    }
    let loader = gdk_pixbuf::PixbufLoader::new();
    let original = std::rc::Rc::new(std::cell::Cell::new((0, 0)));
    let measured = original.clone();
    loader.connect_size_prepared(move |loader, width, height| {
        measured.set((width, height));
        if width > 4096 || height > 4096 || format.is_vector() {
            loader.set_size(512, 512);
        }
    });
    loader.write(bytes)?;
    loader.close()?;
    let (width, height) = original.get();
    anyhow::ensure!(
        loader.pixbuf().is_some() && width > 0 && height > 0,
        "Invalid image"
    );
    let dimensions = image::Dimensions {
        width: width as u32,
        height: height as u32,
    };
    anyhow::ensure!(dimensions.is_square_ish(), "Image is not icon-shaped");
    if format.is_vector() {
        anyhow::ensure!(
            !String::from_utf8_lossy(bytes)
                .to_lowercase()
                .contains("<image"),
            "Raster embedded in SVG"
        );
        return Ok(image::Dimensions {
            width: image::VECTOR_SIDE,
            height: image::VECTOR_SIDE,
        });
    }
    anyhow::ensure!(
        width <= 4096 && height <= 4096,
        "Image dimensions exceed limit"
    );
    anyhow::ensure!(
        dimensions.side() >= super::MIN_ACCEPTABLE_SIDE,
        "Icon below {} px",
        super::MIN_ACCEPTABLE_SIDE
    );
    Ok(dimensions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn magick_available() -> bool {
        Command::new("magick")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    /// 3-frame ICO (16/48/32 px BMP frames) built by ImageMagick itself, so the
    /// bytes exercise a real-world encoder rather than a hand-rolled header.
    fn real_multi_frame_ico(dir: &Path) -> Option<Vec<u8>> {
        let path = dir.join("multi.ico");
        let args: Vec<String> = vec![
            "-size".into(),
            "48x48".into(),
            "xc:red".into(),
            "-define".into(),
            "icon:auto-resize=48,32,16".into(),
            format!("ICO:{}", path.display()),
        ];
        run_magick(&[], &args).ok()?;
        std::fs::read(&path).ok()
    }

    #[test]
    fn content_type_allowlist() {
        assert!(reject_non_image("image/png").is_ok());
        assert!(reject_non_image("IMAGE/SVG+XML").is_ok());
        assert!(reject_non_image("application/octet-stream").is_ok());
        assert!(reject_non_image("application/octet-stream; charset=binary").is_ok());
        assert!(reject_non_image("application/vnd.microsoft.icon").is_ok());
        assert!(reject_non_image("text/html").is_err());
        assert!(reject_non_image("application/json").is_err());
    }

    #[test]
    fn effective_side_ranks_unmeasured_last() {
        let measured = DownloadedIcon {
            path: PathBuf::new(),
            url: String::new(),
            dimensions: Some(image::Dimensions {
                width: 64,
                height: 64,
            }),
            is_vector: false,
            source: IconSource::Icon,
        };
        let unmeasured = DownloadedIcon {
            path: PathBuf::new(),
            url: String::new(),
            dimensions: None,
            is_vector: false,
            source: IconSource::Icon,
        };
        assert!(measured.effective_side() > unmeasured.effective_side());
    }

    #[test]
    fn store_ico_lifts_embedded_png_without_imagemagick() {
        // Single-frame ICO whose payload is a complete 256 px PNG stream. The
        // fast path must return that PNG verbatim, so this test passes on hosts
        // with no ImageMagick at all.
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&256u32.to_be_bytes());
        png.extend_from_slice(&256u32.to_be_bytes());
        png.extend_from_slice(&[8, 6, 0, 0, 0]);

        let mut ico = vec![0, 0, 1, 0, 1, 0];
        ico.extend_from_slice(&[0, 0, 0, 0, 1, 0, 32, 0]);
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
        ico.extend_from_slice(&22u32.to_le_bytes());
        ico.extend_from_slice(&png);

        let tmp = TempDir::new().unwrap();
        let stored = store_ico(&ico, tmp.path(), 3, IconSource::Icon).expect("stored");
        assert_eq!(stored.path, tmp.path().join("icon_3.png"));
        assert_eq!(stored.effective_side(), 256);
        assert_eq!(std::fs::read(&stored.path).unwrap(), png);
    }

    #[test]
    fn store_ico_picks_largest_frame_of_real_ico() {
        if !magick_available() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        let Some(ico) = real_multi_frame_ico(tmp.path()) else {
            return;
        };
        let stored = store_ico(&ico, tmp.path(), 0, IconSource::Icon).expect("stored");
        // Regression pin for the two magick bugs: a bare `magick -` could not
        // decode ICO at all, and an unaddressed multi-frame input never wrote
        // `icon_0.png`. Both showed up as a 16 px icon or a missing file.
        assert_eq!(stored.path, tmp.path().join("icon_0.png"));
        assert!(stored.path.is_file(), "the reported path must exist");
        assert_eq!(
            stored.effective_side(),
            48,
            "the 48 px frame must win over the 16/32 px frames"
        );
    }

    #[test]
    fn run_magick_reports_failure_for_garbage_input() {
        if !magick_available() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("out.png");
        let result = run_magick(
            b"not an image at all",
            &["ICO:-[0]".into(), format!("PNG32:{}", out.display())],
        );
        assert!(result.is_err(), "garbage input must not report success");
    }
}

#[cfg(test)]
mod quality_tests {
    use super::*;
    fn png(width: i32, height: i32) -> Vec<u8> {
        let pixbuf =
            gdk_pixbuf::Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, width, height).unwrap();
        pixbuf.fill(0x11aa22ff);
        pixbuf.save_to_bufferv("png", &[]).unwrap()
    }
    #[test]
    fn native_quality_boundary_and_shape_are_enforced() {
        for side in [16, 32, 48, 63] {
            assert!(validate_image(&png(side, side), ImageFormat::Png).is_err());
        }
        for side in [64, 128, 180, 255, 256, 512, 1024] {
            assert_eq!(
                validate_image(&png(side, side), ImageFormat::Png)
                    .unwrap()
                    .side(),
                side as u32
            );
        }
        assert!(validate_image(&png(1200, 630), ImageFormat::Png).is_err());
    }
    #[test]
    fn malformed_images_and_raster_svg_are_rejected() {
        let mut bytes = png(512, 512);
        bytes.truncate(40);
        assert!(validate_image(&bytes, ImageFormat::Png).is_err());
        assert!(validate_image(b"not image", ImageFormat::Unknown).is_err());
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" fill="green"/></svg>"#;
        assert!(validate_image(svg, ImageFormat::Svg).is_ok());
        let raster = br#"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><image href="tiny.png" width="512" height="512"/></svg>"#;
        assert!(validate_image(raster, ImageFormat::Svg).is_err());
    }
}
