# AGENT.md — BigLinux WebApps

Working notes for any AI coding agent operating on this repository.
Read this before touching the tree. Humans can read it too.

---

## What this project is

Desktop app that turns websites into integrated `.desktop` launchers. Three
binaries backed by a shared core:

| Binary               | Crate             | Role                                                |
| -------------------- | ----------------- | --------------------------------------------------- |
| `big-webapps-gui`    | `webapps-manager` | GTK4/libadwaita UI to CRUD webapps                  |
| `big-webapps-exec`   | `webapps-exec`    | Launcher shim invoked by generated `.desktop` files |
| `big-webapps-viewer` | `webapps-viewer`  | Embedded WebKitGTK window (app-mode webapps)        |
| —                    | `webapps-core`    | Shared models, TOML configs, i18n plumbing          |

The Python version in `/usr/share/biglinux/webapps/` is the legacy stable.
This Rust rewrite replaces it.

---

## Ground rules

1. **Language**
   - Code, identifiers, commits, internal comments → English.
   - User-facing strings (gettext) → Brazilian Portuguese is the primary
     translation; source strings stay in English.
   - Conversation with the maintainer in this repo → Brazilian Portuguese.

2. **File sizes**
   - Target 200–300 lines. Justify going over 500.
   - Function bodies 4–20 logical lines. Longer = split.

3. **Naming**
   - No `data`, `helper`, `util`, `manager`, `process`. Prefer specific,
     grep-friendly names (`resolve_icon_path`, `rebuild_sections`).

4. **Types**
   - Explicit types on all public Rust APIs.
   - Domain primitives over raw strings where possible (see
     `webapps-core::models::{CategoryList, ProfileKind, WebAppUrl, BrowserId}`).

5. **Comments — default to zero**
   - Do NOT write "what" comments when the code already says it.
   - Do NOT write "added for X", "used to be Y", or references to PRs, issues,
     commits, past rewrites, or planning docs. The git log is the history.
   - DO write a one-line comment when the *why* is non-obvious (hidden
     invariant, workaround, surprising ordering).

6. **Scope discipline**
   - Bug fix = fix the bug. No drive-by refactor.
   - No new abstractions until there is a third concrete caller.
   - No feature flags / back-compat shims unless the maintainer asks.
   - No `*.md` planning documents unless explicitly requested.

7. **Safety**
   - Tool side-effects: deleting files, force-pushing, modifying shared state
     → confirm with the maintainer first.
   - No `--no-verify`, no `--amend` on already-pushed commits, no hook skipping.

---

## Repository layout

```
crates/                  Rust workspace (no GTK in -core)
  webapps-core/          shared lib: models, browsers.toml loader, i18n, desktop writer
  webapps-manager/       GUI crate (binary big-webapps-gui, plus lib)
  webapps-exec/          launcher shim (binary big-webapps-exec)
  webapps-viewer/        WebKit window (binary big-webapps-viewer)

biglinux-webapps/        Data tree shipped by every packager
  usr/share/
    applications/        br.com.biglinux.webapps.desktop
    metainfo/            br.com.biglinux.webapps.metainfo.xml       (AppStream)
    icons/hicolor/       big-webapps.svg + big-webapps-symbolic.svg
    biglinux-webapps/    browsers.toml
    biglinux/webapps/    bundled browser icons + viewer profile seed
    desktop-directories/ big-webapps.directory, google-apps.directory
  usr/lib/systemd/user/  biglinux-webapps.service
  usr/bin/               biglinux-webapps-systemd (session init helper)
  etc/xdg/menus/         KDE/GNOME menu integration

packaging/               One subfolder per distribution channel
  arch/                  PKGBUILD + biglinux-webapps.install
  flatpak/               br.com.biglinux.webapps.yml + generate-cargo-sources.sh

flake.nix                Nix flake: package, devShell, apps.default
po/                      Translation catalogs + biglinux-webapps.pot
scripts/
  validate-customizations.sh   one-shot local gate (see below)
  update-translations.sh       regenerate .pot + msgmerge every .po
```

---

## Packaging channels

### Arch (`packaging/arch/PKGBUILD`)
- Auto-detects a local working tree via `BASH_SOURCE[0]`; falls back to
  `git+${url}.git` when building without one.
- Version = `$(date +%y.%m.%d)-$(date +%H%M)`, surfaced to the app via the
  `BIGLINUX_WEBAPPS_VERSION` env var at build time.
- Mirrors the source into `$srcdir/$pkgname` via rsync with `target/`,
  `.git/`, and `packaging/arch/{pkg,src}/` excluded — do not casually add
  new top-level dirs without extending the exclude list.

### Flatpak (`packaging/flatpak/br.com.biglinux.webapps.yml`)
- Runtime: `org.gnome.Platform//48` (current Flathub-recommended branch;
  each GNOME branch supported for ~1 year — bump in lockstep with upstream)
  plus `org.freedesktop.Sdk.Extension.rust-stable//24.08` for the Rust
  toolchain. The extension branch must match the Freedesktop base of the
  GNOME runtime (verify with `flatpak info -m org.gnome.Sdk//<ver>`).
- Local test: `flatpak-builder --user --install --force-clean build-dir
  packaging/flatpak/br.com.biglinux.webapps.yml`.
