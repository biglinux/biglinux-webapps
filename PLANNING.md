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

**Overall grade: C+**

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

### Bugs

- [ ] **`get_app_icon_url.py` uses GTK3**: `get_app_icon_url.py:6` — `gi.require_version("Gtk", "3.0")` while the rest of the app uses GTK4. Cannot coexist in the same process. Currently called via shell (`get_json.sh`), so it works, but is a maintenance hazard. The `lookup_icon` API differs between GTK3 and GTK4.

- [x] **`BROWSER_ICONS_PATH` is relative**: `browser_icon_utils.py:9` — `BROWSER_ICONS_PATH = "icons"` is relative to CWD. Works only because `big-webapps-gui` does `cd /usr/share/biglinux/webapps/` before launching. If launched from any other directory, all browser icons fail silently. **Fix:** Use `os.path.dirname(os.path.realpath(__file__))` to compute absolute path.

- [x] **`name_label.set_ellipsize(True)` wrong API**: `webapp_row.py:73,82` — `set_ellipsize()` expects a `Pango.EllipsizeMode` enum, not a boolean. GTK4 may silently accept `True` as `1` (which maps to `ELLIPSIZE_START`), but the intent is likely `ELLIPSIZE_END`. **Fix:** `name_label.set_ellipsize(Pango.EllipsizeMode.END)`.

- [x] **`_open_folder` infinite recursion**: `application.py:97-100` — If `os.makedirs` creates the folder, it calls `self._open_folder()` again, which now finds the folder and opens it. This is fragile — if `Gtk.show_uri` and `xdg-open` both fail, it loops. There's also no guard against repeated creation. **Fix:** Remove recursion, just `makedirs` then `show_uri` in a single flow.

---

## High Priority (code quality)

### Architecture

- [x] **Extract browser name mapping to data**: `command_executor.py:160-292` — `get_system_default_browser()` has cyclomatic complexity **F(54)** with the same 30-browser if/elif chain **duplicated twice** (xdg-settings and xdg-mime paths). **Fix:** Create a `BROWSER_DESKTOP_MAP` dictionary mapping desktop file patterns to browser IDs. Reduce to ~20 lines.

- [ ] **Duplicate browser name maps**: `browser_model.py:49-74` has a `browser_name_map` dict, `check_browser.sh` has `browsers`, `command_executor.py` has two identical chains, `big-webapps-exec` has a case statement. That's **5 separate places** mapping browser IDs. **Fix:** Single source of truth — either a JSON file or a Python dict imported everywhere. Shell scripts can read the JSON.

- [ ] **Shell script coupling**: `application.py` calls `./get_json.sh`, `./check_browser.sh`, and `command_executor.py` calls `big-webapps create` with shell string interpolation. The Python app is tightly coupled to shell scripts' CWD and output format. **Fix:** Phase out shell wrappers progressively — `get_json.sh` just calls `big-webapps json` through `get_app_icon_url.py`, this entire chain can be a Python function.

- [x] **Two different application IDs**: `main.py:24` sets `br.com.biglinux.webapps`, `application.py:32` sets `org.biglinux.webapps`. One of these is ignored at runtime. **Fix:** Use a single canonical app ID.

### Code Quality

- [x] **Add type hints to all function signatures**: 0/18 files have type annotations. Start with models and utils (pure logic), then UI. This enables mypy and IDE support.

- [x] **Unused variables from GTK signal handlers**: vulture reports 19 unused variables. Most are GTK callback signatures (`controller`, `keycode`, `state`, `param`). **Fix:** Prefix with `_` (e.g., `_controller`, `_keycode`). For truly unused vars like `d` in `application.py:435`, remove them.

- [x] **Format 2 files**: `application.py` and `main_window.py` fail `ruff format --check`. **Fix:** Run `ruff format`.

- [x] **30 E402 import violations**: All `from gi.repository import ...` lines trigger E402 because they follow `gi.require_version()`. This is expected and correct for GI. **Fix:** Add `# noqa: E402` inline or configure ruff to ignore E402 for `gi.repository` imports. Alternatively, add a `ruff.toml` with per-file ignores.

