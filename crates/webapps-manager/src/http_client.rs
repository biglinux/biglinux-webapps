use std::io::Read;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, USER_AGENT};
use reqwest::redirect::Policy;

#[derive(Debug, Clone)]
pub struct RequestHeaders {
    user_agent: String,
    accept: String,
}

#[derive(Debug, Clone)]
pub struct ByteResponse {
    pub status: u16,
    pub final_url: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StreamResponse {
    pub status: u16,
    pub final_url: String,
}

impl RequestHeaders {
    pub fn browser() -> Self {
        Self {
            user_agent:
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120 Safari/537.36"
                    .to_string(),
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/*,*/*;q=0.8"
                .to_string(),
        }
    }
}

pub fn http_get_bytes_capped(
    url: &str,
    headers: &RequestHeaders,
    max_bytes: u64,
    timeout: Duration,
) -> Result<ByteResponse> {
    let response = client(timeout)
        .get(url)
        .headers(header_map(headers, &[])?)
        .send()
        .with_context(|| format!("GET {url}"))?;
    read_capped(response, max_bytes)
}

pub fn http_get_stream_with_extra_headers(
    url: &str,
    headers: &RequestHeaders,
    extra_headers: &[(&str, &str)],
    timeout: Duration,
) -> Result<StreamResponse> {
    let response = client(timeout)
        .get(url)
        .headers(header_map(headers, extra_headers)?)
        .send()
        .with_context(|| format!("GET {url}"))?;
    Ok(StreamResponse {
        status: response.status().as_u16(),
        final_url: response.url().to_string(),
    })
}

fn client(timeout: Duration) -> Client {
    Client::builder()
        .timeout(timeout)
        .redirect(Policy::limited(10))
        .build()
        .expect("reqwest client builder accepts static configuration")
}

fn header_map(headers: &RequestHeaders, extra_headers: &[(&str, &str)]) -> Result<HeaderMap> {
    let mut map = HeaderMap::new();
    map.insert(USER_AGENT, HeaderValue::from_str(&headers.user_agent)?);
    map.insert(ACCEPT, HeaderValue::from_str(&headers.accept)?);
    for (name, value) in extra_headers {
        map.insert(
            HeaderName::from_bytes(name.as_bytes())?,
            HeaderValue::from_str(value)?,
        );
    }
    Ok(map)
}

fn read_capped(mut response: Response, max_bytes: u64) -> Result<ByteResponse> {
    let status = response.status().as_u16();
    let final_url = response.url().to_string();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let mut bytes = Vec::new();
    response
        .by_ref()
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .context("read response body")?;
    if bytes.len() as u64 > max_bytes {
        anyhow::bail!("response exceeds {max_bytes} bytes");
    }
    Ok(ByteResponse {
        status,
        final_url,
        content_type,
        bytes,
    })
}
