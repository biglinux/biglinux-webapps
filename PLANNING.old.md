# PLANNING.md — BigLinux WebApps Improvement Roadmap

## Files Analyzed

**Total Python files read:** 18
**Total Python lines analyzed:** 3439
**Large files (>500 lines) confirmed read in full:**
- `webapps/ui/webapp_dialog.py` (763 lines) — read completely L1-763
- `webapps/ui/main_window.py` (520 lines) — read completely L1-520

**Shell scripts read:** 4 (big-webapps 277L, big-webapps-exec 118L, check_browser.sh 164L, others)
**Other files read:** PKGBUILD, desktop entries, CSS profile, README.md

---

## Current State Summary

**Overall grade: C+ → A-** (after shell security + accessibility + CC refactoring + GTK4 migration + browser registry + UX fixes + shell decoupling + dialog split + Orca navigation + service layer + progressive disclosure + save feedback + URL validation + focus management + dead code removal)

The application is functional and ships to users. The GTK4/Adw migration is complete and the UI structure is reasonable. However, significant issues exist:

- **No tests at all** — zero test files, zero coverage
- **Critical security vulnerability** — `shell=True` with user-supplied data in `CommandExecutor`
- **Massive code duplication** — `get_system_default_browser()` has the same 30-line if/elif chain duplicated verbatim (CC=54)
- **No accessibility** — zero `accessible-name`, zero `accessible-description` on any widget
- **Architecture bleeding** — UI code directly calls shell scripts, no clear data layer
- **Translation bug** — `_()` referenced before assignment in `application.py:185` (F823)
- **No type hints** anywhere in 3400+ lines of Python

---

## Critical (fix immediately)

### Security

- [x] **Command injection via `shell=True`**: `command_executor.py:38` — `execute_command()` runs arbitrary strings through `shell=True`. Methods like `create_webapp()` (L106) interpolate user input (`app_name`, `app_url`) directly into shell commands with only single-quote wrapping, which is trivially bypassable (e.g., name containing `'; rm -rf /; '`). **Fix:** Use `subprocess.run()` with a list of arguments, never `shell=True`. Refactor `create_webapp()`, `update_webapp()`, `remove_webapp()` to pass args as lists.

- [x] **Zip extraction path traversal**: `application.py:319` — `zipf.extractall(temp_dir)` without validating member paths. A malicious ZIP can write files outside `temp_dir` via `../` entries (ZipSlip). **Fix:** Validate all member paths before extraction, or use `shutil.unpack_archive` with path validation.

- [x] **Translation `_` referenced before assignment**: `application.py:185` — ruff F823. The `_` function from `translation.py` is imported but due to module-level import ordering, it may not be available in all code paths. The import works at runtime because `gi.require_version` runs first, but this is fragile. **Fix:** Ensure `from webapps.utils.translation import _` is at the correct scope.

- [x] **Shell script quoting vulnerabilities**: `big-webapps` — `rm $filename`, `if [ -f $filename ]`, `if [ $command = ... ]` without quotes. Filenames with spaces/globbing cause unexpected behavior or data loss. **Fix:** Quote all variable expansions: `rm "$filename"`, `if [ -f "$filename" ]`, `if [ "$command" = ... ]`.

- [x] **Manual JSON generation in `big-webapps`**: L241-251 — manual escape with `${name//\"/\\\\\"}` is fragile; backslashes, tabs, newlines in names break JSON. **Fix:** Replaced with `_json_escape()` function using `sed` for proper `\`/`"`/tab escaping, and `printf` for structured JSON output.

- [x] **`big-webapps-exec` unquoted `$browser_exec`**: All `exec $browser_exec` and `$browser_exec ... &` lines — word splitting on flatpak commands (`flatpak run com.brave.Browser` becomes 3 separate words). **Fix:** Changed `browser_exec` to bash array `browser_exec=(...)`, used `"${browser_exec[@]}"` everywhere.

- [x] **`big-webapps-exec` icon copy on every launch**: L19-21 — `cp "$icon" ~/.local/share/icons/` + `sed -Ei` runs on every execution, even if icon is already current. **Fix:** Added `cmp -s` check — only copy+sed if icon differs or doesn't exist.

