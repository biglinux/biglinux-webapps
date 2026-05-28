# Stage 1 — Cartography

Repo: `biglinux-webapps` (4 crates, 11.6k LOC src). Single-purpose: create + run GTK4/WebKit-backed "web apps" (PWA shortcuts) on BigLinux.

---

## 1. Crate graph

```
webapps-core ────────────┐
   (no intra deps)       │
                         ▼
              webapps-exec  (bin: big-webapps-exec)
              webapps-manager (lib + bin: big-webapps-gui)
              webapps-viewer (bin: big-webapps-viewer)
```

| crate | purpose | intra deps | pub items |
|---|---|---|---|
| `webapps-core` | shared models (WebApp, Browser, Templates), config paths, desktop-entry builder, i18n bootstrap | — | 114 |
| `webapps-exec` | launcher binary; resolves browser cmdline, wayland WM-class swap, flatpak grants | core | 6 |
| `webapps-manager` | GTK4 manager GUI (CRUD, dialogs, favicon, migrations) | core | 55 |
| `webapps-viewer` | WebKit viewer window for "app mode" runs | core | 1 |

Doc-test count: 0. Integration tests: `crates/webapps-manager/tests/crud_integration.rs`.

---

## 2. Hotspot heatmap (lines × git-churn-12m)

Repo imported recently → churn signal is shallow. Treat ranking as proxy for "where edits land".

| score | churn | LOC | path |
|------:|------:|----:|------|
| 1568 | 4 | 392 | `crates/webapps-manager/src/service/migration/mod.rs` |
| 1196 | 4 | 299 | `crates/webapps-core/src/templates/registry.rs` |
|  999 | 3 | 333 | `crates/webapps-manager/tests/crud_integration.rs` |
|  900 | 4 | 225 | `crates/webapps-manager/src/browser_dialog.rs` |
|  885 | 5 | 177 | `crates/webapps-manager/src/template_gallery.rs` |
|  852 | 6 | 142 | `crates/webapps-manager/src/webapp_row.rs` |
|  772 | 4 | 193 | `crates/webapps-core/src/desktop/mod.rs` |
|  645 | 3 | 215 | `crates/webapps-manager/src/service/crud/helpers.rs` |
|  638 | 2 | 319 | `crates/webapps-manager/src/favicon/html.rs` |
|  579 | 3 | 193 | `crates/webapps-manager/src/service/crud/operations.rs` |
|  543 | 3 | 181 | `crates/webapps-manager/src/window/mod.rs` |
|  489 | 3 | 163 | `crates/webapps-core/src/templates/office365.rs` |
|  480 | 2 | 240 | `crates/webapps-manager/src/webapp_dialog/handlers/fields.rs` |
|  465 | 3 | 155 | `crates/webapps-core/src/templates/google.rs` |
|  456 | 3 | 152 | `crates/webapps-core/src/desktop/paths.rs` |
|  434 | 2 | 217 | `crates/webapps-manager/src/favicon/mod.rs` |
|  408 | 4 | 102 | `crates/webapps-core/src/desktop/wm_class.rs` |
|  376 | 2 | 188 | `crates/webapps-exec/src/wayland.rs` |
|  374 | 2 | 187 | `crates/webapps-exec/src/launch.rs` |
|  357 | 3 | 119 | `crates/webapps-manager/src/welcome_dialog.rs` |
|  314 | 2 | 157 | `crates/webapps-manager/src/service/browser.rs` |
|  306 | 2 | 153 | `crates/webapps-manager/src/service/io.rs` |
|  300 | 2 | 150 | `crates/webapps-core/src/templates/productivity.rs` |
|  265 | 5 | 53 | `crates/webapps-core/src/config.rs` |
|  261 | 3 | 87 | `crates/webapps-core/src/desktop/builder.rs` |
|  254 | 2 | 127 | `crates/webapps-viewer/src/window/navigation/fullscreen.rs` |
|  234 | 1 | 234 | `crates/webapps-manager/src/window/actions/remove_multiple.rs` |
|  233 | 1 | 233 | `crates/webapps-manager/src/window/list.rs` |
|  224 | 2 | 112 | `crates/webapps-core/src/templates/media.rs` |
|  224 | 2 | 112 | `crates/webapps-core/src/models/browser.rs` |

---

## 3. Public API anchors (crate-boundary `pub` items)

### `webapps-core`
- `config::{config_dir,data_dir,cache_dir,applications_dir,system_icons_dir,profiles_dir}` — `src/config.rs:16-49`
- `browsers::{BrowserDef, browser_defs, find_def}` — `src/browsers.rs:13,64,69`
- `i18n::init` — `src/i18n.rs:7`
- `models::webapp::{WebApp, WebAppCollection, AppMode, AppCategory, CategoryList}` — `src/models/webapp/*`
- `models::browser::{BrowserKind, Browser, BrowserCollection}` — `src/models/browser.rs`
- `templates::{WebAppTemplate, FileHandler, google::templates, productivity::templates, office365::templates, media::templates, registry…}` — `src/templates/**`
- `desktop::builder::generate_desktop_entry` — `src/desktop/builder.rs:7`
- `desktop::icon::{persist_icon, webapp_icons_dir}` — `src/desktop/icon.rs:33,74`

### `webapps-exec` (bin: `big-webapps-exec`)
- `Args` struct (clap) — `src/main.rs:68`
- `launch::{firefox, chromium, grant_flatpak_access}` — `src/launch.rs:21,57,78`
- `wayland::swap_and_launch` — `src/wayland.rs:39`
- `icon::normalize` — `src/icon.rs:14`

