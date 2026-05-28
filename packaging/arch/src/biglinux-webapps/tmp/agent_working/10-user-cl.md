# Stage 10 — User cognitive load

Surfaces audited: manager window (`window/list.rs`, `webapp_dialog`, `browser_dialog`, `template_gallery`, `welcome_dialog`), viewer (`webapps-viewer/src/window`). Reference: `01-map.md §1`.

## Primary task step counts

| journey | steps | notes |
|---|---:|---|
| Create webapp from scratch | 4 | header **Add** → URL → Name (auto-filled from favicon) → **Save**. Browser defaults to system default. |
| Create webapp from template | 3 | header **Add** → pick template card → **Save** (URL/Name/icon prefilled by template) |
| Launch webapp | 1 | desktop entry → `big-webapps-exec` exec()s browser. No manager involved. |
| Change browser of existing webapp | 3 | row globe button → pick browser → **Save** (toast confirms) |
| Remove webapp | 2 | row trash → **Remove** in destructive AlertDialog |
| Import webapps zip | 3 | main menu → Import → file picker → done (toast) |
| Export all webapps | 2 | main menu → Export → file picker |

All primary journeys ≤ 4 steps. No surface exceeds the 5-step guideline.

## Default-bias review

| field | default | bias | verdict |
|---|---|---|---|
| `WebApp::default().browser` | system default browser (`BrowserStore::default_browser`) | toward user's already-chosen browser | **good** |
| `webapp_dialog` profile mode | shared profile (not isolated) | toward fewer disk writes / faster launch | **good for first-run; isolation is opt-in** |
| `auto_hide_headerbar` | false (Browser mode), implicit true (Viewer App mode) | viewer is chromeless by design | **good** |
| AlertDialog "Remove WebApp?" | `default_response = "cancel"` | toward non-destructive | **good — destructive is never default** |
| Welcome dialog "don't show again" | unset (modal re-shows until user toggles) | toward visibility for new users | **good** |
| Favicon scan | runs on Save automatically | toward "it just works" | **good** |

No bias correction needed.

## Naming consistency

Casing audit of user-visible noun:

| string | location | form |
|---|---|---|
| "WebApp" | almost all gettext sites | **canonical** |
| "Webapp" | none found | — |
| "webapp" | none found in user-visible strings (only as code identifier) | — |
| "WebApps" (plural) | window title, menu | canonical plural |
| "web app" / "web-app" | none | — |

**Verdict: consistent.** One canonical spelling — `WebApp` / `WebApps`. No mixed forms surfaced to user.

Action labels also consistent: **Add**, **Save**, **Remove**, **Cancel**, **Import**, **Export** — verbs first, no trailing punctuation, all gettext-routed.

## Surprise audit

Looked for actions that take effect without clear user signal:

| candidate | finding |
|---|---|
| Save triggers favicon network fetch | scan visible via spinner on the icon row; toast confirms save. **Not a surprise.** |
| Browser change writes desktop entry immediately | toast confirms; no "Are you sure?". Reversible by another browser change. **Acceptable — low blast radius.** |
| Remove deletes desktop entry **and optionally** the profile folder | gated by extra `CheckButton` in AlertDialog (`list.rs:204`) — opt-in for profile removal, with `app.has_custom_profile() && !service::profile_shared(app)` guard. **Well-handled.** |
| Import overwrites existing webapps with same `app_file` | service layer dedupes by hashing browser+url; no silent overwrite path observed. **OK.** |
| Welcome dialog never shows again after dismiss | persisted via `welcome_shown` config flag; reset path: edit `~/.config/biglinux-webapps/config.json`. Could expose toggle in preferences — minor. **Flagged C** |

## Empty-state coverage

| surface | empty state | quality |
|---|---|---|
| Manager list (no webapps) | `adw::StatusPage` icon + title + description + pill CTA "Add WebApp" (`list.rs:54`) | **excellent** — title explains, CTA is the obvious next step |
| Manager search (no matches) | status label "{} results" with `0` (`list.rs:37`) — list area shows nothing | **C-flagged**: empty result set drops to a blank area below the chip. Better: `StatusPage` saying "No matches for '{query}'" with a "Clear search" action. |
| Template gallery (no templates for filter) | not reachable; filter is built from non-empty registry. n/a |
| Browser picker (no browsers installed) | shows the **Viewer** entry always (built-in). Cannot truly be empty. **OK** |
| Viewer (page load fails) | `loading.rs` shows error overlay with retry. **OK** |

## Findings

- **C-10.1** (minor UX): search-zero-results renders a blank list instead of an explanatory `StatusPage`. ~25 LOC fix in `list.rs::populate_list` (branch when `result_count == 0 && has_active_filter`).
- **C-10.2** (minor UX): no in-UI path to re-show the welcome dialog after dismissal. Add a "Show welcome" entry in the main menu (1 line + handler).

## Verdict

User cognitive load is **low**. Step counts are tight, defaults are non-aggressive, destructive actions are gated, naming is consistent. Two small C-grade UX gaps queued for follow-up; no source changes from this stage.