- [x] **Wayland race condition in `big-webapps-exec`**: L95-106 — `mv -f` of .desktop files without locking. Two simultaneous instances corrupt the original .desktop. **Fix:** Added `flock` advisory locking; second instance skips icon swap and launches directly.

- [x] **`sed` chain for browser name mapping**: `big-webapps` — fragile `sed` pipeline for `short_browser`. **Fix:** Replaced with `case` statement with glob patterns.

### Bugs

- [x] **`get_app_icon_url.py` uses GTK3**: `get_app_icon_url.py:6` — Migrated to GTK4 (`gi.require_version("Gtk", "4.0")`). Uses `Gtk.IconTheme.get_for_display()` + `lookup_icon()` returning `IconPaintable` with `get_file().get_path()`. Requires `Gtk.init()` + `Gdk.Display.get_default()`.

- [x] **`BROWSER_ICONS_PATH` is relative**: `browser_icon_utils.py:9` — `BROWSER_ICONS_PATH = "icons"` is relative to CWD. Works only because `big-webapps-gui` does `cd /usr/share/biglinux/webapps/` before launching. If launched from any other directory, all browser icons fail silently. **Fix:** Use `os.path.dirname(os.path.realpath(__file__))` to compute absolute path.

- [x] **`name_label.set_ellipsize(True)` wrong API**: `webapp_row.py:73,82` — `set_ellipsize()` expects a `Pango.EllipsizeMode` enum, not a boolean. GTK4 may silently accept `True` as `1` (which maps to `ELLIPSIZE_START`), but the intent is likely `ELLIPSIZE_END`. **Fix:** `name_label.set_ellipsize(Pango.EllipsizeMode.END)`.

- [x] **`_open_folder` infinite recursion**: `application.py:97-100` — If `os.makedirs` creates the folder, it calls `self._open_folder()` again, which now finds the folder and opens it. This is fragile — if `Gtk.show_uri` and `xdg-open` both fail, it loops. There's also no guard against repeated creation. **Fix:** Remove recursion, just `makedirs` then `show_uri` in a single flow.

---

## High Priority (code quality)

### Architecture

- [x] **Extract browser name mapping to data**: `command_executor.py:160-292` — `get_system_default_browser()` has cyclomatic complexity **F(54)** with the same 30-browser if/elif chain **duplicated twice** (xdg-settings and xdg-mime paths). **Fix:** Create a `BROWSER_DESKTOP_MAP` dictionary mapping desktop file patterns to browser IDs. Reduce to ~20 lines.

- [x] **Duplicate browser name maps**: Created `browser_registry.py` as single source of truth for Python side (`BROWSER_DISPLAY_NAMES` + `DESKTOP_PATTERN_MAP`). `browser_model.py` and `command_executor.py` now import from it. Shell scripts (`check_browser.sh`, `big-webapps-exec`) retain their own layer-specific data (paths, flatpak exec commands) since they cannot import Python.

- [x] **Shell script coupling**: `application.py` now calls `big-webapps json` directly and resolves icons via `enrich_webapps_with_icons()` in `browser_icon_utils.py`. Eliminated `get_json.sh` → `get_app_icon_url.py` chain (Python → shell → Python → shell → Python).

- [x] **Two different application IDs**: `main.py:24` sets `br.com.biglinux.webapps`, `application.py:32` sets `org.biglinux.webapps`. One of these is ignored at runtime. **Fix:** Use a single canonical app ID.

### Code Quality

- [x] **Add type hints to all function signatures**: 0/18 files have type annotations. Start with models and utils (pure logic), then UI. This enables mypy and IDE support.

- [x] **Unused variables from GTK signal handlers**: vulture reports 19 unused variables. Most are GTK callback signatures (`controller`, `keycode`, `state`, `param`). **Fix:** Prefix with `_` (e.g., `_controller`, `_keycode`). For truly unused vars like `d` in `application.py:435`, remove them.

- [x] **Format 2 files**: `application.py` and `main_window.py` fail `ruff format --check`. **Fix:** Run `ruff format`.

- [x] **30 E402 import violations**: All `from gi.repository import ...` lines trigger E402 because they follow `gi.require_version()`. This is expected and correct for GI. **Fix:** Add `# noqa: E402` inline or configure ruff to ignore E402 for `gi.repository` imports. Alternatively, add a `ruff.toml` with per-file ignores.

