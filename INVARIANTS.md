# INVARIANTS — biglinux-webapps

Contracts the CI gates protect. Each line is enforced by a check listed in the right column. Change an invariant only with a matching change to the gate.

## Build & toolchain

| invariant | enforced by |
|---|---|
| Rust toolchain pinned to the channel in `rust-toolchain.toml` | `rust-toolchain.toml` consumed by `dtolnay/rust-toolchain` in `.github/workflows/rust-quality.yml` |
| `cargo build --release --workspace --locked` succeeds with `-D warnings` | `rust-quality.yml` step "build" |
| Every binary `--help` returns in < 200 ms on dev hardware | manual check; benchmark step not yet wired in `rust-quality.yml` |

## Source quality

| invariant | enforced by |
|---|---|
| `cargo fmt --all -- --check` is clean | `rust-quality.yml` step "rustfmt" |
| `cargo clippy --workspace --all-targets` is clean with `-D warnings` | `rust-quality.yml` step "clippy" |
| Test suite is green: `cargo test --release --workspace --locked` | `rust-quality.yml` step "test" |
| No file in `crates/**/*.rs` exceeds 700 LOC (soft cap 400) | review-time; no automated gate yet — proposal: `scripts/file-budget.sh` |

## Supply chain & licensing

| invariant | enforced by |
|---|---|
| All deps pass `cargo deny check` (licences ∈ {MIT, Apache-2.0, BSD-2/3, ISC, Unicode-DFS-2016, MPL-2.0, GPL-3.0-or-later, Zlib, OpenSSL}) | `rust-quality.yml` step "cargo-deny" against `deny.toml` |
| No unused dependencies (`cargo machete` clean) | `rust-quality.yml` step "cargo-machete" |
| Project licence is GPL-3.0-or-later; declared in every binary crate `Cargo.toml` | review-time |

## Security

| invariant | enforced by |
|---|---|
| `webapps-exec` only spawns browsers resolved via the whitelist in `webapps-core::browsers` (`find_def` + `BrowserDef`) | `webapps-exec/src/launch.rs` resolves only `native_paths`/`flatpak_app_id`; unit tests in `webapps-core/src/browsers.rs` |
| Atomic write for persisted permissions: `tmp + rename`, never overwrite-in-place | `webapps-viewer/src/window/permissions/mod.rs::save_permissions` |
| Inter-process file locking via an exclusive advisory lock (`fs2::FileExt::lock_exclusive`) on `WebappsLock` for any write transaction | `webapps-manager/src/service/repository.rs::WebappsLock` |
| All shell-quoted desktop-file `Exec=` lines pass `desktop::sanitize::sanitize_exec_arg` | unit tests in `webapps-core/src/desktop/sanitize.rs` (6 cases) |

## Resource lifecycle

| invariant | enforced by |
|---|---|
| No long-lived signal handler holds a strong `Rc` back to its owner widget — all use `#[weak]` or capture-by-value | review-time; `#[weak]`/`Weak` usages across `crates/webapps-manager/src` |
| Every `glib::SourceId` debounce is `.remove()`d before being replaced | `webapp_dialog/handlers/{lifecycle,fields}.rs` pattern; review-time |
| `WebappsLock::drop` releases the advisory lock even on panic | `repository.rs` `Drop for WebappsLock` impl |

## i18n

| invariant | enforced by |
|---|---|
| 100 % of user-visible strings are routed through `gettext` / `ngettext` | review-time; POT freshness gate (`rust-quality.yml` step "i18n POT freshness") catches new untranslated strings |
| `po/biglinux-webapps.pot` matches `xtr` output from current sources | `rust-quality.yml` step "i18n POT freshness" |
| `.po` catalogs round-trip via `msgmerge` (handled by `scripts/update-translations.sh`) | script invoked by CI step above |

## Accessibility

| invariant | enforced by |
|---|---|
| Every icon-only `Button` has an accessible name via `update_property(accessible::Property::Label, …)` | review-time; live AT-SPI walk in CI not yet wired |
| Destructive actions are never the default response in `adw::AlertDialog` | manual review; `list.rs:197` sets default_response = "cancel" |
| Toast announcements use `adw::ToastOverlay` (role=status) for transient state | enforced by widget choice |

## Performance budgets

Target budgets (not yet wired into CI; validated manually on a VM).

| metric | budget |
|---|---:|
| `big-webapps-exec` size (release, stripped) | ≤ 800 KB |
| `big-webapps-gui` size | ≤ 7 MB |
| `big-webapps-viewer` size | ≤ 3 MB |
| Manager cold RSS @ +1 s | ≤ 90 MB |
| Viewer cold RSS @ +1 s | ≤ 180 MB |
| Manager idle wakeups/s | ≤ 5 |
| Viewer idle wakeups/s | ≤ 10 |
| "Create webapp" p50 | ≤ 800 ms |
| "Launch webapp" p50 | ≤ 700 ms |
| CI test runtime | ≤ 30 s |

## Enforcement entrypoint

The single committed enforcement point is `.github/workflows/rust-quality.yml`
(this file plus that workflow). Internal audit scratch is local-only and not
shipped; every contract above repoints to the real gate, the source path, or an
honest "review-time" / "not yet wired" marker.

## How to extend

To add an invariant: file a row above with a concrete enforcement path. To weaken one: open a PR that updates both this file *and* the enforcing check in the same diff. Drift between the table and the gate is a bug — reviewers should reject either change in isolation.
