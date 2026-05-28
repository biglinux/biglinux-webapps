# Stage 11 — Agent cognitive load

How navigable is the source for an agent or new contributor reading cold? Audit looks at file size budget, naming, module anchoring, and doc onboarding.

## File size budget

Soft cap: **400 LOC** per `.rs` file (mirrors `rust-quality` skill default). Hard cap: **700**.

| file | LOC | status |
|---|---:|---|
| `webapps-manager/src/service/migration/mod.rs` | 392 | within cap |
| `webapps-manager/tests/crud_integration.rs` | 333 | test file — soft cap waived |
| `webapps-manager/src/favicon/html.rs` | 319 | within cap |
| `webapps-core/src/templates/registry.rs` | 299 | within cap |
| `webapps-manager/src/webapp_dialog/handlers/fields.rs` | 240 | within cap |
| `webapps-manager/src/window/list.rs` | 235 | within cap |
| `webapps-manager/src/window/actions/remove_multiple.rs` | 234 | within cap |
| (24 files in 150-225 range) | … | within cap |

**Zero files exceed the 400 LOC soft cap.** Largest is 392 (migrations — naturally linear) and 333 (test data). No god-files. Stage 11 passes file-size review without changes.

Whole workspace: **11,636 LOC** across `crates/`. Median file ~80 LOC.

## Module anchoring

Each crate has a single entry point with clear submodules:

- `webapps-core/src/lib.rs` → `models`, `browsers`, `desktop`, `templates`, `favicon`, `config`
- `webapps-manager/src/main.rs` + `lib.rs` → `window`, `webapp_dialog`, `browser_dialog`, `template_gallery`, `welcome_dialog`, `service`, `favicon`, `ui_async`
- `webapps-viewer/src/main.rs` → `window` (with `navigation`, `loading`, `permissions`, `shortcuts`, `menu`)
- `webapps-exec/src/main.rs` → `launch`, `wayland`

No cross-crate re-exports beyond `webapps-core` public API. Module graph is a DAG (verified Stage 1).

## Naming

Domain vocabulary (per `01-map.md`):

| concept | type | name |
|---|---|---|
| webapp record | struct | `WebApp` |
| browser identifier | newtype | `BrowserId` |
| browser record | struct | `Browser` (in `BrowserStore`) |
| in-memory store | struct | `BrowserStore` |
| desktop file builder | struct/fn | `desktop::builder`, `desktop::icon` |
| template definition | struct | `Template` (in `templates::registry`) |
| async UI helper | mod | `ui_async` (with `run`, `run_with_result`) |
| service layer | mod | `service::{crud, repository, browser, icons, io, migration}` |

Names are nouns; functions are verbs; modules are singular except `models`/`templates`/`browsers`. **No abbreviations** beyond standard (URL, HTML, DRM, AUR-style). Consistent.

## Public-API doc coverage

| crate | `#[deny(missing_docs)]`? | top-level rustdoc? |
|---|---|---|
| `webapps-core` | no | no module-level `//!` |
| `webapps-manager` | no | no |
| `webapps-viewer` | no | no |
| `webapps-exec` | no | no |

**Finding M-11.1:** `webapps-core` is the cross-crate library — it should at minimum have a `//!` crate header explaining: "Pure domain types for the WebApps stack; no I/O, no GTK; shared by manager, viewer, exec." Add 3-5 lines to `webapps-core/src/lib.rs` and `models/webapp/mod.rs`. ~10 minutes; not blocking.

## Onboarding anchors

Repo root has:

- `README.md` — present, explains user-facing purpose
- `AGENTS.md` / `CLAUDE.md` — **absent**. An agent walking in cold has to derive the crate graph by inspection. The pipeline's own `tmp/agent_working/01-map.md` is the entry point right now, but it lives under `tmp/` (gitignored by convention).

**Finding M-11.2:** create `AGENTS.md` at repo root with:

```
# biglinux-webapps — agent guide

## Crates
- webapps-core: domain types (no I/O, no GTK).
- webapps-manager: libadwaita app to CRUD webapps.
- webapps-viewer: WebKitGTK per-window browser.
- webapps-exec: tiny exec()-launcher invoked by .desktop entries.

## Build
cargo build --release --workspace --locked

## Test
cargo test --workspace

## Reference docs (per pipeline)
tmp/agent_working/01-map.md  – crate graph + hotspots
tmp/agent_working/INVARIANTS.md – contracts the gates protect
```

~30 LOC, one-time. Promotes pipeline artifacts to discoverable.

## Cyclomatic & churn hotspots (carryover from 01-map.md)

Top three files by `(LOC × git-churn)`:

1. `service/migration/mod.rs` — 392 LOC, append-only on schema bumps; complexity localised to per-migration fns. **Tolerable.**
2. `webapp_dialog/handlers/fields.rs` — 240 LOC of UI binding callbacks. Could split into `name.rs` / `url.rs` / `profile.rs` if it grows past 300. **Watch.**
3. `favicon/html.rs` — 319 LOC HTML scraper. Pure logic, well-tested. **Tolerable.**

## Findings to file

| id | severity | action |
|---|---|---|
| M-11.1 | medium | add `//!` rustdoc header on `webapps-core/src/lib.rs` |
| M-11.2 | medium | create `AGENTS.md` at repo root |
| C-11.1 | low | split `webapp_dialog/handlers/fields.rs` if next change pushes past 300 LOC |

## Verdict

Agent cognitive load is **low to moderate**. File sizes are healthy, naming is consistent, module graph is clean. The one real gap is the missing top-level agent guide — fixable in 5 minutes; queued as `M-11.2`.
