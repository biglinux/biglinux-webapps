# PLANNING.md — BigLinux WebApps v4.0.0 Full Audit

> Generated: 2025-06-23 | Updated: 2025-06-25 (Phase 2 fixes applied)  
> Codebase: 26 .rs files, ~5100 LOC, 3 crates  
> Tooling: cargo clippy, cargo fmt, cargo audit, manual review, GTK4/Adwaita audit, Orca a11y audit

---

## 0. Project Health Summary

| Metric | Before | After | Status |
|--------|--------|-------|--------|
| Crates | 3 | 3 | ✅ |
| Total Lines | 4628 | ~5100 (+tests) | ✅ |
| Files >300L | 4 | 4 | ⚠️ |
| Clippy warnings | 33 | **0** | ✅ Fixed |
| Format diffs | 74 | **0** | ✅ Fixed |
| Tests | 0 | **25** (36 assertions) | ✅ Fixed |
| Unsafe blocks | 0 | 0 | ✅ |
| `.unwrap()` calls | 2 | **0** (→ `.expect()`) | ✅ Fixed |
| Secrets/hardcoded creds | 0 | 0 | ✅ |
| cargo-audit | Unknown | **0 vulns, 2 warnings** (transitive) | ✅ |
| README accuracy | 🔴 Python | **✅ Rust** | ✅ Fixed |
| A11y icon buttons | 🔴 ~15 unnamed | **✅ All labeled** | ✅ Fixed |
| ITP/permissions docs | ⚠️ No comments | **✅ Documented** | ✅ Fixed |

### Cargo.toml Health

| Dependency | Version | Notes |
|------------|---------|-------|
| gtk4 | 0.11 (v4_10) | Current stable |
| libadwaita | 0.9 (v1_6) | Current stable |
| webkit6 | 0.6 (v2_50) | Current stable |
| reqwest | 0.12 (native-tls, blocking) | ✅ native-tls for CI compat |
| scraper | 0.22 | OK |
| image | 0.25 | OK |
| zip | 2.0 (via crate dep) | OK |
| clap | 4 | OK |
| anyhow | 1 | OK |
| tokio | 1 | ⚠️ Only used for zbus — verify necessity |

### Release Profile

```toml
lto = "thin"       # good balance
opt-level = "s"     # size-optimized
strip = true        # no debug symbols
codegen-units = 1   # max optimization
```

✅ Appropriate for distribution binary.

---

## 1. Critical Issues (Must Fix Before Release) — ✅ ALL RESOLVED

### 1.1 ✅ ~~Zero Test Coverage~~ → 25 tests passing

**Impact:** No regression safety net. Any change can silently break functionality.

**Files needing tests first (by risk):**
| File | LOC | Risk | What to test |
|------|-----|------|-------------|
| service.rs | 609 | HIGH | CRUD ops, desktop file generation, shell_split(), parse_exec_line(), migrate_legacy |
| registry.rs | 138 | MED | match_url(), search(), category filtering |
| webapp.rs | 95 | LOW | Default values, serialization roundtrip |
| browser.rs | 120 | LOW | exec_for_url(), icon_name() mapping |
| favicon.rs | 258 | MED | extract_title(), extract_icon_urls(), resolve_url() (mock HTTP) |

**Recommended approach:**
- Unit tests for pure logic (shell_split, parse_exec_line, extract_title, resolve_url)
- Integration tests with temp dirs for service CRUD + desktop file I/O
- Mock reqwest for favicon fetch tests
- Target: 80%+ coverage on service.rs and registry.rs

### 1.2 ✅ ~~README Incorrect~~ → Fixed (Python → Rust)

**Current:** Says "Built with Python", lists python-bs4, python-requests, python-gobject as deps.  
**Reality:** 100% Rust with GTK4/libadwaita.

**Fix:** Rewrite Technical Details + Dependencies sections.

### 1.3 ✅ ~~Accessibility — Icon Buttons Without Names~~ → ~15 buttons labeled

