use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::http_client::{http_get_bytes_capped, RequestHeaders};

/// Hard cap on icon byte size. Favicons in the wild rarely exceed 200 KB; the
/// 1 MB ceiling defends against decompression abuse while leaving headroom for
/// well-padded SVG/PNG sets some sites ship.
const MAX_ICON_BYTES: usize = 1024 * 1024;

pub(super) fn download_icon(
    url: &str,
    cache_dir: &std::path::Path,
    index: usize,
) -> Result<PathBuf> {
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
        let content_type = content_type.to_lowercase();
        // Strict allowlist: `image/*` covers png/jpeg/webp/svg/x-icon, plus
        // explicit `application/octet-stream` (some CDNs serve favicons that
        // way). Reject anything else — accepting "could be image" responses
        // expanded the parser attack surface unnecessarily.
        let acceptable = content_type.starts_with("image/")
            || content_type == "application/octet-stream"
            || content_type.starts_with("application/octet-stream;")
            || content_type.starts_with("application/vnd.microsoft.icon");
        if !acceptable {
            anyhow::bail!("Not an image: {content_type}");
        }
    }

    let bytes = response.bytes;
    if bytes.is_empty() {
        anyhow::bail!("Empty response");
    }

    let ext = guess_extension(url, &bytes);
    let path = cache_dir.join(format!("icon_{index}.{ext}"));

    if ext == "ico" {
        let png_path = cache_dir.join(format!("icon_{index}.png"));
        if convert_icon_to_png_with_magick(&bytes, &png_path).is_ok() {
            return Ok(png_path);
        }
    }

    std::fs::write(&path, &bytes)?;
    Ok(path)
}

fn convert_icon_to_png_with_magick(bytes: &[u8], png_path: &std::path::Path) -> Result<()> {
    let mut command = Command::new("magick");
    command.arg("-");
    command.arg(format!("PNG:{}", png_path.display()));
    command.stdin(Stdio::piped());
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());
    let mut child = command.spawn().context("spawn ImageMagick magick")?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("failed to open magick stdin"))?;
        stdin.write_all(bytes)?;
    }
    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("magick icon conversion failed: {stderr}")
    }
}

fn guess_extension(url: &str, bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\x89PNG") {
        return "png";
    }
    if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
        return "svg";
    }
    if bytes.starts_with(&[0, 0, 1, 0]) || bytes.starts_with(&[0, 0, 2, 0]) {
        return "ico";
    }
    if url.contains(".svg") {
        return "svg";
    }
    if url.contains(".ico") {
        return "ico";
    }
    "png"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guess_extension_png_magic() {
        assert_eq!(guess_extension("https://x.com/img", b"\x89PNG\r\n"), "png");
    }

    #[test]
    fn guess_extension_svg_magic() {
        assert_eq!(guess_extension("https://x.com/img", b"<svg "), "svg");
    }

    #[test]
    fn guess_extension_ico_magic() {
        assert_eq!(guess_extension("https://x.com/img", &[0, 0, 1, 0]), "ico");
    }

    #[test]
    fn guess_extension_url_fallback() {
        assert_eq!(guess_extension("https://x.com/icon.svg", b"unknown"), "svg");
        assert_eq!(guess_extension("https://x.com/icon.ico", b"unknown"), "ico");
    }

    #[test]
    fn guess_extension_default_png() {
        assert_eq!(guess_extension("https://x.com/img", b"unknown"), "png");
    }
}
