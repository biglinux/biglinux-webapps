# Stage 3 — Semantic duplication

Signal: identical name + identical/near-identical body across crates. Cross-checked sibling BigLinux Rust checkouts under `../big-rust-components`, `../big-media-runtime` (if present locally — none of these helpers exist there yet, so they would be net-new additions for one consumer; keep them in `webapps-core` for now).

## Clusters

### Cluster A — `init_logger` *(byte-identical body)*
| site | LOC |
|---|---|
| `crates/webapps-manager/src/main.rs:26` | 3 |
| `crates/webapps-viewer/src/main.rs:63` | 3 |

Body: `env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();`

**Canonical**: new `webapps_core::logging::init_default()` (1-line wrapper). Each main.rs becomes `webapps_core::logging::init_default();`. Lines saved: ~4 (the two function definitions collapse to a one-line shim each).

### Cluster B — `load_css` *(scaffold-identical, payload differs)*
| site | LOC |
|---|---|
| `crates/webapps-manager/src/style.rs:58` | 12 |
| `crates/webapps-viewer/src/window/loading.rs:57` | 14 (incl. `Once` guard) |

Both: `CssProvider::new` → `load_from_data(&'static CSS)` → `style_context_add_provider_for_display(default_display, …, PRIORITY_APPLICATION)`.

**Canonical**: `webapps_core::style::register_css(css: &'static str)` that calls the GTK incantation once per CSS payload (cache by pointer in a `Mutex<HashSet<*const u8>>` — pointer identity is fine since callers pass `&'static`). The viewer's `Once` becomes implicit. Manager's call site loses `style.rs` framing.

Lines saved: ~12 + de-duplicates the GTK wiring (one source of truth if Adwaita changes the priority constant).

### Cluster C — `glib_markup_escape` *(trivial wrapper)*
| site | LOC |
|---|---|
| `crates/webapps-manager/src/webapp_row.rs:140` | 3 (returns `glib::GString`) |
| `crates/webapps-manager/src/window/actions/remove_multiple.rs:232` | 3 (returns `String`) |

Both delegate to `glib::markup_escape_text`. Wrappers add no value beyond a name. Both call sites can use `glib::markup_escape_text(v)` (manager) / `glib::markup_escape_text(v).to_string()` (remove_multiple) inline.

**Canonical**: delete both wrappers; inline `glib::markup_escape_text` at call sites. Lines saved: 6.

### Cluster D — geometry persistence *(divergent re-implementation)*
| site | LOC | scope |
|---|---|---|
| `crates/webapps-manager/src/geometry.rs` | 163 | `IsA<gtk::Window>` (general) |
| `crates/webapps-viewer/src/window/geometry.rs` | 61 | concrete `adw::ApplicationWindow` only |

Same JSON shape (`{"width","height","maximized"}`), same atomic write pattern intent (viewer writes non-atomically — bug-prone), same fallback flow. Viewer's narrower copy exists because the manager helper lives in a sibling crate; viewer cannot reach it.

**Canonical**: move geometry persistence to `webapps_core::geometry` and have both crates depend on it. Picks up:
1. atomic `tmp + rename` from manager
2. fullscreen-skip + maximized-bool from viewer
3. type bound `IsA<gtk::Window>` from manager (works for `adw::ApplicationWindow`, `adw::Window`, dialogs)

Lines saved: ~60 in viewer, plus the single-source-of-truth win — the JSON schema can't drift.

## Not duplication (false positives)

- `fn templates() -> Vec<WebAppTemplate>` × 5 (`templates/{google,media,office365,communication,productivity}.rs`) — intentional registry pattern, each returns a distinct catalogue. **Keep.**
- `fn show(…)` × 4 across dialogs — disambiguated by module path; signatures + bodies differ. **Keep.**
- `fn build(…)` × many — same. **Keep.**

## Action plan (after ack)

1. Land canonical helpers in `webapps-core` (`logging`, `style`, `geometry` modules) — single commit, additions only.
2. Migrate each consumer crate to the canonical helpers — one commit per consumer (`webapps-manager`, then `webapps-viewer`).
3. Delete the inline wrappers from Cluster C — third commit.
4. After all callers migrated, delete the old per-crate impls. Final commit.

Total expected reduction: ~80 LOC plus elimination of two divergence-prone copies (geometry, css). All clusters are crate-internal — no public-API churn outside `webapps-core` (where additions only).
