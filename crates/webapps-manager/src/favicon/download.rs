use anyhow::Result;
use std::io::Read;
use std::path::PathBuf;

/// Hard cap on icon byte size. Favicons in the wild rarely exceed 200 KB; the
/// 1 MB ceiling defends against decompression abuse while leaving headroom for
/// well-padded SVG/PNG sets some sites ship.
const MAX_ICON_BYTES: usize = 1024 * 1024;

/// Read a response body, refusing to buffer more than `max_bytes`.
///
/// `Response::bytes`/`text` read the whole body into memory first and only
/// then can a size be checked — a server that omits or lies about
/// `Content-Length` could stream gigabytes and OOM the process before any
/// post-hoc length check runs. Reading through a `take`-bounded reader caps
/// the allocation at the source. Reads one byte past the limit so an
/// over-size body is detected rather than silently truncated.
pub(super) fn read_body_capped(
    response: reqwest::blocking::Response,
    max_bytes: usize,
) -> Result<Vec<u8>> {
    if let Some(declared) = response.content_length() {
        if declared as usize > max_bytes {
            anyhow::bail!("Response too large: {declared} bytes");
        }
    }
    let mut body = Vec::new();
    response.take(max_bytes as u64 + 1).read_to_end(&mut body)?;
    if body.len() > max_bytes {
        anyhow::bail!("Response too large: exceeds {max_bytes} bytes");
    }
    Ok(body)
}

pub(super) fn download_icon(
    client: &reqwest::blocking::Client,
    url: &str,
    cache_dir: &std::path::Path,
    index: usize,
) -> Result<PathBuf> {
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(5))
        .send()?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }

    if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        if let Ok(content_type) = content_type.to_str() {
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
    }

    let bytes = read_body_capped(response, MAX_ICON_BYTES)?;
    if bytes.is_empty() {
        anyhow::bail!("Empty response");
    }

    let ext = guess_extension(url, &bytes);
    let path = cache_dir.join(format!("icon_{index}.{ext}"));

    if ext == "ico" {
        if let Ok(image) = image::load_from_memory(&bytes) {
            let png_path = cache_dir.join(format!("icon_{index}.png"));
            if image.save(&png_path).is_ok() {
                return Ok(png_path);
            }
        }
    }

    std::fs::write(&path, &bytes)?;
    Ok(path)
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
