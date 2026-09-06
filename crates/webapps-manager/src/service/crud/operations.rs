use super::super::{browser_url::resolve_browser_url, transaction::RegistryTransaction};
use super::{
    cleanup::{cleanup_deleted_app, cleanup_unused_icon},
    validation::validate_webapp,
};
use anyhow::{bail, Context, Result};
use webapps_core::{
    desktop,
    models::{AppMode, WebApp},
};

pub fn create_webapp(webapp: &WebApp) -> Result<()> {
    let mut app = prepare_webapp(webapp, true)?;
    let mut transaction = RegistryTransaction::begin()?;
    ensure_available(&transaction, &app.app_file, None)?;
    install(&mut transaction, &mut app)?;
    transaction.collection.add(app);
    transaction.commit()?;
    desktop::refresh_desktop_database();
    Ok(())
}

pub fn update_webapp(webapp: &WebApp) -> Result<()> {
    let mut app = prepare_webapp(webapp, false)?;
    let mut transaction = RegistryTransaction::begin()?;
    let previous = transaction
        .collection
        .webapps
        .iter()
        .find(|candidate| candidate.app_file == webapp.app_file)
        .cloned()
        .context("Webapp no longer exists; reload the list")?;
    ensure_available(&transaction, &app.app_file, Some(&previous.app_file))?;
    install(&mut transaction, &mut app)?;
    if app.app_file != previous.app_file {
        transaction.remove(&desktop::desktop_file_path(&previous))?;
    }
    transaction.collection.remove_by_file(&previous.app_file);
    transaction.collection.add(app);
    transaction.commit()?;
    cleanup_unused_icon(&previous.app_icon, &transaction.collection);
    desktop::refresh_desktop_database();
    Ok(())
}

pub fn delete_webapp(webapp: &WebApp, delete_profile: bool) -> Result<()> {
    let mut transaction = RegistryTransaction::begin()?;
    let app = transaction
        .collection
        .webapps
        .iter()
        .find(|candidate| candidate.app_file == webapp.app_file)
        .cloned()
        .context("Webapp no longer exists; reload the list")?;
    validate_webapp(&app)?;
    transaction.remove(&desktop::desktop_file_path(&app))?;
    transaction.collection.remove_by_file(&app.app_file);
    transaction.commit()?;
    cleanup_after_commit(&app, delete_profile, &transaction);
    desktop::refresh_desktop_database();
    Ok(())
}

pub fn delete_all_webapps() -> Result<()> {
    let mut transaction = RegistryTransaction::begin()?;
    let apps = transaction.collection.webapps.clone();
    for app in &apps {
        validate_webapp(app)?;
        transaction.remove(&desktop::desktop_file_path(app))?;
    }
    transaction.collection.webapps.clear();
    transaction.commit()?;
    for app in &apps {
        cleanup_after_commit(app, false, &transaction);
    }
    desktop::refresh_desktop_database();
    Ok(())
}

fn prepare_webapp(webapp: &WebApp, is_new: bool) -> Result<WebApp> {
    validate_webapp(webapp)?;
    let mut app = webapp.clone();
    app.app_url = app.normalized_url()?.into_string();
    if app.app_mode == AppMode::Browser {
        let unchanged_launch = !is_new
            && crate::service::try_load_webapps()?
                .webapps
                .iter()
                .any(|previous| {
                    previous.app_file == app.app_file
                        && previous.app_url == app.app_url
                        && previous.browser == app.browser
                        && previous.app_profile == app.app_profile
                });
        if unchanged_launch {
            return Ok(app);
        }
        app.app_url = resolve_browser_url(&app.app_url);
        app.app_file = desktop::canonical_browser_desktop_filename(&app);
    } else if is_new || app.app_file.is_empty() {
        app.app_file = desktop::viewer_desktop_filename(&app.app_url);
    }
    validate_webapp(&app)?;
    Ok(app)
}

fn ensure_available(
    transaction: &RegistryTransaction,
    filename: &str,
    previous: Option<&str>,
) -> Result<()> {
    if previous == Some(filename) {
        return Ok(());
    }
    if transaction
        .collection
        .webapps
        .iter()
        .any(|app| app.app_file == filename)
        || webapps_core::config::applications_dir()
            .join(filename)
            .exists()
    {
        bail!("A webapp already uses this launcher; choose a different browser or profile");
    }
    Ok(())
}

pub(super) fn install(transaction: &mut RegistryTransaction, app: &mut WebApp) -> Result<()> {
    if let Some(target) = desktop::icon_destination(app) {
        let bytes = std::fs::read(&app.app_icon).context("Read selected icon")?;
        transaction.write(&target, &bytes)?;
        app.app_icon = target.to_string_lossy().into_owned();
    }
    app.app_icon_url = app.app_icon.clone();
    transaction.write(
        &desktop::desktop_file_path(app),
        desktop::generate_desktop_entry(app).as_bytes(),
    )
}

fn cleanup_after_commit(app: &WebApp, delete_profile: bool, transaction: &RegistryTransaction) {
    if let Err(err) = cleanup_deleted_app(app, delete_profile, &transaction.collection) {
        log::warn!("Webapp removed; profile cleanup failed: {err:#}");
    }
}
