# Architecture — biglinux-webapps

## 1. Purpose

`biglinux-webapps` ships a GTK4/libadwaita stack for creating, managing, and
launching site-specific browser instances ("WebApps") on BigLinux. It generates
isolated browser profiles per webapp, persists a JSON catalog under XDG dirs,
emits `.desktop` files, and ships its own WebKit viewer for sites that should
not pull in a full browser. The workspace is a four-crate split between a
toolkit-free core, two GUI binaries, and a small `exec` launcher.

## 2. Crate Layout

- `crates/webapps-core/` — toolkit-free domain: `models/` (webapp, browser,
  permissions), `templates/registry.rs` (curated catalog), `browsers.rs`
  (whitelist + flatpak detection), `desktop/{builder,paths,sanitize,wm_class,
  icon}.rs`, `config.rs`, `i18n.rs`. No GTK. Used by every other crate.
- `crates/webapps-exec/` — binary `big-webapps-exec`: minimal launcher invoked
  from generated `.desktop` files. Modules: `launch.rs` (Firefox via `exec()`,
  Chromium via `spawn()`), `wayland.rs` (icon-swap workaround), `icon.rs`.
- `crates/webapps-manager/` — binary `big-webapps-gui` + lib: Relm4 + classic
  GTK manager UI. `service/` (CRUD, repository with flock, migration,
  icons, browser detect, welcome), `relm4_window/` (typed list +
  factory rows), `window/` (legacy imperative shell), `webapp_dialog/`,
  `template_gallery.rs`, `welcome_dialog.rs`, `favicon/`, `ui_async.rs`.
- `crates/webapps-viewer/` — binary `big-webapps-viewer`: WebKit6 viewer.
  `window/{chrome,session,settings,navigation,permissions,downloads,
  shortcuts,geometry,context_menu,loading,startup}.rs`.

Dependency direction: `core` ← `exec`, `manager`, `viewer`. No cycles.
`manager` and `viewer` never link to each other.

## 3. State Machine + Message Flow

Webapp lifecycle (manager):

```
Welcome? --no--> List(load) --create-->  Dialog(new)   --validate+save--> List
                  |   ^                  Dialog(edit)  --validate+save--> List
                  |   |                  Dialog(delete)--confirm-------->  List
                  v   |
              Empty<--+
```

- `service::repository::load_webapps` holds an `flock(LOCK_EX)` `WebappsLock`
  for every write transaction; `Drop` unlocks even on panic (INVARIANTS).
- Relm4 surface: `WebAppListController` (FactoryVecDeque) owns
  `WebAppRowFactory` items. Inputs: `Reload`, `Add(Webapp)`, `Remove(id)`,
  `Update(Webapp)`. Outputs forwarded to the main window component.
- Imperative `window/` shell is being migrated to `relm4_window/`; both
  coexist behind the same `service` facade — single source of truth.
- Viewer state per process: window geometry (`geometry.rs` persists), session
  cookies (per-`app_id` WebKit data manager), permissions (atomic
  `tmp + rename` write in `window/permissions/`).

## 4. Async Boundaries

- Manager `ui_async.rs` wraps `glib::MainContext::spawn_local` for favicon
  fetches, browser detection, and icon resolution.
- `favicon/` performs capped blocking HTTP through `big_os_kit::http_client`
  inside `glib` IO threads to avoid blocking the GTK main loop.
- All Relm4 components run on the main context; no background tokio runtime.
- Viewer downloads/permissions hook into WebKit asynchronous callbacks; user
  consent is awaited via `adw::AlertDialog` futures.
- `glib::SourceId` debounces (dialog text entries) are `.remove()`d before
  replacement (INVARIANTS — lifecycle).

## 5. External Processes (`Command::new`)