- Flathub submission checklist:
  1. Swap the `type: dir` source for `type: git` pinned to the release tag.
  2. Run `packaging/flatpak/generate-cargo-sources.sh` to produce
     `cargo-sources.json`, then add it to the manifest `sources:` list.
  3. Drop `--share=network` from `build-options.build-args`, set
     `CARGO_NET_OFFLINE: 'true'` in the build env, and change the build
     commands to `cargo --offline fetch` + `cargo build --offline --release --locked`.
- The app-id is `br.com.biglinux.webapps` and must stay that way — all
  `.desktop`, metainfo, icon, and D-Bus names key off it.

### Nix (`flake.nix`)
- `nix build` produces the full package with binaries, desktop entry,
  metainfo, icons, browsers.toml, and compiled locales.
- `nix run` launches `big-webapps-gui`.
- `nix develop` drops into a shell with cargo + all native GTK/WebKit deps.
- webkitgtk-6.0 is referenced as `pkgs.webkitgtk_6_0`; adjust if a newer
  nixpkgs pin renames the attribute.

---

## Validation gate

Before declaring any change complete, run:

```bash
./scripts/validate-customizations.sh
```

That wraps:

```
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
shellcheck biglinux-webapps/usr/bin/biglinux-webapps-systemd packaging/arch/PKGBUILD
shfmt -d   biglinux-webapps/usr/bin/biglinux-webapps-systemd packaging/arch/PKGBUILD
bash -n    biglinux-webapps/usr/bin/biglinux-webapps-systemd packaging/arch/PKGBUILD
```

Warnings are failures. If the script is missing a tool, install it; do not
comment out the check.

UI changes require a manual sanity run:

```bash
cargo run --release --bin big-webapps-gui
```

If you cannot launch a GUI from your environment, say so explicitly in the
summary instead of claiming UI success.

---

## Translations

1. Adding a new gettext string in code → run
   `./scripts/update-translations.sh` (needs `cargo install xtr`).
   That regenerates `po/biglinux-webapps.pot` and `msgmerge`s each
   `po/*.po`. Strings that match a close existing entry get flagged
   `#, fuzzy` — translators review those. Untranslated / fuzzy strings
   are **dropped** from the compiled `.mo`, so the UI falls back to
   English until a human updates them.
2. pt-BR is the primary translation. Keep it fully translated at all
   times. Other languages can lag behind.
3. The PKGBUILD, Flatpak manifest, and Nix flake all compile `.po → .mo`
   at build time and install to
   `/usr/share/locale/<lang>/LC_MESSAGES/biglinux-webapps.mo`. The `-`
   in `pt-BR` is normalized to `_` (POSIX) during install.

---

## Subsystems with gotchas

### Browser detection (`webapps-core::browsers` + `webapps-manager::service::browser`)
- Source of truth is `browsers.toml`, not code.
- Default-browser resolution uses `desktop_pattern` + `desktop_aliases` + the
  Flatpak `flatpak_app_id`. Longest match wins.
- When `xdg-settings` fails, `xdg-mime query default x-scheme-handler/http`
  is the fallback.
- `flatpak_id` is returned when the system default points to the Flatpak
  variant; the native `id` otherwise.

### Worker threads (`ui_async::run_with_result`)
- Any filesystem/network call that can block longer than ~50 ms runs on a
  worker. The result callback executes back on the main loop.
- Do not call GTK APIs from inside the worker closure. Only from the
  result callback.

### Icon resolution (`service::icons::resolve_icon_path`)
- Returns either an absolute path (file found) or a plain icon name
  (theme lookup). The caller chooses `set_from_file` vs `set_icon_name`
  based on that distinction.
- Walks a size fallback chain and progressive suffix stripping; keep both.

### CRUD atomicity (`service::crud`)
- `create_webapp` / `update_webapp` / `delete_webapp` / `delete_all_webapps`
  each hold an advisory file lock and have rollback paths. Integration
  tests in `crates/webapps-manager/tests/crud_integration.rs` guard those
  contracts — do not break them casually.

### Geometry persistence (`geometry`)
- Window and per-dialog sizes are persisted to JSON under the user cache
  dir. Binding helpers: `load_geometry`, `save_geometry`, `bind_adw_dialog`.

### Wayland icon swap (`webapps-exec::wayland`)
- Chromium-family browsers don't let us set WM_CLASS. Workaround: swap
  the `.desktop` Icon= entry, launch, then restore. `DEFAULT_SWAP_SETTLE_MS`
  is tuned; override via `BIG_WEBAPPS_SWAP_SETTLE_MS` if a compositor needs
  more time.

---

## Release / versioning

- Cargo version lives in `Cargo.toml` (`[workspace.package]`). Keep it in
  step with the AppStream `<releases>` entries in
  `biglinux-webapps/usr/share/metainfo/br.com.biglinux.webapps.metainfo.xml`.
- Arch PKGBUILD uses a date-based `pkgver`; bumping it is automatic.
- Flatpak and Nix read the Cargo version (or git tag). Tag releases as
  `v<major>.<minor>.<patch>`.
- `Cargo.lock` is committed. Do not regenerate it casually — commit
  intentional updates as their own change.

---

## Prompts to refuse

- "Rewrite everything in async" / large-scale refactors without a
  concrete failing case.
- "Skip the hook / bypass the lint" — investigate the failure instead.
- "Add a planning doc / architecture diagram" unless the maintainer
  explicitly asked.
- Any change that ships code behind a feature flag purely because an
  agent was unsure.

When blocked by ambiguity, ask. Do not guess the maintainer's intent
on wide-blast-radius operations.