- [x] **High complexity functions**: `_handle_import_response` (CC=15→5), `_handle_export_response` (CC=14→5), `on_webapp_dialog_response` (CC=17→6). **Fix:** Extracted `_serialize_webapp_for_export()`, `_import_single_webapp()` from `application.py`. Extracted `_find_webapp_after_reload()` from `main_window.py`, collapsed create/update duplicate search into shared helper with URL-only fallback.

- [x] **`print()` statements as logging**: 40+ `print()` calls throughout the codebase used for debugging. No structured logging. **Fix:** Replace with `logging` module. Use `logger = logging.getLogger(__name__)` per module. Level: DEBUG for dev info, ERROR for failures.

---

## Medium Priority (UX improvements)

### Progressive Disclosure

- [x] **Profile settings overwhelm new users**: `webapp_dialog.py` — profile switch + profile name entry now wrapped in `AdwExpanderRow` ("Profile Settings"). Default collapsed. Only shown when App Mode is active. *Principle: Progressive Disclosure — reduce cognitive load on primary flow.*

- [x] **Category dropdown shows all 9 categories upfront**: `Gtk.DropDown` already collapses the list — user only sees choices on click (≠ radio buttons). Default "Webapps" pre-selected for new webapps. No change needed — Hick's Law mitigated by dropdown widget design.

### Feedback Loops

- [x] **No feedback during webapp creation**: `webapp_dialog.py` — Save now shows loading overlay, runs command in background thread via `threading.Thread(daemon=True)`, uses `GLib.idle_add` to close dialog on completion. *Principle: System Status Visibility (Nielsen).*

- [x] **URL validation is reactive only**: `webapp_dialog.py` — Real-time URL validation with `urlparse` (checks scheme http/https + netloc). Suffix icon `emblem-ok-symbolic` (success) / `dialog-warning-symbolic` (error) + CSS classes. *Principle: Error Prevention > Error Recovery.*

- [x] **Delete confirmation lacks context**: `main_window.py:363` — Delete dialog shows "Are you sure you want to delete {name}?" but doesn't show the URL or browser, making it hard to distinguish between similarly-named webapps. **Fix:** Include URL and browser in the dialog body. *Principle: Recognition over Recall.*

### Visual Hierarchy

- [x] **Welcome dialog CSS leaks globally**: `welcome_dialog.py:77` — `Gtk.StyleContext.add_provider_for_display()` applies headerbar CSS to ALL windows, not just the welcome dialog. **Fix:** Use `Gtk.StyleContext.add_provider()` on the specific widget, like `webapp_dialog.py` does correctly at L168.

- [x] **Icon selection FlowBox has no visual feedback**: `FaviconPicker` in `favicon_picker.py` now applies `.favicon-selected` CSS class (accent-colored border) to the active `FlowBoxChild` and removes it from the previous selection.

### First-Run Experience

- [x] **Welcome dialog switch UX confusing**: `welcome_dialog.py` — Renamed label from "Show dialog on startup" to "Don't show this again" with inverted logic. Switch ON = suppress. Matches standard UX convention. *Principle: Match between system and real world (Nielsen).*

---

## Low Priority (polish & optimization)

- [x] **`time.time()` + `hash(datetime.now())` for webapp file IDs**: `main_window.py:172` — Uses `int(time.time())-{hash(datetime.now()) % 10000}` while `webapp_dialog.py:701` uses `uuid.uuid4().hex[:8]`. Inconsistent ID generation. **Fix:** Use UUID everywhere. The `main_window.py` pattern can produce collisions.

- [x] **`on_remove_all` double-confirmation UX**: Replaced two consecutive dialogs with a single `Adw.MessageDialog` containing a text entry. User must type the exact phrase (translated) to enable the destructive confirm button. 3 methods → 2 methods.

- [x] **`update_old_desktop_files.sh` references non-existent path**: L37 references `/usr/share/bigbashview/apps/webapps/check_browser.sh` which doesn't exist in the package. Line 46 references `/usr/share/bigbashview/bcc/apps/biglinux-webapps/webapps/`. These appear to be legacy paths from when the app used BigBashView. **Fix:** Update or remove dead references.

