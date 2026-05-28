# Stage 7 — Performance baseline & budgets

Captured on dev host (no display). Live-RSS / wakeups / flamegraph measurements need a Wayland session — to be collected on the BigLinux VM as part of Stage 13 CI wiring (see `linux-ui-a11y` skill for `kwin_wayland --virtual` recipe). Static + cold-CLI numbers captured here.

## Build & binary baseline

`cargo build --release --workspace --locked` — finished in **72 s** on a warm cache.

| binary | role | size (release, stripped) | budget |
|---|---|---:|---:|
| `big-webapps-exec` | desktop-file launcher (run-once, then `exec()`) | 643 KB | ≤ 800 KB |
| `big-webapps-gui` | manager (libadwaita + gtk4 + reqwest blocking) | 5.5 MB | ≤ 7 MB |
| `big-webapps-viewer` | per-window WebKit shell | 2.4 MB | ≤ 3 MB |

Release profile is already optimal for size: `lto = "thin"`, `opt-level = "s"`, `strip = true`. No `panic = "abort"` — adopting it would save ~10% in `big-webapps-gui` but lose backtraces; **keep**.

## CLI cold time-to-help (no GUI init)

| binary | wall | user | sys | notes |
|---|---:|---:|---:|---|
| `big-webapps-exec` (no-arg error path) | 1 ms | 0 ms | 2 ms | minimal argv parse; never reaches GTK |
| `big-webapps-viewer --help` | 94 ms | 33 ms | 40 ms | clap parses before GTK init; sys time dominated by dynamic-linker resolve of webkit6 dependencies |
| `big-webapps-gui --help` | 34 ms | 23 ms | 11 ms | uses gtk-application help routing |

Budget: every binary `--help` must return < 200 ms on dev hardware (NVMe SSD, glibc 2.42). Regression > 50 ms triggers review.

## Test-suite latency

`cargo test --release --workspace` — 158 tests, all bins, finished in **~2.5 s** runtime (compile-time excluded; test cumulative `finished in` lines sum to 0.06 s wall, plus binary load).

Budget: total test runtime ≤ 30 s on CI. Stage 12 must not exceed.

## Measurements deferred to live display (VM)

Pipeline calls for, on the running app:

1. **Cold-start RSS at +1 s after window-shown** — `big-webapps-gui` and `big-webapps-viewer`.
2. **Idle RSS at +60 s** — both binaries.
3. **Idle wakeups/s** via `perf stat -e sched:sched_switch -p <pid> sleep 30`.
4. **p50/p99 latency for top-3 user actions**:
   - **Create webapp** (open `webapp_dialog` → fill → save → desktop entry written + icon persisted) — touches `service::repository`, `desktop::builder`, `favicon::fetch_site_info` (network), `icons::resolve_icon_path`.
   - **Launch webapp** (`big-webapps-exec` → `exec(firefox)` or `spawn(chromium)`) — touches `webapps-core::browsers`, `wayland::swap_and_launch` (flock + rename + sleep 500 ms).
   - **Viewer first paint** (`big-webapps-viewer --url <…>` → `webkit::WebView::load_uri`) — dominated by WebKit init + DNS + TLS.
5. **Largest single allocation** under "import 100-webapp zip" — `heaptrack`.
6. **Top-10 hot functions** — `cargo flamegraph -p webapps-manager --bin big-webapps-gui`, scripted CRUD workload.

Place those in `07-perf-vm.md` once collected; Stage 13 CI gate compares against budgets.

## Provisional budgets (to validate on VM)

| metric | budget | rationale |
|---|---:|---|
| Manager cold RSS @ +1s | ≤ 90 MB | comparable libadwaita apps (`gnome-software` ≈ 110 MB; manager has no app-grid loader) |
| Viewer cold RSS @ +1s | ≤ 180 MB | WebKit web process + UI process; WebKitGTK 6 minimum ≈ 150 MB |
| Idle wakeups/s (manager) | ≤ 5 | no timers running idle except gtk frame-clock |
| Idle wakeups/s (viewer) | ≤ 10 | WebKit may run rAF |
| "Create webapp" p50 | ≤ 800 ms | favicon HTTP fetch dominates; one DNS + TLS + ≤ 5 MB body |
| "Launch webapp" p50 | ≤ 700 ms | dominated by `swap_settle()` 500 ms + browser exec |
| `swap_settle` wait | ≤ 500 ms | already constant; reducible if compositor confirms via `wl_surface.commit` event |

## Quick wins identified without live measurement

1. **`reqwest::Client` rebuilt per favicon call** (`favicon/mod.rs:48`). On a multi-icon scan (link rel="icon" + apple-touch + manifest icons) one webapp build does 3-6 HTTP requests, each paying the TLS handshake. Cache a `Client` in an `OnceLock`. Likely 200-400 ms saved per "Create webapp".
2. **`swap_settle = 500 ms` is unconditional** (`webapps-exec/src/wayland.rs:24`). The compositor publishes a `wl_surface.commit` on icon update; subscribing reduces median launch latency by ~400 ms. Higher complexity — defer.
3. **`webapps-manager` favicon scan walks HTML on the UI thread** (`favicon::fetch_site_info`). Moving to a glib worker thread is straightforward and removes the visible UI hitch during "Create webapp". Stage 11 will likely move this anyway.

No speculative micro-opts — wait for VM flamegraph before any function-level work.
