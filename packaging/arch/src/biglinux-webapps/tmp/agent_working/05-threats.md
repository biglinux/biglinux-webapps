# Stage 5 — STRIDE threat model

Scope: every boundary in `01-map.md §5`. Privilege model: the binaries run unprivileged as the user. There is no setuid/sudo/polkit surface (`grep` confirms). Trust boundary lives between the local user and:
1. content fetched over HTTP (favicons, manifests)
2. webapp ZIP archives the user imports
3. websites loaded in the WebKit viewer
4. argv interpolated into shell-less `Command::new` calls

Subprocess argv uses `Command::args(&[..])` exclusively — no shell, no `sh -c`. No string concatenation into argv. That eliminates the classic shell-injection class up front; the remaining risks are semantic.

---

## High severity

### H-1 — flatpak override grants arbitrary filesystem path
- **Boundary**: `webapps-exec/src/launch.rs:78 grant_flatpak_access` →
  `flatpak override --user --filesystem=<profiles_dir>/<browser_id> <app_id>`
- **STRIDE**: Elevation of Privilege (user-scope, not root) + Tampering
- **Vector**: `browser_id` flows from the `.desktop` `Exec=` line argv. A maliciously crafted webapp (or a modified desktop entry) can pick a `browser_id` like `../../..` and persuade `flatpak override` to widen access of the named `app_id` to an arbitrary host path.
- **Status today**: `browser_id` is the second positional argv of `big-webapps-exec`; nothing validates it before `profiles_dir.join(browser_id)`. `Path::join` happily accepts `..`.
- **Fix**: validate `browser_id` against the canonical list in `webapps_core::browsers::find_def(...).is_some()` *before* any path join. Reject otherwise. Same fix applies to `firefox()`/`chromium()` profile_dir construction at `launch.rs:29,176`.

### H-2 — webapp import skips zip entries that fail canonicalize for transient reasons
- **Boundary**: `webapps-manager/src/service/io.rs:55 import_webapps`
- **STRIDE**: Tampering
- **Status today**: GOOD — current code (lines 95-117) requires `parent.canonicalize() == icons_canonical`, denies on canonicalize failure, caps `entry.size()` and the actual decompressed `copy()` to `MAX_EXTRACTED_FILE_BYTES`. Filename guard rejects `/`, `\`, `..`. Comment explicitly notes the previous silent-allow bug was fixed.
- **Residual risk**: only `icons/<basename>` entries are extracted; manifest `webapps.json` is parsed as JSON with no schema validation — a malicious manifest can inject arbitrary `app_url`/`app_name` (which then flow into desktop files). See M-2.
- **Fix**: leave as-is (defence in depth). Add a manifest schema check (URL must parse + scheme ∈ {http,https,file}, `app_id` matches `[A-Za-z0-9_.-]+`).

### H-3 — viewer permission default-deny is correct; window of misuse during prompt-decision persistence
- **Boundary**: `webapps-viewer/src/window/permissions/mod.rs classify_request` + `save_permissions`
- **STRIDE**: Information Disclosure / EoP
- **Status today**: GOOD baseline — unknown permission types fall through to `Deny`; `DeviceInfoPermissionRequest` is explicitly `Deny` (fingerprinting); `ClipboardPermissionRequest` requires a user prompt. **However** `save_permissions` writes JSON via `std::fs::write` (non-atomic) so a crash mid-write loses the user's saved deny; next launch the prompt re-appears, and a fatigued user might allow.
- **Fix**: route through the same `tmp + rename` pattern already used in `webapps-manager/src/service/repository.rs`. Trivial change.

---

## Medium severity

### M-1 — favicon HTTP client redirects 10 times + parses HTML from arbitrary origin
- **Boundary**: `webapps-manager/src/favicon/{mod,html,download}.rs`
- **STRIDE**: DoS (decompression bomb), Tampering (favicon path traversal already addressed)
- **Status today**: PARTIAL — redirect limit 10, but no response-size cap; `scraper`/`html5ever` parse may allocate freely on hostile HTML.
- **Fix**: cap `content-length`/streamed bytes at e.g. 5 MiB; reject `text/html` over that. Also reject non-HTTPS final URLs (today `normalize_http_url` allows `http://`).