- [ ] **High complexity functions**: `_handle_import_response` (CC=15), `_handle_export_response` (CC=14), `on_webapp_dialog_response` (CC=17), `WebAppDialog.__init__` (CC=11), `WebsiteMetadataParser.handle_starttag` (CC=12), `_fetch_info_thread` (CC=13). **Fix:** Extract sub-functions. E.g., `_handle_import_response` can call `_process_single_import()` and `_show_import_result()`.

- [x] **`print()` statements as logging**: 40+ `print()` calls throughout the codebase used for debugging. No structured logging. **Fix:** Replace with `logging` module. Use `logger = logging.getLogger(__name__)` per module. Level: DEBUG for dev info, ERROR for failures.

---

## Medium Priority (UX improvements)

### Progressive Disclosure

- [ ] **Profile settings overwhelm new users**: `webapp_dialog.py` shows profile switch + profile name entry immediately. Most users won't need custom profiles. **Fix:** Show profile options only in an "Advanced" expander (`Gtk.Expander` or `AdwExpanderRow`). Default to "Browser" profile silently. *Principle: Progressive Disclosure — reduce cognitive load on primary flow.*

- [ ] **Category dropdown shows all 9 categories upfront**: Most users only need "Webapps". **Fix:** Default to "Webapps", move others to an "Advanced" section or use a searchable combo. *Principle: Hick's Law — fewer choices = faster decisions.*

### Feedback Loops

- [ ] **No feedback during webapp creation**: When user clicks "Save", there's no visual indication that the shell command is running. If `big-webapps create` takes time, the dialog just closes. **Fix:** Show a spinner or progress indicator during save, similar to the "Detect" loading overlay already implemented. *Principle: System Status Visibility (Nielsen).*

- [ ] **URL validation is reactive only**: User can type anything and only discovers issues when "Detect" fails or save produces a broken webapp. **Fix:** Add real-time URL validation with visual feedback (green check or red X suffix icon). Validate scheme, domain format. *Principle: Error Prevention > Error Recovery.*

- [x] **Delete confirmation lacks context**: `main_window.py:363` — Delete dialog shows "Are you sure you want to delete {name}?" but doesn't show the URL or browser, making it hard to distinguish between similarly-named webapps. **Fix:** Include URL and browser in the dialog body. *Principle: Recognition over Recall.*

### Visual Hierarchy

- [x] **Welcome dialog CSS leaks globally**: `welcome_dialog.py:77` — `Gtk.StyleContext.add_provider_for_display()` applies headerbar CSS to ALL windows, not just the welcome dialog. **Fix:** Use `Gtk.StyleContext.add_provider()` on the specific widget, like `webapp_dialog.py` does correctly at L168.

- [ ] **Icon selection FlowBox has no visual feedback**: When user selects a favicon in the detected icons, there's no selected state indicator (border, background). The FlowBox selection mode is set but no CSS highlights the selected child. **Fix:** Add CSS class or use `AdwActionRow` with radio-button-style selection.

### First-Run Experience

- [ ] **Welcome dialog is dismissible permanently with one click**: `welcome_dialog.py` — The "Show dialog on startup" switch defaults to ON, but once user unchecks it, there's no way to re-enable except manually editing `~/.config/biglinux-webapps/welcome_shown.json`. The "Show Welcome Screen" menu item always opens the dialog but doesn't re-enable the checkbox. **Fix:** Clarify the UX — either the menu item always shows it (current behavior is fine) and the switch controls auto-show-on-startup (which is also correct). Actually this works, but the switch label is confusing — it says "Show dialog on startup" but the switch state is inverted from what a user expects. If switch is ON, dialog shows. This is correct but may confuse users who expect "Don't show again" pattern. *Principle: Match between system and real world (Nielsen).* Consider renaming to "Don't show this again" with inverted logic.

