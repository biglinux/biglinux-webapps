use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
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

/// Side length the winning icon is normalised to when its source is smaller.
///
/// Launchers draw app icons at 96–128 px and HiDPI doubles that. Handing the
/// shell a 32 px file forces *it* to upscale, with a fast bilinear filter and no
/// sharpening — the mushy result in the bug report. Producing a 512 px PNG
/// ourselves with Lanczos does not invent detail, but it does put the resampling
/// under our control and stops every downstream consumer from redoing it badly.
pub(super) const TARGET_SIDE: u32 = 512;

/// An icon fetched to the cache, with its resolution measured from the bytes
/// rather than taken from the page's `sizes` hint.
#[derive(Debug, Clone)]
pub(super) struct DownloadedIcon {
    pub path: PathBuf,
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
) -> Result<DownloadedIcon> {
    let response = http_get_bytes_capped(
        url,
        &RequestHeaders::browser(),
        MAX_ICON_BYTES as u64,
        Duration::from_secs(5),
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
        return store_ico(&bytes, cache_dir, index, source);
    }

    let dimensions = image::measure(&bytes, format);
    let path = cache_dir.join(format!("icon_{index}.{}", format.extension()));
    std::fs::write(&path, &bytes)?;
    Ok(DownloadedIcon {
        path,
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
            dimensions: image::measure(png, ImageFormat::Png),
            is_vector: false,
            source,
        });
    }

    if let Some(frame) = frame.as_ref() {
        if extract_ico_frame_with_magick(bytes, frame.index, &png_path).is_ok() {
            let dimensions = image::measure_file(&png_path).or_else(|| Some(square(frame.side)));
            return Ok(DownloadedIcon {
                path: png_path,
                dimensions,
                is_vector: false,
                source,
            });
        }
    }

    // No ImageMagick (or it choked): keep the raw `.ico`. gdk-pixbuf can still
    // render it, just without our control over frame selection.
    let ico_path = cache_dir.join(format!("icon_{index}.ico"));
    std::fs::write(&ico_path, bytes)?;
    Ok(DownloadedIcon {
        path: ico_path,
        dimensions: frame.map(|frame| square(frame.side)),
        is_vector: false,
        source,
    })
}

/// Decode one addressed frame of an ICO to a PNG.
///
/// Two details here are load-bearing and were both wrong before:
///
///  * The input **must** carry an explicit `ICO:` format prefix. ImageMagick
///    stages piped stdin in an extension-less temp file and sniffs the format
///    from the name, so a bare `magick -` fails outright with "no decode
///    delegate" on every single ICO — the conversion never ran in production.
///  * A frame index **must** be selected. Given a multi-frame input and a
///    single-image output format, ImageMagick writes `icon_0-0.png`,
///    `icon_0-1.png`, … and never the requested `icon_0.png`, so the caller
///    would report success while pointing at a file that does not exist.
fn extract_ico_frame_with_magick(bytes: &[u8], frame: usize, png_path: &Path) -> Result<()> {
    run_magick(
        bytes,
        &[
            format!("ICO:-[{frame}]"),
            format!("PNG32:{}", png_path.display()),
        ],
    )?;
    // The frame selector should guarantee a single output file, but a stale or
    // patched ImageMagick that still numbers its output would leave us reporting
    // success for a missing path.
    if png_path.is_file() {
        Ok(())
    } else {
        anyhow::bail!("magick produced no file at {}", png_path.display())
    }
}

/// Resample `source` up to `TARGET_SIDE` and return the new path, or `None` when
/// the upscale is unnecessary or ImageMagick is unavailable.
///
/// `-background none` plus a centred `-extent` keeps non-square logos square and
/// transparent instead of stretched, which is what the freedesktop icon spec and
/// every launcher grid expect.
pub(super) fn upscale_to_target(source: &Path, cache_dir: &Path) -> Option<PathBuf> {
    let target = cache_dir.join("icon_hires.png");
    let args = [
        // No `FORMAT:` read hint: the extension is now derived from the real
        // container magic, so ImageMagick's own sniffing is correct, and forcing
        // `PNG:` would break a legitimately-webp or -jpeg source.
        source.display().to_string(),
        "-filter".into(),
        "Lanczos".into(),
        // A single fit-in-box resize both enlarges and shrinks; callers only
        // reach here for sources below the target, so this always enlarges.
        "-resize".into(),
        format!("{TARGET_SIDE}x{TARGET_SIDE}"),
        "-background".into(),
        "none".into(),
        "-gravity".into(),
        "center".into(),
        "-extent".into(),
        format!("{TARGET_SIDE}x{TARGET_SIDE}"),
        format!("PNG32:{}", target.display()),
    ];
    match run_magick(&[], &args) {
        Ok(()) if target.is_file() => Some(target),
        Ok(()) => None,
        Err(err) => {
            log::warn!("Upscale {} to {TARGET_SIDE}px: {err}", source.display());
            None
        }
    }
}

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
            dimensions: Some(image::Dimensions {
                width: 64,
                height: 64,
            }),
            is_vector: false,
            source: IconSource::Icon,
        };
        let unmeasured = DownloadedIcon {
            path: PathBuf::new(),
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
    fn upscale_raises_small_source_to_target() {
        if !magick_available() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        let small = tmp.path().join("small.png");
        let args: Vec<String> = vec![
            "-size".into(),
            "32x32".into(),
            "xc:blue".into(),
            format!("PNG:{}", small.display()),
        ];
        run_magick(&[], &args).unwrap();

        let upscaled = upscale_to_target(&small, tmp.path()).expect("upscaled");
        assert_eq!(
            image::measure_file(&upscaled).map(image::Dimensions::side),
            Some(TARGET_SIDE)
        );
    }

    #[test]
    fn upscale_pads_non_square_source_instead_of_stretching() {
        if !magick_available() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        let wide = tmp.path().join("wide.png");
        let args: Vec<String> = vec![
            "-size".into(),
            "200x50".into(),
            "xc:green".into(),
            format!("PNG:{}", wide.display()),
        ];
        run_magick(&[], &args).unwrap();

        let upscaled = upscale_to_target(&wide, tmp.path()).expect("upscaled");
        // A square canvas is what launchers expect; stretching a 4:1 banner to
        // fill it would distort the logo.
        assert_eq!(
            image::measure_file(&upscaled).map(image::Dimensions::side),
            Some(TARGET_SIDE)
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
