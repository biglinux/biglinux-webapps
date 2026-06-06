# biglinux-webapps — Audit Readiness — 2026-05-27

## Score: 10.0 / 10

GTK4 + libadwaita Rust webapp manager + viewer + exec runtime. Cargo
workspace of 4 crates (webapps-core, webapps-exec, webapps-manager,
webapps-viewer).

## Gate matrix

| Gate | Cmd | Result |
|---|---|---|
| R1 fmt | `cargo fmt --all -- --check` | PASS |
| R2 clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| R3 tests | `cargo nextest run --workspace --all-features` | PASS — 158/158 |
| R4 doc | `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --all-features` | PASS (after `SYSTEM_PATH` doc link fix) |
| R5 deny | `cargo deny check` | PASS |
| R6 audit | `cargo audit --ignore RUSTSEC-2024-0436 --deny warnings` | PASS |
| R7 machete | `cargo machete --with-metadata` | PASS (after `gio`/`relm4` removed) |

## Punch list — all verified

| ID | Action | Status |
|---|---|---|
| BWA-1 | `cargo deny`: `wildcards = "deny"` + `allow-wildcard-paths = true` properly configured. No bans/license violations. | DONE |
| BWA-2 | Component-check migrations: vendored `packaging/arch/src/*` excluded via packaging-dir ignore set. Active source tree is mid-migration to `big_app_kit::{desktop,dialogs,files}` + `big_relm4_components::feedback::tooltip` (Relm4 onda6). | DONE |
| BWA-3 | **Defense-in-depth: `base_spec` returns `Option`, never falls back to `browser_id` as program name.** `crates/webapps-exec/src/launch.rs:140-162` — if `def` has no resolved `flatpak_app_id` or no existing `native_paths` entry, the function returns `None` and the caller (`firefox`/`chromium`) logs + exits/returns rather than executing `Command::new(browser_id)`. Closes the LD_PRELOAD / registry-tampering vector that REVIEW-WORKSPACE flagged. | DONE |
| BWA-4 | 158 unit tests cover sanitize.rs adversarial inputs + CRUD. No `tests/` integration target (binary-only crates). | RESIDUAL (binary-only) |
| New | `SYSTEM_PATH` broken intra-doc link fixed in `crates/webapps-core/src/browsers.rs:62`. | DONE |
| New | Unused direct deps removed: `gio` (webapps-manager — uses `gtk::gio::*`), `relm4` (webapps-viewer — uses only `big_relm4_components`). | DONE |

## Residual accepted

- `RUSTSEC-2024-0436 paste` (transitive, accepted workspace-wide).

## Reproduce gates

```sh
cd biglinux-webapps   # repo root
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
cargo nextest run --workspace --all-features
cargo deny check
cargo audit --ignore RUSTSEC-2024-0436 --deny warnings
cargo machete --with-metadata
```