~15 icon-only buttons across the app have NO `set_accessible_name()`. Orca reads them as generic "button".

| Location | Widget | Current Orca Output | Fix |
|----------|--------|-------------------|-----|
| manager/window.rs | search toggle | "toggle button" | `set_accessible_name("Search")` |
| manager/window.rs | add button | "button" | `set_accessible_name("Add WebApp")` |
| manager/window.rs | menu button | "menu button" | `set_accessible_name("Main Menu")` |
| webapp_dialog.rs | detect button | "button" | `set_accessible_name("Detect from website")` |
| webapp_dialog.rs | icon chooser | "button" | `set_accessible_name("Choose icon")` |
| webapp_row.rs | browser button | "button" | `set_accessible_name("Change browser")` |
| webapp_row.rs | edit button | "button" | `set_accessible_name("Edit")` |
| webapp_row.rs | delete button | "button" | `set_accessible_name("Delete")` |
| viewer/window.rs | back button | "button" | `set_accessible_name("Back")` |
| viewer/window.rs | forward button | "button" | `set_accessible_name("Forward")` |
| viewer/window.rs | reload button | "button" | `set_accessible_name("Reload")` |
| viewer/window.rs | fullscreen button | "button" | `set_accessible_name("Toggle fullscreen")` |

**Effort:** Low (1-2 lines each). **Impact:** Huge for screen reader users.

---

## 2. High Priority Issues — ✅ MOSTLY RESOLVED

### 2.1 ✅ ~~Clippy Warnings (33)~~ → 0 warnings

**Auto-fixable:** `cargo clippy --fix --allow-dirty`

| Warning | Count | Fix |
|---------|-------|-----|
| needless_borrows_for_generic_args | 30 | Remove `&` on `&gettext(...)` calls |
| redundant_import | 1 | Remove `use gdk4;` in viewer |
| let_and_return | 1 | Inline return value |
| field_assignment_outside_initializer | 1 | Move to struct init |

### 2.2 ✅ ~~Format Violations (74 diffs)~~ → 0 diffs

**Fix:** `cargo fmt`

### 2.3 ⚠️ Blocking I/O on Main Thread

These synchronous operations can freeze the UI:

| Location | Operation | Risk |
|----------|-----------|------|
| service.rs | `load_webapps()` — reads JSON from disk | Low (small file) |
| service.rs | `detect_browsers()` — spawns `flatpak list` + `xdg-settings` | **Medium** (process exec) |
| service.rs | `import_webapps()` — ZIP extraction | **High** (large archives) |
| service.rs | `export_webapps()` — ZIP creation | Medium |
| favicon.rs | `fetch_site_info()` — HTTP + image processing | **Already threaded** ✅ |
| desktop.rs | `fs::create_dir_all`, `fs::write` | Low |

**Recommended:** Move `import_webapps()` and `export_webapps()` to background threads with progress indication. `detect_browsers()` should be cached after first call.

### 2.4 ✅ ~~2 Unwrap Calls in Viewer~~ → .expect() with context

| Location | Line | Context | Fix |
|----------|------|---------|-----|
| viewer/window.rs | ~331 | `user_content_manager().unwrap()` | `.expect("WebView must have UCM")` |
| viewer/window.rs | ~463 | `application().unwrap()` | `.expect("Window must have app")` |

These are safe in practice (WebView always has UCM, Window always has app) but should use `.expect()` with context message for debuggability.

### 2.5 ✅ ~~ITP Disabled Without Documentation~~ → Commented

```rust
session.set_itp_enabled(false);
```

Disabling Intelligent Tracking Prevention allows all cross-site tracking. This was intentional (for cookie persistence in login-required webapps like YouTube/Spotify), but:
- Should have a code comment explaining WHY
- Consider making it per-webapp configurable in the future

### 2.6 ⚠️ Permission Auto-Grant (Viewer)

Camera, microphone, and notification permissions are auto-granted without user consent or logging. This is convenient but:
- Security risk for untrusted webapps
- No audit trail
- Consider: prompt once → remember decision per webapp

