use std::{collections::HashSet, path::PathBuf};
use webapps_core::{browsers::BrowserDef, config, models::Browser, subprocess::SubprocessSpec};

pub(super) fn detect(definitions: &[BrowserDef]) -> Vec<Browser> {
    let listed = listed_app_ids();
    let roots = installation_roots();
    definitions
        .iter()
        .filter_map(|definition| {
            let app_id = definition.flatpak_app_id.as_deref()?;
            let browser_id = definition.flatpak_id.as_ref()?;
            (listed.contains(app_id) || has_installation(app_id, &roots)).then(|| Browser {
                browser_id: browser_id.clone(),
                is_default: false,
            })
        })
        .collect()
}

fn listed_app_ids() -> HashSet<String> {
    let result = SubprocessSpec::builder()
        .program("flatpak")
        .on_host()
        .args(["list", "--app", "--columns=application"])
        .build()
        .run();
    match result {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect(),
        Ok(output) => {
            log::warn!(
                "Flatpak detection failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
            HashSet::new()
        }
        Err(error) => {
            log::debug!("Flatpak detection unavailable: {error}");
            HashSet::new()
        }
    }
}

fn installation_roots() -> Vec<PathBuf> {
    let user = std::env::var_os("FLATPAK_USER_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if config::is_flatpak() && std::env::var_os("HOST_XDG_DATA_HOME").is_none() {
                if let Some(home) = dirs::home_dir() {
                    return home.join(".local/share/flatpak");
                }
            }
            config::host_data_dir().join("flatpak")
        });
    let system = std::env::var_os("FLATPAK_SYSTEM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/var/lib/flatpak"));
    vec![user, system]
}

fn has_installation(app_id: &str, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| {
        [
            root.join("app").join(app_id),
            root.join("exports/bin").join(app_id),
        ]
        .iter()
        .any(|path| {
            if config::is_flatpak() {
                config::host_command("test")
                    .arg("-e")
                    .arg(path)
                    .status()
                    .is_ok_and(|status| status.success())
            } else {
                path.exists()
            }
        })
    })
}