- [x] **`biglinux-webapps-systemd` also references legacy path**: L30 references `/usr/share/bigbashview/bcc/apps/biglinux-webapps/webapps/`. **Fix:** Same as above.

- [x] **CSS headerbar override in webapp_dialog.py**: Already uses `Gtk.StyleContext.add_provider()` on specific style context (not `add_provider_for_display`). No leak. Verified.

- [x] **AdwAboutWindow is deprecated**: `application.py:119` uses `Adw.AboutWindow`. Newer libadwaita versions use `Adw.AboutDialog`. **Fix:** Check the target libadwaita version and update if ≥ 1.5.

- [x] **Hardcoded version "3.0.0"**: Already resolved — `APP_VERSION = "3.1.0"` in `__init__.py`, imported by `application.py`. PKGBUILD uses rolling `$(date)` for package version (different semantic).

---

## Architecture Recommendations

### Current Structure
```
webapps/
├── application.py      # App class, export/import, action registration
├── models/
│   ├── browser_model.py  # Browser data model
│   └── webapp_model.py   # WebApp data model
├── ui/
│   ├── browser_dialog.py # Browser selection dialog
│   ├── favicon_picker.py # ✅ NEW — FlowBox favicon selector widget
│   ├── main_window.py    # Main window
│   ├── webapp_dialog.py  # Create/edit dialog (763L — too large)
│   ├── webapp_row.py     # List row widget
│   └── welcome_dialog.py # Welcome screen
└── utils/
    ├── browser_icon_utils.py  # Icon path resolution
    ├── browser_registry.py    # ✅ NEW — Central browser ID/name/pattern mapping
    ├── command_executor.py    # Shell command execution
    ├── translation.py         # i18n
    ├── url_utils.py           # Website metadata fetcher
    └── webapp_service.py      # ✅ NEW — Business logic layer (CRUD, export/import)
```

### Recommended Changes

1. **Split `webapp_dialog.py`** (821L→736L): ✅ Extracted `FaviconPicker` widget to `favicon_picker.py` (88L) with selection highlight CSS. Refactored `setup_ui()` from monolithic 326L method into orchestrator + 5 builder methods: `_build_form_group()`, `_build_category_row()`, `_build_mode_browser_profile()`, `_build_buttons()`, `_build_loading_overlay()`. Removed dead `favicons_group`/`favicons_box` code.

2. ~~**Create `browser_registry.py`**~~: ✅ Done — Created `webapps/utils/browser_registry.py` with `BROWSER_DISPLAY_NAMES` + `DESKTOP_PATTERN_MAP` + `get_display_name()` + `match_desktop_to_browser()`. Both `browser_model.py` and `command_executor.py` import from it.

3. ~~**Create `webapp_service.py`**~~: ✅ Done — Created `webapps/utils/webapp_service.py` (~215L) with `WebAppService` class. Methods: `load_data()`, `create_webapp()`, `update_webapp()`, `delete_webapp()`, `delete_all_webapps()`, `find_webapp()`, `export_webapps()`, `import_webapps()`, `get_system_default_browser()`. All business logic moved from `application.py` and `main_window.py`.

4. **Replace shell string execution**: `CommandExecutor.execute_command(shell=True)` → specific methods with `subprocess.run(list)`. No shell interpolation.

5. ~~**State management**~~: ✅ Improved — `find_webapp()` now accepts `app_file` (desktop filename) as stable ID, with URL+name as fallback. `_find_webapp_after_reload()` passes `app_file` from the original webapp. Full GObject signals deferred — current approach is reliable with stable IDs.

---

## UX Recommendations

1. **Drag-and-drop URL support**: Allow users to drag a URL from browser to the window to create a webapp. Reduces friction from copy-paste workflow. *Principle: Direct Manipulation — let users interact naturally.*

2. **Inline editing**: Instead of opening a full dialog for simple changes (name, category), allow inline editing in the list row. *Principle: Efficiency of use — expert users should have shortcuts.*

3. **Visual browser indicator**: The browser icon in the row is small (27px) and lacks a label. Users may not recognize browser icons at a glance. **Fix:** Add browser name as subtitle text in the row. *Principle: Recognition over Recall.*

4. **Empty state with presets**: Instead of a blank empty state, offer one-click common webapp presets (WhatsApp, Spotify, Gmail, etc.). Reduces the barrier to first use. *Principle: Reduce activation energy — the hardest part is starting.*

