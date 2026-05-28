# Stage 0 — Readiness gate

Repo: `/home/bruno/codigo-pacotes/biglinux-webapps`  ·  Branch: `main`  ·  Date: 2026-05-16

| # | Check | Result | Evidence |
|---|-------|--------|----------|
| 1 | `git status` clean | **PASS** | `git status --short` → empty |
| 2 | `cargo build --locked` | **PASS** | dev profile, 1m02s, no warnings of note |
| 3 | `cargo test --workspace --locked` | **PASS** | 158 tests across 8 binaries (59+6+63+0+13+17+0+0), 0 fail |
| 4 | Toolchain pinned (`rust-toolchain.toml`) | **FAIL** | file missing |
| 5 | CI config present + recent | **PASS** | `.github/workflows/build-package.yml`, mtime 2026-05-16 |
| 6 | LICENSE + SPDX in Cargo.toml | **PASS** | `LICENSE` at root + `license = "GPL-3.0-or-later"` workspace-wide; sub-crates inherit |

## FAIL: toolchain pin

CI reproducibility risk — a contributor on a newer/older nightly silently drifts. PIPELINE Stage 13 also requires a CI gate referencing the pinned toolchain.

### Minimum fix

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.95.0"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

Channel = current MSRV that matches the existing `Cargo.lock` resolver and gtk4/webkit6 crate-version floors. Bump to current stable when CI is wired (Stage 13).

## Notes (non-blocking, surfaced to later stages)

- No per-file SPDX header in any `lib.rs`/`main.rs`. Acceptable under repo-root LICENSE for GPL distribution; Stage 4 will decide whether to add headers.
- No `deny.toml` — Stage 4 will propose one.
- No `clippy` run in this gate — deferred to Stage 13 CI wiring.

## Decision

Per user "no-pause" directive: applying minimum fix (toolchain pin) inline, then proceeding to Stage 1.
