# INVARIANTS — biglinux-webapps

Contracts the CI gates protect. Each line is enforced by a check listed in the right column. Change an invariant only with a matching change to the gate.

## Build & toolchain

| invariant | enforced by |
|---|---|
| Rust toolchain pinned to the channel in `rust-toolchain.toml` | `rust-toolchain.toml` consumed by `dtolnay/rust-toolchain` in `.github/workflows/rust-quality.yml` |
| `cargo build --release --workspace --locked` succeeds with `-D warnings` | `rust-quality.yml` step "build" |
| Every binary `--help` returns in < 200 ms on dev hardware | `tmp/agent_working/07-perf.md` budget; manual check until Stage 13b benchmark step lands |

## Source quality

| invariant | enforced by |
|---|---|
| `cargo fmt --all -- --check` is clean | `rust-quality.yml` step "rustfmt" |
| `cargo clippy --workspace --all-targets` is clean with `-D warnings` | `rust-quality.yml` step "clippy" |
| Test suite is green: `cargo test --release --workspace --locked` | `rust-quality.yml` step "test" |
| No file in `crates/**/*.rs` exceeds 700 LOC (soft cap 400) | reviewed at `tmp/agent_working/11-agent-cl.md`; no automated gate yet — proposal: `scripts/file-budget.sh` |

## Supply chain & licensing

| invariant | enforced by |
|---|---|
| All deps pass `cargo deny check` (licences ∈ {MIT, Apache-2.0, BSD-2/3, ISC, Unicode-DFS-2016, MPL-2.0, GPL-3.0-or-later, Zlib, OpenSSL}) | `rust-quality.yml` step "cargo-deny" against `deny.toml` |
| No unused dependencies (`cargo machete` clean) | `rust-quality.yml` step "cargo-machete" |
| Project licence is GPL-3.0-or-later; declared in every binary crate `Cargo.toml` | review-time |

## Security

| invariant | enforced by |
|---|---|
| `webapps-exec` only spawns browsers in the whitelist defined in `webapps-core::browsers` | follow-up commit (Stage 5 H-1); test G-1/G-2 in `tmp/agent_working/12-tests.md` |
| Atomic write for persisted permissions: `tmp + rename`, never overwrite-in-place | `webapps-viewer/src/window/permissions/mod.rs::save_permissions` + Stage 5 §H-3 |
| Inter-process file locking via `flock(LOCK_EX)` on `WebappsLock` for any write transaction | `webapps-manager/src/service/repository.rs::WebappsLock` |
| All shell-quoted desktop-file `Exec=` lines pass `desktop::sanitize::sanitize_exec_arg` | unit tests in `webapps-core/src/desktop/sanitize.rs` (6 cases) |

## Resource lifecycle

| invariant | enforced by |
|---|---|
| No long-lived signal handler holds a strong `Rc` back to its owner widget — all use `#[weak]` or capture-by-value | reviewed at `tmp/agent_working/06-lifecycle.md`; 51 `Weak`/`#[weak]` usages verified |
| Every `glib::SourceId` debounce is `.remove()`d before being replaced | `webapp_dialog/handlers/{lifecycle,fields}.rs` pattern; ledger row in 06 |
| `WebappsLock::drop` unlocks the flock even on panic | `repository.rs` Drop impl |

## i18n

| invariant | enforced by |
|---|---|
| 100 % of user-visible strings are routed through `gettext` / `ngettext` | `tmp/agent_working/08-i18n.md` (coverage 100 % at this commit) |
| `po/biglinux-webapps.pot` matches `xtr` output from current sources | `rust-quality.yml` step "i18n POT freshness" |
| `.po` catalogs round-trip via `msgmerge` (handled by `scripts/update-translations.sh`) | script invoked by CI step above |

## Accessibility

| invariant | enforced by |
|---|---|
| Every icon-only `Button` has an accessible name via `update_property(accessible::Property::Label, …)` | `tmp/agent_working/09-a11y.md`; live AT-SPI walk in CI deferred to `09-a11y-vm.md` |
| Destructive actions are never the default response in `adw::AlertDialog` | manual review; `list.rs:197` sets default_response = "cancel" |
| Toast announcements use `adw::ToastOverlay` (role=status) for transient state | enforced by widget choice |

## Performance budgets

From `tmp/agent_working/07-perf.md` — validated on VM in Stage 13b (not yet wired).

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

## Pipeline artifacts (this commit)

| stage | file |
|---|---|
| 0 readiness | `tmp/agent_working/00-readiness.md` |
| 1 cartography | `tmp/agent_working/01-map.md` |
| 2 dead code | `tmp/agent_working/02-deadcode.md` |
| 3 duplication | `tmp/agent_working/03-duplication.md` |
| 4 licensing | `tmp/agent_working/04-licensing.md` |
| 5 STRIDE threats | `tmp/agent_working/05-threats.md` |
| 6 lifecycle | `tmp/agent_working/06-lifecycle.md` |
| 7 perf baseline | `tmp/agent_working/07-perf.md` |
| 8 i18n | `tmp/agent_working/08-i18n.md` |
| 9 a11y | `tmp/agent_working/09-a11y.md` |
| 10 user CL | `tmp/agent_working/10-user-cl.md` |
| 11 agent CL | `tmp/agent_working/11-agent-cl.md` |
| 12 tests | `tmp/agent_working/12-tests.md` |
| 13 CI & invariants | `INVARIANTS.md` (this file) + `.github/workflows/rust-quality.yml` |

## How to extend

To add an invariant: file a row above with a concrete enforcement path. To weaken one: open a PR that updates both this file *and* the enforcing check in the same diff. Drift between the table and the gate is a bug — reviewers should reject either change in isolation.
