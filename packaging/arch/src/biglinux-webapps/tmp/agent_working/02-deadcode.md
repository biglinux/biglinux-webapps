# Stage 2 — Dead-code amputation

Source signals: `cargo clippy --all-targets --locked` (0 warnings), `cargo build` (0 warnings), `cargo machete`, manual sweep of `01-map.md` boundaries.

## Findings

| path:line | kind | last touched | proposed action |
|---|---|---|---|
| `crates/webapps-viewer/Cargo.toml:24` | dep `anyhow` | imported | **delete** — no `use anyhow` / `anyhow::` anywhere under `webapps-viewer/src`; viewer uses `glib::Error` + `Option` only |
| `crates/webapps-viewer/Cargo.toml:25` | dep `dirs` | imported | **delete** — paths flow through `webapps-core::config` helpers; no `dirs::` reference |
| `crates/webapps-viewer/Cargo.toml:18` | dep `serde` | imported | **delete** — only `serde_json` is used (raw `Value` + `json!` macro); no `#[derive(Serialize/Deserialize)]` in viewer crate |
| — | `pub` items at workspace boundary nothing imports | n/a | **none** — every `pub` re-exported by `webapps-core` lib has at least one consumer in `webapps-manager`/`webapps-exec`/`webapps-viewer` (verified by grep across the 4 crates) |
| — | private items unreachable from entry points | n/a | **none surfaced** — `cargo build --all-targets` emits 0 `dead_code` warnings; lint is on by default |
| — | Cargo features never enabled | n/a | **keep** — only feature in tree is `zip/deflate` (always enabled) |
| — | resources not loaded at runtime | n/a | **none** — only embedded asset is `biglinux-webapps/usr/share/biglinux-webapps/browsers.toml` via `include_str!` at `webapps-core/src/browsers.rs:53`; loaded at every `browser_defs()` call |
| — | `#[allow(dead_code)]` workarounds | n/a | **none** — zero hits in `crates/*/src` |

## Verification

- `cargo machete` exits with the 3 deps above and nothing else.
- Cross-grep confirms callers exist for every `pub` listed in `01-map.md §3`.
- `RUSTFLAGS="-W dead_code -W unused" cargo check --workspace --all-targets` adds no new warnings.

## Action plan

Single commit, viewer crate only:

```diff
 [dependencies]
 webapps-core = { path = "../webapps-core" }
 gtk4.workspace = true
 libadwaita.workspace = true
 webkit6.workspace = true
 glib.workspace = true
 gio.workspace = true
 gdk4.workspace = true
-serde.workspace = true
 serde_json.workspace = true
 log.workspace = true
 env_logger.workspace = true
-anyhow.workspace = true
-dirs.workspace = true
 clap.workspace = true
 gettextrs.workspace = true
 url.workspace = true
```

Re-run `cargo test --workspace --locked` after. Stage 1 cartography unaffected (no source files removed).
