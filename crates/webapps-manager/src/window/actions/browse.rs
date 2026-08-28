use crate::platform::desktop;

use webapps_core::config;

use super::super::context::WindowContext;
use crate::ui_async;

pub(super) fn install(context: &WindowContext) {
    desktop::install_action(&*context.window, "browse-apps", || {
        // `open::that` blocks on the spawned `xdg-open` exit status, so push it
        // to a worker thread; nothing in the UI depends on the result.
        let path = config::applications_dir();
        ui_async::run_with_result(move || open::that(path), report_open_error);
    });

    desktop::install_action(&*context.window, "browse-profiles", || {
        // Create the directory then open it on the worker thread — the ordering
        // (mkdir before open) is preserved inside the single job, and neither
        // step blocks the main loop.
        let path = config::profiles_dir();
        ui_async::run_with_result(
            move || {
                let _ = std::fs::create_dir_all(&path);
                open::that(path)
            },
            report_open_error,
        );
    });
}

fn report_open_error(result: std::io::Result<()>) {
    if let Err(err) = result {
        log::warn!("Open folder in file manager: {err}");
    }
}