---

## 3. Medium Priority Issues

### 3.1 Accessibility — Live Regions

When the user types in search (manager/window.rs) or template gallery, the list is rebuilt silently. Orca does not announce changes.

**Fix:** After `populate_list()`, update an accessible live region with result count:
```rust
// After populating
status_label.set_label(&format!("{} webapps found", count));
// Ensure status_label has AccessibleRole::Status
```

### 3.2 Accessibility — Category Headers Without Semantic Role

Category section headers in window.rs and template_gallery.rs use visual styling (`title-4` CSS class) but no semantic `AccessibleRole::Heading`.

**Fix:**
```rust
header.set_accessible_role(gtk::AccessibleRole::Heading);
```

### 3.3 Accessibility — Switches Without State Announcement

App Mode switch (webapp_dialog.rs) and "Don't show again" switch (welcome_dialog.rs) don't clearly announce state.

**Better pattern:** Use `adw::SwitchRow` instead of manual `gtk::Switch` + `gtk::Label` in `gtk::Box`. AdwSwitchRow handles accessible properties automatically.

### 3.4 Accessibility — Welcome Dialog Markup

Uses Pango markup (`<span>`, `<b>`) in labels. Some Orca versions may read the markup tags literally.

**Fix:** Use `gtk::Label::set_attributes()` with explicit `pango::AttrList` instead of inline markup. Or verify behavior with Orca before changing.

### 3.5 UX — Cognitive Overload in Create/Edit Dialog

9+ simultaneous input fields violate the 7±2 cognitive load rule.

**Recommendation:**
- "Behavior" section (App Mode, Browser, Profile) should be collapsed by default for new webapps
- Only URL + Name + Icon visible initially (progressive disclosure)
- Power users expand Behavior manually

### 3.6 UX — Category Dropdown Unclear

Desktop categories ("Network", "AudioVideo", "Development") are not self-explanatory for non-technical users.

**Fix:** Add subtitle to category ActionRow: "Controls where the webapp appears in the application menu"

### 3.7 UX — Template Apply Not Reversible

Clicking a template in the gallery immediately overwrites form fields with no undo.

**Fix:** Consider "Apply Template?" confirmation dialog with preview, OR store pre-template state for undo.

### 3.8 UX — Browser Button Confusing in App Mode

webapp_row.rs shows a browser-icon button for every webapp, but in App mode it shows `application-x-executable-symbolic` icon. Users may not understand its purpose.

**Fix:** Hide or disable the browser button when `app_mode == App`.

### 3.9 Large Files Should Be Split

| File | LOC | Recommendation |
|------|-----|---------------|
| service.rs | 609 | Split: service/crud.rs, service/browser.rs, service/migration.rs, service/io.rs |
| viewer/window.rs | 657 | Extract: viewer/navigation.rs, viewer/downloads.rs, viewer/shortcuts.rs |
| manager/window.rs | 574 | Extract: window/actions.rs, window/list.rs |
| webapp_dialog.rs | 557 | OK (single dialog, hard to split meaningfully) |

### 3.10 Non-Idiomatic Patterns

| Location | Issue | Fix |
|----------|-------|-----|
| service.rs `shell_split()` | Custom tokenizer, no escaped-quote support | Document limitation or use `shell-words` crate |
| service.rs `parse_legacy_desktop()` | Manual loop + break | Use `.take_while()` iterator |
| registry.rs `categories()` | Collect → sort → return | Use `BTreeMap` or `itertools::sorted()` |
| webapp_row.rs `load_icon()` | Nested if-else fallback | Use `.or_else()` chain |
| manager/window.rs | `Rc::new(content_box.as_ref().clone())` | Simplify clone pattern |

### 3.11 Polling Pattern for Background Work

webapp_dialog.rs uses `glib::timeout_add_local(100ms)` to poll a `mpsc::channel` for favicon results. This is functional but wasteful.