| Site | Argv invariant |
|------|----------------|
| `webapps-exec/launch.rs:37` `Command::new(<browser>)` then `.exec()` | Firefox-family. Replaces process image. Browser binary resolved from `webapps-core::browsers` whitelist; profile path argv-quoted, never shell-expanded. Env: `XAPP_FORCE_GTKWINDOW_ICON`, `MOZ_APP_REMOTINGNAME`. |
| `webapps-exec/launch.rs:61` `Command::new(<browser>).spawn()` | Chromium-family. Same whitelist gate. `--class=` / `--user-data-dir=` constructed from sanitized `app_id`. |
| `webapps-exec/launch.rs:80` `Command::new("flatpak").args(["override","--user","--filesystem=…",app_id])` | Flatpak grant only. `--filesystem` value is the runtime profile dir; never user-controlled. |
| `webapps-core/desktop/paths.rs:122` `Command::new("update-desktop-database")` | Refresh `xdg` DB after install/remove. No user argv. |
| `webapps-core/desktop/paths.rs:145` `Command::new("dconf").args(*args)` | `args` is a `&'static [&str]` literal (`write` / `read`). No user input on argv. |
| `webapps-manager/service/browser.rs:26` `Command::new("flatpak").args(["list","--app","--columns=application"])` | Static argv. Stdout parsed for app IDs. |
| `webapps-manager/service/browser.rs:62` `Command::new("xdg-settings").args(["get","default-web-browser"])` | Static argv. |
| `webapps-manager/service/browser.rs:71` `Command::new("xdg-mime").args(…)` | Static argv only; no user strings. |

Invariant: every browser path passed to `Command::new` must resolve through
`webapps-core::browsers::BrowserDef` (whitelist). Direct user-supplied
executables are rejected at validation time (INVARIANTS — security).

## 6. Security Posture

- Browser whitelist: `webapps-exec` refuses any browser not declared in
  `webapps-core::browsers`. No shell, no `system()`.
- Desktop file safety: `desktop::sanitize::sanitize_exec_arg` quotes every
  field interpolated into `Exec=` (6 unit cases in `sanitize.rs`).
- Persisted data: catalog JSON write is `tmp + rename`; permissions store
  follows the same pattern (`window/permissions/mod.rs::save_permissions`).
- Inter-process locking: `WebappsLock` (`flock(LOCK_EX)`) protects every
  write transaction; unlocked on `Drop` even under panic.
- Profile isolation: each WebApp gets its own browser profile dir under
  `~/.local/share/biglinux-webapps/profiles/<browser>/<slug>/`.
- Flatpak: only `flatpak override --user --filesystem=` issued; no
  `--talk-name`, no socket grants beyond the profile data dir.
- Viewer permissions: camera/mic/geolocation prompts route through
  `adw::AlertDialog`; default response is non-destructive (INVARIANTS — a11y).

## 7. Testing Strategy

- Unit: `webapps-core` for sanitize, paths, browsers, templates.
- Integration: `crates/webapps-manager/tests/crud_integration.rs` drives the
  CRUD pipeline against a sandboxed `$XDG_DATA_HOME` (uses `tempfile` +
  `serial_test`).
- Property: planned for desktop-file builder; today covered by sanitize
  cases.
- i18n: `po/biglinux-webapps.pot` round-trip enforced in `rust-quality.yml`.
- a11y: AT-SPI walk deferred to VM job; static review in `INVARIANTS.md`.
- Performance budget: each binary `--help` < 200 ms (manual, tracked).

## 8. Packaging

`packaging/arch/PKGBUILD`: declares `pkgname=biglinux-webapps`, depends on
`gtk4`, `libadwaita`, `webkitgtk-6.0`, `xdg-utils`, `desktop-file-utils`;
makedepends on `rust`, `cargo`, `gettext`. Local-source detection prefers
the workspace checkout (`BIGLINUX_WEBAPPS_LOCAL_SOURCE` env, or `Cargo.toml +
crates/ + biglinux-webapps/` next to the PKGBUILD) over the git fallback.
Installs three binaries (`big-webapps-exec`, `big-webapps-gui`,
`big-webapps-viewer`), the `biglinux-webapps/` resource tree
(profile CSS/JS, icons, desktop templates), and compiled `.mo` catalogs.
Flatpak manifest under `packaging/flatpak/` mirrors the Arch package.
