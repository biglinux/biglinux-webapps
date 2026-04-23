use crate::{favicon, ui_async};

/// Spawn favicon scrape; the callback is *guaranteed* to fire so callers can
/// reliably hide loading spinners even if the worker thread dies.
pub(super) fn detect_site_info(url: String, on_result: impl FnOnce(favicon::SiteInfo) + 'static) {
    ui_async::run_with_result_or_default(
        move || match favicon::fetch_site_info(&url) {
            Ok(info) => info,
            Err(err) => {
                log::error!("Fetch site info: {err}");
                favicon::SiteInfo {
                    title: String::new(),
                    icon_paths: Vec::new(),
                }
            }
        },
        move |maybe_info| {
            on_result(maybe_info.unwrap_or(favicon::SiteInfo {
                title: String::new(),
                icon_paths: Vec::new(),
            }));
        },
    );
}