**Better:** Use `glib::spawn_future_local()` with `async` channel (if glib-async available), or at minimum increase poll interval to 250ms.

---

## 4. Low Priority / Nice-to-Have

### 4.1 Fullscreen Reveal Not Discoverable

Viewer's fullscreen mode hides the header bar. Users must move mouse to top edge to reveal controls. This is not documented or hinted at.

**Fix:** On first fullscreen entry, show a transient toast: "Move mouse to top to show controls, or press Esc/F11 to exit"

### 4.2 Geometry Load Silent Failure

viewer/window.rs loads/saves window geometry from JSON but silently ignores parse errors.

**Fix:** Add `log::warn!()` on geometry parse failure.

### 4.3 Download UX

Viewer prompts for every download without remembering last directory.

**Future:** Remember last download directory in config.

### 4.4 Menu Button Discoverability

Main window's menu button (Import/Export/Browse/Remove All) uses only an icon with no label. New users may not discover these features.

**Options:**
- Add label "Menu" to button
- Move Import/Export to a visible toolbar section
- Add keyboard shortcut hints in menu items

### 4.5 CSS Hardcoded Values

style.rs uses hardcoded pixel values (6px, 12px, 56px) instead of Adwaita spacing scale. Not a functional issue but diverges from HIG consistency.

### 4.6 tokio Dependency Audit

tokio is pulled in (rt-multi-thread, macros) but appears only used for zbus async runtime. Verify if zbus can use glib async runtime instead to eliminate the tokio dependency and reduce binary size.

---

## 5. Security Review

| Check | Status | Notes |
|-------|--------|-------|
| Hardcoded secrets | ✅ None | |
| SQL injection | ✅ N/A | No SQL (SQLite is WebKit-managed) |
| Command injection | ✅ Safe | Uses `Command::new()` with list args, no `shell=True` |
| Path traversal (ZIP) | ✅ Validated | `canonicalize()` check in import |
| XSS | ✅ N/A | No HTML generation |
| SSRF | ⚠️ Low risk | favicon.rs fetches user-provided URLs — rate limited by UI, 10s timeout, 5MB limit |
| Permission model | ⚠️ | Auto-grants camera/mic/notification — see 2.6 |
| Cookie security | ✅ | SQLite persistent storage, file-system protected |
| ITP disabled | ⚠️ | Cross-site tracking allowed — see 2.5 |

**Overall:** No critical security vulnerabilities. Low-risk items documented above.

---

## 6. Architecture Overview

```
biglinux-webapps/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── webapps-core/           # shared library
│   │   └── src/
│   │       ├── config.rs       # APP_ID, version, paths
│   │       ├── desktop.rs      # .desktop file I/O
│   │       ├── i18n.rs         # gettext init
│   │       ├── models/         # WebApp, Browser, AppMode
│   │       └── templates/      # preset webapp templates (registry pattern)
│   │
│   ├── webapps-manager/        # GTK4 management UI (big-webapps-gui)
│   │   └── src/
│   │       ├── main.rs         # entry point
│   │       ├── window.rs       # main window (574L) ⚠️
│   │       ├── webapp_dialog.rs # create/edit (557L) ⚠️
│   │       ├── service.rs      # business logic (609L) ⚠️
│   │       ├── favicon.rs      # HTTP fetch + icon processing
│   │       ├── webapp_row.rs   # list item widget
│   │       ├── browser_dialog.rs
│   │       ├── template_gallery.rs
│   │       ├── welcome_dialog.rs
│   │       └── style.rs        # CSS
│   │
│   └── webapps-viewer/         # WebKitGTK browser (big-webapps-viewer)
│       └── src/
│           ├── main.rs         # CLI entry (clap)
│           └── window.rs       # browser window (657L) ⚠️
│
├── data/                       # icons, .desktop, metainfo
└── po/                         # translations
```

### Architecture Strengths
- ✅ Clean 3-crate separation (core/manager/viewer)
- ✅ Template registry pattern — extensible without code changes
- ✅ Desktop file generation isolated in core
- ✅ Proper i18n with gettext
- ✅ CSS centralized in style.rs