### `webapps-manager` (lib + bin `big-webapps-gui`)
- `geometry::{geometry_path, load_geometry, save_geometry, bind_dialog, bind_adw_dialog}` — `src/geometry.rs`
- `browser_dialog::{BrowserSelection, show}` — `src/browser_dialog.rs:11,27`
- `favicon::{SiteInfo, fetch_site_info}` — `src/favicon/mod.rs:9,14`
- `service::{repository::{load_webapps,save_webapps,mutate_webapps}, io::{export_webapps,import_webapps}, browser::detect_browsers, icons::resolve_icon_path, browser_url::resolve_browser_url, migration::{migrate_legacy_desktops, regenerate_app_mode_desktops, regenerate_browser_mode_desktops, migrate_browser_desktop_filenames, persist_existing_icons}}`
- `webapp_dialog::{DialogResult, show}` — `src/webapp_dialog/mod.rs:18,22`
- `style::load_css` — `src/style.rs:58`

### `webapps-viewer` (bin: `big-webapps-viewer`)
- `window::build` — `src/window/mod.rs:31`  (only public anchor; rest is `pub(crate)`)

---

## 4. Entry points

- **bins**: `big-webapps-exec` (`webapps-exec/src/main.rs`), `big-webapps-gui` (`webapps-manager/src/main.rs`), `big-webapps-viewer` (`webapps-viewer/src/main.rs`)
- **libs**: `webapps-core::lib`, `webapps-manager::lib`
- **integration tests**: `webapps-manager/tests/crud_integration.rs`
- **build.rs**: none
- **examples**: none

---

## 5. External boundaries

### Subprocess (argv-sensitive)
| spawn site | argv | notes |
|---|---|---|
| `webapps-core/src/desktop/paths.rs:122` | `update-desktop-database <applications_dir>` | post-write hook |
| `webapps-core/src/desktop/paths.rs:145` | `dconf <args>` (XDG KDE-only branch) | reads `XDG_CURRENT_DESKTOP` |
| `webapps-exec/src/launch.rs:80` | `flatpak override --user --filesystem=… <app-id>` | runs Stage-5 critical surface |
| `webapps-manager/src/service/browser.rs:26` | `flatpak list --app --columns=…` | enumeration |
| `webapps-manager/src/service/browser.rs:62` | `xdg-settings get default-web-browser` | read |
| `webapps-manager/src/service/browser.rs:71` | `xdg-mime …` | mime register |

### Env reads
| file:line | var | sink |
|---|---|---|
| `webapps-core/src/desktop/paths.rs:131` | `XDG_CURRENT_DESKTOP` | branch only |
| `webapps-exec/src/icon.rs:15` | `HOME` | path join |
| `webapps-exec/src/launch.rs:66` | `XDG_SESSION_TYPE` | branch only |
| `webapps-exec/src/wayland.rs:28` | `BIGLINUX_WEBAPPS_SWAP_SETTLE_MS` | parse u64 |
| `webapps-exec/src/wayland.rs:40` | `HOME` | path join |
| `webapps-manager/src/service/browser_url.rs:28` | `LANG` | Accept-Language header |

### Filesystem writes (config, cache, icons, geometry, profiles)
~74 sites under `data_dir`/`cache_dir`/`config_dir`/`profiles_dir`/`applications_dir`. All routed through `webapps-core::config::*` paths. Direct writes: `geometry.rs:96,160`, `repository.rs:87,115` (atomic via `tmp + rename`), `welcome.rs:13`, `favicon/download.rs:67`, `desktop/paths.rs:94`, `desktop/icon.rs:*`.

### Filesystem reads (untrusted-content surface)
- Web favicons / manifests fetched over HTTP and parsed (`favicon/{mod,html,download}.rs`).
- Imported zip archive of webapps (`service/io.rs:55 import_webapps`).
- Legacy desktop files for migration (`service/migration/mod.rs`).
- Permissions JSON written by viewer (`viewer/window/permissions/mod.rs:68`).

### Network
- `reqwest::blocking::Client` for favicon + manifest fetches (`favicon/{mod,download,html}.rs`).
- `webkit6::WebView::load_uri` (`viewer/window/startup.rs:31`, `navigation/url_entry.rs:20`).
- HTTP client uses redirect limit 10 (`favicon/mod.rs:43`).

### IPC / D-Bus / Wayland
- No direct dbus crate. KDE branch shells `dconf` only.
- Wayland surface manipulation via WM_CLASS swap in `webapps-exec/src/wayland.rs` (X11 fallback also).
- WebKit native messaging / GIO portal mediation implicit through `webkit6`.

### FFI
- All through `gtk4`, `libadwaita`, `webkit6`, `gdk4`, `gdk-pixbuf`, `gio`, `cairo`, `pango`, `glib`, `soup3`. No hand-written `bindgen`.

---

## Inputs to later stages

- Hotspot top-5 + every file > 300 LOC → Stage 11 (split candidates).
- Subprocess + reqwest sites → Stage 5 (STRIDE).
- `Command::spawn`, `webkit::WebView`, `reqwest::Client`, `glib signal connect`, `gio Cancellable` → Stage 6 (lifecycle ledger).
- `webkit6::WebView` cold-start dominates startup RSS → Stage 7 anchor.
- `webapps-manager` widget files → Stage 8/9 (i18n + a11y).