---

## Low Priority (polish & optimization)

- [x] **`time.time()` + `hash(datetime.now())` for webapp file IDs**: `main_window.py:172` — Uses `int(time.time())-{hash(datetime.now()) % 10000}` while `webapp_dialog.py:701` uses `uuid.uuid4().hex[:8]`. Inconsistent ID generation. **Fix:** Use UUID everywhere. The `main_window.py` pattern can produce collisions.

- [ ] **`on_remove_all` double-confirmation UX**: Two consecutive dialogs is annoying. **Fix:** Single dialog with a text confirmation field (type "REMOVE ALL" to confirm). *Principle: Proportional friction — match effort to consequence.*

- [x] **`update_old_desktop_files.sh` references non-existent path**: L37 references `/usr/share/bigbashview/apps/webapps/check_browser.sh` which doesn't exist in the package. Line 46 references `/usr/share/bigbashview/bcc/apps/biglinux-webapps/webapps/`. These appear to be legacy paths from when the app used BigBashView. **Fix:** Update or remove dead references.

- [x] **`biglinux-webapps-systemd` also references legacy path**: L30 references `/usr/share/bigbashview/bcc/apps/biglinux-webapps/webapps/`. **Fix:** Same as above.

- [ ] **CSS headerbar override in webapp_dialog.py**: `webapp_dialog.py:157-162` — Creates a CSS provider to reduce headerbar padding. This should use the app-level stylesheet, not inline CSS per dialog.

- [x] **AdwAboutWindow is deprecated**: `application.py:119` uses `Adw.AboutWindow`. Newer libadwaita versions use `Adw.AboutDialog`. **Fix:** Check the target libadwaita version and update if ≥ 1.5.

- [ ] **Hardcoded version "3.0.0"**: `application.py:123` — No single source of truth for version. PKGBUILD uses `$(date)`, desktop file has no version, Python has "3.0.0". **Fix:** Single version source, read from a VERSION file or `pyproject.toml`.

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
│   ├── main_window.py    # Main window
│   ├── webapp_dialog.py  # Create/edit dialog (763L — too large)
│   ├── webapp_row.py     # List row widget
│   └── welcome_dialog.py # Welcome screen
└── utils/
    ├── browser_icon_utils.py  # Icon path resolution
    ├── command_executor.py    # Shell command execution
    ├── translation.py         # i18n
    └── url_utils.py           # Website metadata fetcher