### Architecture Weaknesses
- ⚠️ No GObject subclassing — all widgets built imperatively with Rc<RefCell>
- ⚠️ service.rs mixes CRUD, browser detection, migration, import/export (SRP violation)
- ⚠️ 4 files >500 LOC — should be split for maintainability
- ⚠️ No async I/O pattern — blocking calls on UI thread (see 2.3)
- ⚠️ No state management layer — Rc<RefCell<AppState>> passed everywhere via clones

---

## 7. Action Plan

### Phase A: Quick Wins (< 1 day)

| # | Task | Files | Auto? |
|---|------|-------|-------|
| A1 | `cargo fmt` | all | ✅ Auto |
| A2 | `cargo clippy --fix` | all | ✅ Auto |
| A3 | Fix 2 unwraps → `.expect()` | viewer/window.rs | Manual |
| A4 | Add `set_accessible_name()` to ~15 icon buttons | 5 files | Manual |
| A5 | Add ITP comment explaining WHY disabled | viewer/window.rs | Manual |
| A6 | Fix README (Python → Rust) | README.md | Manual |

### Phase B: Core Quality (2-3 days)

| # | Task | Files | Notes |
|---|------|-------|-------|
| B1 | Write unit tests for `shell_split()`, `parse_exec_line()` | service.rs | Pure logic, easy tests |
| B2 | Write unit tests for `extract_title()`, `resolve_url()` | favicon.rs | Pure logic |
| B3 | Write unit tests for registry `match_url()`, `search()` | registry.rs | Pure logic |
| B4 | Write integration tests for service CRUD | service.rs | Temp dir needed |
| B5 | Install + run `cargo audit` | Cargo.toml | Check dep vulns |
| B6 | Add semantic heading roles to category headers | window.rs, template_gallery.rs | A11y |
| B7 | Add live region for search result feedback | window.rs | A11y |

### Phase C: UX Polish (1 week)

| # | Task | Files | Notes |
|---|------|-------|-------|
| C1 | Collapse "Behavior" section by default (new webapp) | webapp_dialog.rs | Cognitive load |
| C2 | Hide browser button in App mode rows | webapp_row.rs | Clarity |
| C3 | Add category explanation subtitle | webapp_dialog.rs | User guidance |
| C4 | Replace manual Switch+Label with AdwSwitchRow | webapp_dialog.rs, welcome_dialog.rs | A11y + HIG |
| C5 | Fullscreen first-use hint toast | viewer/window.rs | Discoverability |
| C6 | Validate URL in real-time (not just on Save) | webapp_dialog.rs | Error prevention |

### Phase D: Architecture (2+ weeks)

| # | Task | Files | Notes |
|---|------|-------|-------|
| D1 | Split service.rs into modules | service/ | SRP |
| D2 | Split viewer/window.rs | viewer/ | Maintainability |
| D3 | Move import/export to background threads | service.rs, window.rs | UI responsiveness |
| D4 | Cache browser detection results | service.rs | Avoid repeated process spawns |
| D5 | Evaluate removing tokio (use glib async for zbus) | Cargo.toml | Binary size |
| D6 | Add per-webapp permission preferences | viewer/window.rs | Security |

---

## 8. Baseline Metrics

```
Binary sizes (release, stripped):
  big-webapps-gui:    TBD (measure after build)
  big-webapps-viewer: TBD

Startup time: TBD (measure with `time big-webapps-gui`)

Clippy warnings: 33 → target 0
Format diffs:    74 → target 0
Test count:       0 → target 30+ (Phase B)
Test coverage:    0% → target 80% on service.rs, registry.rs
Unwrap calls:     2 → target 0
A11y issues:    ~20 → target 0 (Phase A+B+C)
```

---

## 9. Orca Screen Reader Test Checklist