### M-2 — imported webapp manifest values flow into `.desktop` `Exec=` line unvalidated
- **Boundary**: `service/io.rs::import_webapps` → `desktop/builder.rs::generate_desktop_entry`
- **STRIDE**: Tampering (host machine) + EoP (user-scope)
- **Vector**: imported `WebApp.app_url`/`app_name` reach `generate_desktop_entry` → `Exec=` line via `webapps-core/src/desktop/builder.rs:43 build_exec_command`. `desktop/sanitize.rs::sanitize_desktop_value` is the only defence; it strips line breaks but not, e.g., embedded `%U` field codes or quotes.
- **Status today**: PARTIAL — `sanitize_desktop_value` exists; verify it rejects `%[fFuUdDnNickvm]` field codes (those execute as substitutions per the Desktop Entry spec). Also verify `Type=Application` / `Exec=` line is built from a fixed template.
- **Fix**: extend sanitiser to escape `%` and reject control chars; add unit test mirroring the import path (zip → manifest → desktop builder).

### M-3 — Wayland icon-swap leaves backup on crash, lock-file in user-writable dir
- **Boundary**: `webapps-exec/src/wayland.rs swap_and_launch`
- **STRIDE**: Tampering (next launch picks up attacker-modified backup)
- **Status today**: PARTIAL — `flock(LOCK_EX|LOCK_NB)` prevents concurrent swap; auto-restore of stale backup at line 56 runs *before* the lock is acquired, so two instances could race the restore.
- **Fix**: move the stale-backup restore *inside* the lock-held branch; if lock isn't held, skip restore.

### M-4 — `dconf write` issued in KDE-detected branch? No — only when `XDG_CURRENT_DESKTOP` contains `"gnome"`
- **Boundary**: `webapps-core/src/desktop/paths.rs:131`
- **STRIDE**: Tampering of user dconf
- **Status today**: GOOD — argv is a static string array; no interpolation of user data. Only the trigger (env var) is attacker-influenceable. Worst case: forced re-creation of GNOME `WebApps` folder. Acceptable.

---

## Low severity

### L-1 — `HOME` env trusted blindly
- `webapps-exec/src/icon.rs:15`, `wayland.rs:40` → `format!("{home}/.local/share/applications")`
- A controlled subshell can spoof `HOME` to point at a writable share. Same as every desktop tool; XDG spec accepts this. **Accept**.

### L-2 — `LANG` used for `Accept-Language` header
- `webapps-manager/src/service/browser_url.rs:28` → user-controlled `LANG` flows to outbound HTTP. Fingerprintable but already public via every browser. **Accept**.

### L-3 — `update-desktop-database` / `xdg-settings` / `xdg-mime` / `dconf` PATH lookups
- Standard `PATH` lookup. Hostile `$PATH` ordering is out of scope (user already has shell access). Accept.

### L-4 — `BIG_WEBAPPS_SWAP_SETTLE_MS` parsed `u64` then `Duration::from_millis` — no upper bound
- Could be set to `u64::MAX` to wedge launch. User-controlled, low-impact. Cap at e.g. 5000 ms.

### L-5 — `Repudiation`: launch / import / permission decisions go only to `env_logger` (stderr); no audit file
- For a desktop tool this is fine. **Accept**.

### L-6 — WebKit DoS: untrusted JS can spin a tight loop, allocate GB. WebKit has its own caps + the OS will OOM-kill the per-webapp process. **Accept**; not a webapp-specific risk.

### L-7 — Information disclosure via crash dumps: panic backtraces include argv (url, profile). Cosmetic. **Accept**.

---

## Required commits before any other refactor

1. **H-1** — `browser_id` whitelist in `webapps-exec`. New file: `crates/webapps-exec/src/validate.rs` (10 LOC). Fail-closed on unknown id.
2. **H-3** — atomic `save_permissions`. 3-line change in `viewer/window/permissions/mod.rs`.
3. **M-2** — sanitiser hardening + import-path test. ~30 LOC in `webapps-core/src/desktop/sanitize.rs` + 1 test.
4. **M-3** — move stale-backup restore inside `Some(fd)` branch in `wayland.rs:55-65`. 5-line change.

Defer M-1, M-4, L-1..L-7 to next quarterly pass unless surfaced earlier.

Stage 5 commit message template: `security: <STRIDE-id> <one-line fix>`.