```

### Recommended Changes

1. **Split `webapp_dialog.py`** (763L): Extract favicon detection UI into `favicon_picker.py`, icon selection into `icon_chooser.py`. Dialog itself should only handle form fields and validation. Target: <300L per file.

2. **Create `browser_registry.py`**: Single source of truth for browser ID ↔ name ↔ path ↔ desktop file mapping. Replace 5 duplicate maps. Load from JSON if needed by shell scripts too.

3. **Create `webapp_service.py`**: Business logic layer between UI and shell commands. Methods: `create_webapp()`, `update_webapp()`, `delete_webapp()`, `export_collection()`, `import_collection()`. Move all business logic from `application.py` and `main_window.py` here.

4. **Replace shell string execution**: `CommandExecutor.execute_command(shell=True)` → specific methods with `subprocess.run(list)`. No shell interpolation.

5. **State management**: `main_window.py` currently does `self.app.load_data()` after every operation, then manually searches for the updated webapp by URL+name. This is fragile. **Fix:** `WebAppCollection` should use stable IDs and emit GObject signals on changes.

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

- [ ] **All buttons lack accessible names**: Throughout the codebase, buttons are created with only icons and no accessible name. A blind user hears nothing or "button" when focusing these:
  - `webapp_row.py:99` — browser button (icon-only, no label, no accessible-name)
  - `webapp_row.py:105` — edit button (icon-only, tooltip exists but Orca reads accessible-name first)
  - `webapp_row.py:114` — delete button (icon-only)
  - `main_window.py:68` — search toggle button
  - `main_window.py:77` — menu button
  **Fix:** Add `button.set_accessible_name(_("Edit WebApp"))` or use `button.set_label()` for each.

- [ ] **Icon FlowBox items have no labels**: `webapp_dialog.py:600-610` — Favicon selection items are `Gtk.Image` inside `Gtk.Box`. Orca cannot announce what each icon represents. A blind user cannot distinguish between favicons. **Fix:** Add `accessible-description` with the source URL or ordinal ("Icon 1 of 5").

- [ ] **Category dropdown has no accessible label**: `webapp_dialog.py:260` — The `Gtk.DropDown` is added as a suffix to `AdwActionRow`. While AdwActionRow provides some labeling, the dropdown itself needs `set_accessible_name(_("Category"))`.

- [ ] **Profile switch has no accessible label**: `webapp_dialog.py:303` — `Gtk.Switch` is a suffix. The parent `AdwActionRow` title helps, but explicit `switch.set_accessible_name()` is recommended for Orca to announce "Use separate profile, switch, off".

- [ ] **Search entry has no accessible label**: `main_window.py:107` — `Gtk.SearchEntry` inside `Gtk.SearchBar` has no label. Orca would announce "text entry" with no context. **Fix:** `search_entry.set_accessible_name(_("Search WebApps"))`.

### High — Missing state announcements

- [ ] **Loading overlay not announced**: `webapp_dialog.py:390-420` — The loading overlay shows a spinner and "Loading..." text. Orca users get no announcement that the detection is in progress or has completed. **Fix:** Set `accessible-role` for the overlay, or use `Atk.StateSet` to announce "busy" state. Alternatively, move focus to a status label.

- [ ] **Toast notifications are not announced by Orca**: While `Adw.Toast` may or may not be picked up by screen readers depending on the version, there's no guarantee. **Fix:** Supplement toasts with `Gtk.AccessibleRole.STATUS` or `aria-live` equivalent. Consider using `Adw.MessageDialog` for critical success/failure messages when screen reader is active.

- [ ] **Empty state not focused on load**: `main_window.py:133-138` — When there are no webapps, the `AdwStatusPage` is shown but not focused. Orca user may not know the page is empty. **Fix:** Grab focus to the status page or announce it.

### Medium — Navigation issues

- [ ] **No skip-navigation for category headers**: `main_window.py:494` — Category headers are `Gtk.Label` elements, not focusable. Screen reader users must Tab through every webapp to reach the next category. **Fix:** Use `Gtk.Expander` or heading landmarks.

- [ ] **Dialog focus order is not optimal**: `webapp_dialog.py` — The first focusable element is the URL entry, which is correct for new webapps. But for editing existing webapps, the name field might be more relevant. Consider dynamic focus based on `is_new`.

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

- [ ] All interactive elements have accessible labels — **FAIL** (buttons, entries, dropdown)
- [ ] Keyboard navigation works for all flows — **PARTIAL** (ESC closes dialogs ✓, Tab order untested)
- [ ] Color is never the only indicator — **PARTIAL** (delete icon uses `error` CSS class = red only)
- [ ] Text is readable at 2x font size — **UNTESTED** (no responsive breakpoints; `AdwClamp` is used in dialog ✓)
- [ ] Focus indicators are visible — **DEFAULT** (relies on Adwaita theme defaults, should be fine)

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

### From radon (7 high-complexity functions)
| Function | CC | Grade | Location |
|---|---|---|---|
| `get_system_default_browser` | 54 | F | `command_executor.py:160` |
| `on_webapp_dialog_response` | 17 | C | `main_window.py:204` |
| `_handle_import_response` | 15 | C | `application.py:295` |
| `_handle_export_response` | 14 | C | `application.py:174` |
| `_fetch_info_thread` | 13 | C | `url_utils.py:118` |
| `handle_starttag` | 12 | C | `url_utils.py:33` |
| `WebAppDialog.__init__` | 11 | C | `webapp_dialog.py:32` |

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