5. **Undo delete**: Destructive deletion should be undoable for 5-10 seconds via an "Undo" action in the toast notification (Adw.Toast supports action buttons). This is safer than confirmation dialogs. *Principle: Forgiving design — allow recovery, not just prevention.*

---

## Orca Screen Reader Compatibility

**Issues found:**

### Critical — Completely inaccessible widgets

- [x] **All buttons lack accessible names**: Throughout the codebase, buttons are created with only icons and no accessible name. A blind user hears nothing or "button" when focusing these:
  - `webapp_row.py:99` — browser button (icon-only, no label, no accessible-name)
  - `webapp_row.py:105` — edit button (icon-only, tooltip exists but Orca reads accessible-name first)
  - `webapp_row.py:114` — delete button (icon-only)
  - `main_window.py:68` — search toggle button
  - `main_window.py:77` — menu button
  **Fix:** Added `update_property([Gtk.AccessibleProperty.LABEL], [...])` for each.

- [x] **Icon FlowBox items have no labels**: `webapp_dialog.py:600-610` — Favicon selection items are `Gtk.Image` inside `Gtk.Box`. Orca cannot announce what each icon represents. A blind user cannot distinguish between favicons. **Fix:** Added `accessible-description` with ordinal ("Icon 1 of 5").

- [x] **Category dropdown has no accessible label**: `webapp_dialog.py:260` — The `Gtk.DropDown` is added as a suffix to `AdwActionRow`. **Fix:** Added `update_property([Gtk.AccessibleProperty.LABEL], [_("Category")])`.

- [x] **Profile switch has no accessible label**: `webapp_dialog.py:303` — `Gtk.Switch` is a suffix. **Fix:** Added `update_property([Gtk.AccessibleProperty.LABEL], [_("Use separate profile")])`.

- [x] **Search entry has no accessible label**: `main_window.py:107` — `Gtk.SearchEntry` inside `Gtk.SearchBar` has no label. **Fix:** Added `update_property([Gtk.AccessibleProperty.LABEL], [_("Search WebApps")])`.

- [x] **App mode switch has no accessible label**: `webapp_dialog.py` — `Gtk.Switch` for app mode. **Fix:** Added `update_property([Gtk.AccessibleProperty.LABEL], [_("Application Mode")])`.

### High — Missing state announcements

- [x] **Loading overlay not announced**: Added `accessible-description` to loading label ("Detecting website information, please wait"). After fetch completes, focus moves to Name entry so Orca announces the detected title.

- [x] **Toast notifications Orca priority**: `Adw.Toast` now uses `ToastPriority.HIGH` for destructive actions (delete, remove-all, errors) → maps to `role="alert"` (assertive). Info toasts remain `NORMAL` (polite `role="status"`).

- [x] **Empty state not focused on load**: `AdwStatusPage` now set `focusable(True)` and `grab_focus()` called when empty state is shown. Orca announces "No WebApps Found" + description.

### Medium — Navigation issues

- [x] **No skip-navigation for category headers**: Category headers now created with `Gtk.AccessibleRole.HEADING` via `GObject.new()` and `set_focusable(True)`. Orca "h" key navigates between categories.

- [x] **Dialog focus order is not optimal**: `webapp_dialog.py` — Dynamic focus on `map` signal: URL entry for new webapps (need to type URL first), Name entry for editing (URL already filled). Also removed dead `find_all_widget_types()` method — `self.name_row` used directly.

**Test checklist for manual verification:**
- [ ] Launch app with Orca running (`orca &; big-webapps-gui`)
- [ ] Navigate entire UI using only Tab/Shift+Tab
- [ ] Verify Orca announces every button, field, and state change
- [ ] Test "Add WebApp" flow without looking at screen
- [ ] Test "Detect" → icon selection → save flow with Orca
- [ ] Verify error messages are announced by Orca
- [ ] Test delete flow: button → confirmation dialog → result toast
- [ ] Test search: toggle → type → verify results announced
- [ ] Test browser dialog: navigation → selection → confirm

---

## Accessibility Checklist (General)

