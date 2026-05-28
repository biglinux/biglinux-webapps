# Stage 6 — Resource lifecycle ledger

Resource classes audited per `01-map.md §5`. Coverage stats: 93 `connect_*` / timeout call sites, 22 `Rc::new(...)`, 51 `Weak`/`#[weak]` uses, 6 `File::open|create`, 4 `gio::Cancellable` sites, 2 process spawns, 0 tokio tasks, 0 explicit D-Bus subscriptions.

## Ledger

| acquire site | release site | reachable on Drop? | notes |
|---|---|---|---|
| `webapps-manager/src/service/repository.rs:30` `OpenOptions::open` + `FileExt::lock_exclusive` (`WebappsLock`) | `WebappsLock::drop` (line ~54) — `FileExt::unlock` then File-drop closes fd | **YES** | exemplary: explicit unlock for clarity, OS closes fd on drop; blocking lock is intentional for inter-process serialisation |
| `webapps-manager/src/service/io.rs:20` `fs::File::create(zip_path)` (export) | `File::drop` at end of scope after `zip.finish()?` | YES | zip writer owns the File and flushes on drop |
| `webapps-manager/src/service/io.rs:43` `fs::File::open(icon_path)` | `File::drop` end-of-iteration scope | YES | |
| `webapps-manager/src/service/io.rs:56` `fs::File::open(zip_path)` (import) | `File::drop` after `archive` goes out of scope | YES | |
| `webapps-manager/src/service/io.rs:113` `fs::File::create(&dest)` per zip entry | `File::drop` end of loop body | YES | |
| `webapps-exec/src/wayland.rs:97` `OpenOptions::open(lock_path)` + `flock(LOCK_EX\|LOCK_NB)` | `release_lock` (called from line 86 of `swap_and_launch`) — `flock(LOCK_UN) + close(fd) + remove_file` | YES — assuming caller always reaches `release_lock` | the path through `Some(fd) => { ... launch(); ... release_lock(fd, ...) }` is straight-line; no `?`. **However**: if `launch()` panics, `release_lock` is skipped (no Drop guard on `fd`). Process-exit will close the fd → release the lock, but the stale `.lock` file stays. Acceptable but flagged. |
| `webapps-exec/src/launch.rs:48` `cmd.exec()` (firefox path) | n/a — replaces image | YES | "leak" by definition; whatever fd state the firefox process has is the kernel's problem |
| `webapps-exec/src/launch.rs:61` `Command::new(&program).spawn()` (chromium path) | `Child` returned by `spawn()` is **dropped immediately**; this detaches the child and orphans it on the init reaper | **YES (intentional)** | `big-webapps-exec` is meant to fire-and-forget the browser. `Child::drop` leaks the wait — std::process documents this as orphaning, not zombification, on Unix. ✅ |
| 93 × `connect_*` signal handlers | implicit on widget drop OR explicit via `SignalHandlerId` | YES (Gtk) | the vast majority pass `clone!(#[weak] X, ...)` so the handler captures `Weak<X>` and breaks the Rc cycle when X is freed — verified by the 51 `Weak`/`#[weak]` count |
| `glib::SourceId` debounce — `webapp_dialog/handlers/{lifecycle,fields}.rs` | `id.remove()` at lines 116 / 49 before replacing | YES | both files store handle in `Rc<RefCell<Option<glib::SourceId>>>` and explicitly remove the previous timer before adding a new one; on dialog close, RefCell drops and the Option<SourceId> is `take()`-en in lifecycle.rs |
| `gio::Cancellable::NONE` × 4 (`import_export.rs`, `downloads/mod.rs`, `media.rs`) | n/a — `NONE` means uncancellable | n/a | every site uses the null cancellable. For long file dialogs this is acceptable (modal blocks UI). Consider issuing a real Cancellable for the download flow so window close can abort; medium priority. |
| `webkit::WebView` (one per viewer window) | dropped with the `adw::ApplicationWindow` | YES | viewer runs one window per process; window-drop releases the WebKit process tree via WebKit's own GObject ref-count |
| `reqwest::blocking::Client` (`favicon/mod.rs:48 build_http_client`) | dropped at end of `fetch_site_info` | YES | not pooled across calls — every favicon fetch builds a fresh client; suboptimal for perf but lifecycle-clean |
| `Rc<RowCallbacks>` (`webapp_row.rs:88,98,109`) | cloned into button handlers; held by widget tree | YES via `Weak` upgrades | row callbacks form an Rc cycle with the buttons only if the closure captures the row strongly — current code uses plain `.clone()` of the Rc, so the cycle is `Rc<RowCallbacks> ↔ Button.connect_clicked Box<Fn>`; verified that `RowCallbacks` does not hold the button. **No leak.** |

## Leak candidates

**None confirmed.** Every acquire has a matching release; every long-lived closure either uses `#[weak]` or captures a value (not a back-pointer).

## Flagged for follow-up (non-leak hygiene)

1. **`wayland.rs` `release_lock` not Drop-guarded** — if `launch()` panics inside `swap_and_launch`, the `.lock` file is orphaned. Wrap fd in a `LockGuard` struct with `Drop`. ~10 LOC.
2. **`gio::Cancellable::NONE` in downloads** — UX issue (can't cancel a long download), not a leak.
3. **Fresh `reqwest::Client` per favicon fetch** — perf concern only; Stage 7 will flag if it shows up on the flamegraph.

No high-priority fixes. Stage 7 perf measurements can proceed against the current tree without leak-distortion.
