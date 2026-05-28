# Stage 12 — Test suite review

Counted via `grep -E "^\s*#\[test\]|fn test_"` across `crates/`. **Total: 207 test functions** in 25 files (including `crud_integration.rs` integration suite).

## Coverage by surface

| crate | unit-tests (in src) | integration | hot areas covered |
|---|---:|---:|---|
| `webapps-core` | 65 | 0 | `models::webapp` (19), `desktop::builder` (18), `templates::registry` (11), `desktop::sanitize` (6), `desktop::icon` (5), `browsers` (6) |
| `webapps-manager` | 73 | 13 | `favicon::html` (15), `service::browser` (8), `validation` (7), `icons` (7), `migration::{shell,parse}` (12), `browser_url` (5), `crud` (5), `state` (5), `import_export` (4), `favicon::download` (6), `webapp_dialog::handlers::fields` (2) |
| `webapps-viewer` | 22 | 0 | `navigation::webview` (6), `startup` (5), `navigation::url_entry` (4), `permissions` (4), `fullscreen` (3) |
| `webapps-exec` | 7 | 0 | `wayland` (7) |

Integration suite (`crud_integration.rs`): exercises end-to-end create/update/delete against a temp `XDG_DATA_HOME`, with desktop-file + icon assertions. Real I/O, no mocks. This is the single most valuable test in the workspace.

## Keep / delete / add matrix

### Keep (high signal)

| test | reason |
|---|---|
| `desktop::builder` 18 cases | covers exec-line escape rules, AppMode → cmd mapping, profile arg propagation. Regressions here brick every launch. |
| `desktop::sanitize` 6 cases | the only line of defence against command injection (Stage 5 H-1). **Critical.** |
| `models::webapp` 19 cases | round-trip of the persistence schema; protects migration correctness. |
| `favicon::html` 15 cases | HTML scraping is the messiest input the app handles. |
| `migration::{shell,parse}` 12 cases | schema-bump path. Test data is the only insurance against silent corruption on upgrade. |
| `wayland::swap_and_launch` 7 cases | flock semantics + symlink-swap correctness. |
| `crud_integration` 13 cases | end-to-end + filesystem effects. |

### Delete (none)

No obviously redundant or trivial tests found. All have a clear contract under test.

### Add (gaps)

| gap | severity | proposed test |
|---|---|---|
| **G-1** `webapps-exec::launch` (browser dispatch) | **high** | table-driven: `BrowserId::FIREFOX` → `firefox`, `BrowserId::CHROMIUM_PROFILE` → `chromium --profile-directory=…`, `BrowserId::VIEWER` → `big-webapps-viewer`. Currently 0 tests in this module. Argv-construction bug = silent wrong-browser launch. |
| **G-2** `webapps-exec` browser-id whitelist (Stage 5 H-1 fix-companion) | **high** | once whitelist lands, assert: malicious `BrowserId("rm -rf ~")` returns `BrowserDispatch::Reject`, not `Spawn`. |
| **G-3** `webapps-manager::favicon::fetch_site_info` HTTP-level | medium | wiremock or `httpmock` to spin a local server returning crafted `<link rel>` permutations; current `html.rs` tests cover parsing but not the HTTP+merge pipeline. |
| **G-4** atomic permissions save (just landed in Stage 5) | medium | write to a temp dir, induce a "rename target is open" race, assert no partial file appears (Linux: `O_TMPFILE`-equivalent invariant). |
| **G-5** `webapps-viewer::shortcuts::window_actions` | medium | accelerator → action table — currently no tests. |
| **G-6** i18n catalog round-trip | low | `xtr` → POT diff against committed POT; gate in CI (Stage 13). |

### Re-balance — none

No test moves needed. Test code is co-located with the module it covers (Rust idiomatic). The integration test correctly lives under `crates/webapps-manager/tests/`.

## Property / fuzz candidates

Three modules have natural property-test angles:

1. `desktop::sanitize::sanitize_exec_arg` — `quickcheck` "for all UTF-8 strings, output never contains an unescaped `\``, `$`, or newline". 1 prop, ~30 LOC.
2. `models::webapp::app_file_from` — "for all `(browser, url)` pairs, resulting filename is a valid POSIX path component". ~30 LOC.
3. `favicon::html::parse_meta` — corpus fuzzing with `cargo-fuzz` on saved real-world HTML. Defer to a follow-up; not blocking.

Property tests added would bring the suite from 207 → ~210 with disproportionate confidence gain.

## Flakiness audit

`crud_integration.rs` uses `tempfile::tempdir()` per test → no shared state. The wayland tests use mocked `FdLockGuard`. No timing-based assertions in the suite. **No known flakes.**

## Test-runtime budget

From `07-perf.md`: full `cargo test --release --workspace` finishes in ~2.5 s. Adding G-1…G-5 adds at most ~1 s (HTTP mock startup dominates). Budget remains under 30 s CI cap.

## Verdict

207 tests is a healthy baseline for an 11.6 kLOC GTK app. Coverage maps onto the highest-risk surfaces (sanitisation, migration, exec/launch flock, desktop-file builder). The single material gap is **`webapps-exec::launch` having zero tests** — a binary that exec()s arbitrary browsers should not be untested at the dispatch layer.

**Action items going into Stage 13:** add G-1 + G-2 as part of the H-1 fix commit; add G-3 + G-4 as a follow-up; defer G-5 + G-6 to backlog.