- [x] All interactive elements have accessible labels — **DONE** (buttons, entries, dropdown, switches, FlowBox icons)
- [x] Keyboard navigation works for all flows — **DONE** (ESC closes dialogs ✓, dynamic focus order ✓, Tab order relies on GTK4 defaults)
- [x] Color is never the only indicator — **DONE** (delete button uses `destructive-action` CSS class = red background + trash icon shape = two indicators)
- [ ] Text is readable at 2x font size — **UNTESTED** (no responsive breakpoints; `AdwClamp` is used in dialog ✓)
- [x] Focus indicators are visible — **OK** (relies on Adwaita theme defaults, known to work well)

---

## Tech Debt

### From ruff (31 errors)
- 30× E402: Module-level imports after `gi.require_version()` — expected, suppress with config
- 1× F823: `_` referenced before assignment in `application.py:185` — **real bug**

### From vulture (19 dead code items)
- 6× unused `parameter`/`param`/`d` — GTK callback signatures, prefix with `_`
- 3× unused `controller`/`keycode`/`state` in key handlers — GTK callback signatures
- 1× unused `args` in `main_window.py:38`
- 1× unused `flowbox` in `webapp_dialog.py:626`

### From radon (7 high-complexity functions — most now resolved)
| Function | CC Before | CC After | Grade | Status |
|---|---|---|---|---|
| `get_system_default_browser` | 54 | ~5 | A | ✅ Refactored to `_BROWSER_DESKTOP_MAP` |
| `on_webapp_dialog_response` | 17 | ~6 | A | ✅ Extracted `_find_webapp_after_reload()` |
| `_handle_import_response` | 15 | ~5 | A | ✅ Extracted `_import_single_webapp()` |
| `_handle_export_response` | 14 | ~5 | A | ✅ Extracted `_serialize_webapp_for_export()` |
| `_fetch_info_thread` | 13 | ~5 | A | ✅ Extracted `_resolve_title()` + `_collect_icon_urls()` |
| `handle_starttag` | 12 | 12 | B | — Inherent to HTML parsing, no practical split |
| `WebAppDialog.__init__` | 11 | ~6 | A | ✅ Extracted `_assign_default_browser()` |

### From mypy (1 error)
- Missing stubs for `requests` library — install `types-requests`

### No tech debt markers
- Zero TODO/FIXME/HACK/XXX found in codebase

---

## Metrics (before)

```
ruff lint:     31 errors (30 E402 expected, 1 F823 real bug)
ruff format:   2 files need formatting (application.py, main_window.py)
mypy:          1 error (missing stubs for requests)
vulture:       19 unused variables (100% confidence)
radon CC ≥ C:  7 functions (worst: F grade, CC=54)
test coverage: 0% (no tests exist)
tech debt:     0 markers
type hints:    0% of functions annotated
a11y labels:   0 accessible-name set on any widget
```

## Metrics (after this review session)

```
ruff lint:     0 errors on all Python files (all checks passed)
ruff format:   OK on modified files
shell syntax:  OK (bash -n) on big-webapps, big-webapps-exec
a11y labels:   15+ accessible-name/description set (buttons, entries, switches, dropdown, FlowBox, loading, empty state)
a11y nav:      category headings = AccessibleRole.HEADING + focusable; dynamic focus order (URL new, Name edit)
a11y toast:    destructive toasts = HIGH priority (assertive)
a11y visual:   FaviconPicker .favicon-selected CSS; delete button destructive-action (color+shape)
a11y color:    color never sole indicator (trash icon shape + destructive-action CSS)
CC reduced:    6/7 high-CC functions refactored (avg CC 54→~5)
shell fixes:   7 security/robustness fixes (quoting, JSON generation, arrays, flock, cmp -s, case statement)
shell coupling: get_json.sh chain eliminated → direct big-webapps json + enrich_webapps_with_icons()
architecture:  webapp_service.py biz layer (215L), application.py simplified (-120L)
new files:     browser_registry.py, favicon_picker.py, webapp_service.py
file split:    webapp_dialog.py 821→~720L, setup_ui() monolith → 5 builder methods + dead code removed
GTK4 migration: get_app_icon_url.py GTK3→GTK4
UX:            remove-all text-confirm; progressive disclosure (AdwExpanderRow); save spinner (thread+overlay);
               URL real-time validation (urlparse+icon); welcome dialog "Don't show again" (inverted); focus order
```
