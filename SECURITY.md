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
- Atomic JSON profile write via `BigAtomicJsonStore` (crash-mid-rename safe)
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
| Tampering | `BigAtomicJsonStore` for profiles, signed model artifacts |
| Repudiation | structured logs via `tracing` |
| Information disclosure | per-app profile isolation, redaction in logs |
| DoS | WebView process cap, profile size budget |
| Elevation of privilege | WebView sandbox, no setuid, user-only install |
