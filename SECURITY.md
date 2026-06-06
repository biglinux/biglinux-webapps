# Security Policy — biglinux-webapps

## Supported versions

| Version | Supported |
|---------|-----------|
| latest stable (main) | ✅ |
| previous minor       | ✅ (security patches) |
| older                | ❌ |

## Reporting a vulnerability

**Do NOT open public issues for security bugs.**

- **Preferred:** GitHub Security Advisory — https://github.com/biglinux/biglinux-webapps/security/advisories/new
- **Backup:** email `security@biglinux.com.br` (PGP key on keys.openpgp.org, fingerprint TBD)

Include: affected version, reproduction steps, impact, suggested fix (optional).

## Response SLA

| Severity | First response | Patch target |
|----------|---------------:|-------------:|
| CRITICAL (RCE, privilege escalation, data loss) | 24h | 72h |
| HIGH (auth bypass, sandbox escape)              | 72h | 7d  |
| MEDIUM (info leak, DoS)                         | 7d  | 30d |
| LOW (defense-in-depth)                          | 14d | next minor |

## In scope

- WebView sandbox flags (no host filesystem access, no node integration)
- Per-app profile dir isolation (cookies, storage, cache scoped per webapp)
- URL validation against allowlist scheme (`https://`, no `file://`, no `javascript:`)
- Atomic JSON profile/state write via tmp-file + `rename` (crash-mid-rename safe; `crates/webapps-viewer/src/window/permissions/mod.rs`, `crates/webapps-manager/src/service/repository.rs`)
- Icon download path canonicalization (no traversal into XDG dirs)
- Subprocess argv terminator on launcher invocations

## Out of scope

- Bugs reproducible only with non-default debug builds (RUSTFLAGS=-C debug-assertions)
- Issues in third-party deps without exploitable path through this code (report upstream)
- Self-XSS, social engineering
- DoS via resource exhaustion below documented limits (see INVARIANTS.md budgets)

## Disclosure

Coordinated. CVE requested when applicable. Credit in CHANGELOG + release notes. 90-day default embargo unless severity dictates faster public.

## Security-relevant invariants

See `INVARIANTS.md` for the enforced contract (subprocess argv, path canonicalization, FFI lifetimes, etc.).

## STRIDE mapping

| Threat | Mitigation |
|--------|-----------|
| Spoofing | URL scheme allowlist, per-app origin pinning |
| Tampering | atomic tmp+rename writes; exclusive advisory lock (`fs2::FileExt::lock_exclusive` on `webapps.json.lock`) across every read-modify-write transaction |
| Repudiation | logs via `log` + `env_logger` |
| Information disclosure | per-app profile dir isolation |
| DoS | favicon/manifest fetch size + timeout caps (`crates/webapps-manager/src/favicon/download.rs`) |
| Elevation of privilege | WebView sandbox, no setuid, user-only install |
