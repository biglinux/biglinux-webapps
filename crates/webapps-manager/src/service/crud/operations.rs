use anyhow::{Context, Result};

use webapps_core::desktop;
use webapps_core::models::{AppMode, WebApp};

use super::super::browser_url::resolve_browser_url;
use super::super::repository::{load_webapps, mutate_webapps};
use super::helpers::{cleanup_deleted_app, profile_dir_for, validate_webapp};

pub fn create_webapp(webapp: &WebApp) -> Result<()> {
    let mut app = webapp.clone();
    // Browser-mode entries must use the filename Chromium will synthesize as
    // its Wayland app_id (e.g. `brave-open.spotify.com__intl-pt_-Default.desktop`).
    // GNOME Shell looks the running window up by exact app_id match against the
    // .desktop basename — anything else falls back to the host browser's icon.
    // Resolve redirects with the system locale first: Chromium derives the
    // app_id from the *post-redirect* URL (Spotify → /intl-pt/ on pt-BR), so we
    // must store the same URL Brave will end up showing.
    if app.app_mode == AppMode::Browser {
        app.app_url = resolve_browser_url(&app.app_url);
        app.app_file = desktop::canonical_browser_desktop_filename(&app);
    } else {
        app.app_file = desktop::viewer_desktop_filename(&app.app_url);
    }

    validate_webapp(&app)?;
    if let Some(stable_icon) = desktop::persist_icon(&app) {
        app.app_icon = stable_icon;
    }
    desktop::install_desktop_entry(&app)?;

    let app_for_rollback = app.clone();
    if let Err(err) = mutate_webapps(move |collection| {
        collection.add(app);
        Ok(())
    }) {
        if let Err(cleanup_err) = desktop::remove_desktop_entry(&app_for_rollback) {
            log::error!("Rollback failed after create_webapp persistence error: {cleanup_err}");
        }
        return Err(err).context("Persist webapps after creating desktop entry");
    }

    Ok(())
}

pub fn update_webapp(webapp: &WebApp) -> Result<()> {
    validate_webapp(webapp)?;
    let mut app = webapp.clone();
    if let Some(stable_icon) = desktop::persist_icon(&app) {
        app.app_icon = stable_icon;
    }

    let previous_file = webapp.app_file.clone();
    if app.app_mode == AppMode::Browser {
        app.app_url = resolve_browser_url(&app.app_url);
        let canonical = desktop::canonical_browser_desktop_filename(&app);
        if canonical != app.app_file {
            app.app_file = canonical;
        }
    } else {
        let canonical = desktop::viewer_desktop_filename(&app.app_url);
        if canonical != app.app_file {
            app.app_file = canonical;
        }
    }

    let webapp_clone = app.clone();
    desktop::install_desktop_entry(&app)?;

    // Capture previous so rollback can restore the desktop entry exactly.
    let previous_holder: std::sync::Mutex<Option<WebApp>> = std::sync::Mutex::new(None);
    let previous_holder = std::sync::Arc::new(previous_holder);
    let holder_for_mutate = previous_holder.clone();
    let previous_file_for_mutate = previous_file.clone();

    let result = mutate_webapps(move |collection| {
        let previous = collection
            .webapps
            .iter()
            .find(|app| app.app_file == previous_file_for_mutate)
            .cloned();
        if let Ok(mut slot) = holder_for_mutate.lock() {
            *slot = previous;
        }
        collection.remove_by_file(&previous_file_for_mutate);
        collection.add(webapp_clone);
        Ok(())
    });

    if let Err(err) = result {
        if app.app_file != previous_file {
            if let Err(cleanup_err) = desktop::remove_desktop_entry(&app) {
                log::error!(
                    "Rollback failed to remove renamed desktop entry {}: {cleanup_err}",
                    app.app_file
                );
            }
        }
        let previous = previous_holder.lock().ok().and_then(|g| g.clone());
        match previous {
            Some(app) => {
                if let Err(restore_err) = desktop::install_desktop_entry(&app) {
                    log::error!(
                        "Rollback failed after update_webapp persistence error: {restore_err}"
                    );
                }
            }
            None => {
                if let Err(cleanup_err) = desktop::remove_desktop_entry(webapp) {
                    log::error!(
                        "Rollback failed after update_webapp persistence error: {cleanup_err}"
                    );
                }
            }
        }
        return Err(err).context("Persist webapps after updating desktop entry");
    }

    if !previous_file.is_empty() && app.app_file != previous_file {
        if let Err(err) = desktop::remove_desktop_file(&previous_file) {
            log::warn!("Remove stale desktop entry {previous_file} during rename: {err}");
        }
    }

    Ok(())
}

pub fn delete_webapp(webapp: &WebApp, delete_profile: bool) -> Result<()> {
    validate_webapp(webapp)?;
    if delete_profile {
        profile_dir_for(webapp)?;
    }

    desktop::remove_desktop_entry(webapp)?;
    let app_file = webapp.app_file.clone();
    if let Err(err) = mutate_webapps(move |collection| {
        collection.remove_by_file(&app_file);
        Ok(())
    }) {
        if let Err(restore_err) = desktop::install_desktop_entry(webapp) {
            log::error!("Rollback failed after delete_webapp persistence error: {restore_err}");
        }
        return Err(err).context("Persist webapps after removing desktop entry");
    }

    cleanup_deleted_app(webapp, delete_profile)?;

    Ok(())
}

pub fn delete_all_webapps() -> Result<()> {
    let snapshot = load_webapps();
    let mut removed_entries = Vec::with_capacity(snapshot.webapps.len());

    for app in &snapshot.webapps {
        if let Err(err) = desktop::remove_desktop_entry(app) {
            for removed_app in &removed_entries {
                if let Err(restore_err) = desktop::install_desktop_entry(removed_app) {
                    log::error!("Rollback failed after delete_all_webapps error: {restore_err}");
                }
            }

            return Err(err).context("Remove desktop entry during delete_all_webapps");
        }

        removed_entries.push(app.clone());
    }

    let removed_for_rollback = removed_entries.clone();
    if let Err(err) = mutate_webapps(move |collection| {
        collection.webapps.clear();
        Ok(())
    }) {
        for app in &removed_for_rollback {
            if let Err(restore_err) = desktop::install_desktop_entry(app) {
                log::error!(
                    "Rollback failed after delete_all_webapps persistence error: {restore_err}"
                );
            }
        }

        return Err(err).context("Persist empty webapps collection after delete_all_webapps");
    }

    for app in &snapshot.webapps {
        if let Err(err) = cleanup_deleted_app(app, false) {
            log::warn!(
                "Cleanup after delete_all_webapps failed for {}: {err}",
                app.app_name
            );
        }
    }

    Ok(())
}