```
[ ] Open app → Tab through header buttons → all announced with name
[ ] Type in search → result count announced via live region
[ ] Open Add dialog → all fields have accessible labels
[ ] Toggle App Mode switch → state change announced
[ ] Select template → action result announced
[ ] Open browser dialog → radio buttons announce browser name
[ ] Webapp list rows → each announces webapp name and available actions
[ ] Delete webapp → confirmation dialog is accessible
[ ] Open viewer → navigation buttons all announced
[ ] Ctrl+L in viewer → URL entry focus announced
[ ] F11 in viewer → fullscreen state announced
```

---

## 10. Notes

- **tokio:** Present in workspace deps but only used by zbus crate (D-Bus IPC). If zbus can run on glib main context, removing tokio would reduce compile time and binary size.
- **webkit6 accessibility:** WebKitGTK has inherent limitations for screen readers — web content inside the WebView is not fully accessible to Orca. This is an upstream limitation, not fixable in this project. Document in README.
- **Browser detection caching:** `detect_browsers()` spawns external processes (flatpak list, xdg-settings). Results should be cached for the app session since installed browsers don't change during a single run.
- **shell_split() limitations:** Custom tokenizer doesn't handle escaped quotes (`\"`) or heredoc-style strings. Document this or switch to `shell-words` crate.
- **ZIP path traversal:** Current `canonicalize()` check is correct but `canonicalize()` requires the path to exist. Consider using `Path::components()` check for `..` segments as additional defense.

---

## 11. Phase 2 Fixes Applied (2025-06-25)

All changes verified: cargo build ✅, cargo clippy 0 warnings ✅, 35/35 tests ✅

### Security
| ID | Fix | File | Details |
|----|-----|------|---------|
| SEC-02 | ✅ Zip bomb protection | service.rs | `Read::take(50MB)` limit per extracted file + cleanup on oversize |

### Code Quality
| ID | Fix | File | Details |
|----|-----|------|---------|
| QUALITY-02 | ✅ Signal handler leak | webapp_dialog.rs | `favicon_flow.connect_child_activated` moved out of detect handler → wired once |
| QUALITY-06 | ✅ Atomic JSON writes | service.rs | Write `.json.tmp` → `fs::rename()` → no corruption on crash |

### UX
| ID | Fix | File | Details |
|----|-----|------|---------|
| UX-01 | ✅ Validation feedback | webapp_dialog.rs | Error CSS class + `grab_focus()` on empty/invalid fields |
| UX-02 | ✅ Save error display | webapp_dialog.rs | `adw::Banner` error message, dialog stays open |
| UX-05 | ✅ Skip auto-detect on template | webapp_dialog.rs | `skip_auto_detect` Cell flag → no redundant favicon fetch |

### Polish
| ID | Fix | File | Details |
|----|-----|------|---------|
| POLISH-03 | ✅ Geometry save accuracy | viewer/window.rs | Use `window.width()/height()` for non-maximized, `default_size()` fallback for maximized |
| POLISH-05 | ✅ Cancel debounce on close | webapp_dialog.rs | `connect_destroy` cancels pending `SourceId` timer |
| POLISH-06 | ✅ Context menu action group leak | viewer/window.rs | Action group created once, action takes URI as `String` param → reused per right-click |

### Phase 3 — High+Medium Priority (2025-06-25)

| ID | Fix | File | Details |
|----|-----|------|---------|
| 2.6 | ✅ Permission prompt system | viewer/window.rs | Camera/mic/geolocation → `adw::AlertDialog` prompt, decision persisted in `permissions.json`, other perms auto-granted |
| 2.3 | ✅ Background I/O threads | manager/window.rs | Import/export run on `std::thread::spawn`, result polled via `glib::timeout_add_local` |
| 3.5 | ✅ Collapse Behavior section | webapp_dialog.rs | New webapps: Behavior group hidden, "Advanced Settings…" button reveals it |
| 3.9 | ✅ Split service.rs | service/ | 735L monolith → 4 modules: mod.rs(249), browser.rs(126), io.rs(130), migration.rs(255) |
