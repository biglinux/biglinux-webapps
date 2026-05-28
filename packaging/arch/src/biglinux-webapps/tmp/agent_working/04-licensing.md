# Stage 4 — License & supply chain

Tool: `cargo deny 0.19.4` with provisional `deny.toml` (no committed config yet).

## 1. `cargo deny check` — provisional config

`licenses` and `sources` pass. `bans` FAIL on 3 + 5 warnings:

### errors
| crate | issue | fix |
|---|---|---|
| `webapps-core = { path = "../webapps-core" }` (consumer: `webapps-exec`) | wildcard intra-workspace dep — no `version =` field | add `version = "4.0.6"` or convert to `workspace = true` |
| same for `webapps-manager` | same | same |
| same for `webapps-viewer` | same | same |

These are intra-workspace path deps. cargo-deny treats absent `version` as a wildcard. For a workspace that *publishes* any crate the version is required; for a local-only workspace it is a soft requirement that keeps `cargo publish` honest. Recommended: add a workspace dep entry.

### warnings (duplicates)
| crate | versions | upgrade path |
|---|---|---|
| `getrandom` | 0.2.17, 0.3.4 | `rand`/`reqwest`/`tempfile` ecosystem mid-transition; no action — resolves over time |
| `serde_spanned` | 0.6.9, 1.1.1 | pulled by both `toml 0.8` (manager) and `toml 1.1` (transitive) |
| `toml`, `toml_datetime`, `toml_edit` | 0.x + 1.x | same; bump `webapps-core::browsers` parsing to `toml 1` to collapse |
| `winnow` | 0.7.15, 1.0.1 | transitive of `toml` versions above |

Net duplicate-version cost is small; collapsing toml→1.x is the leverage point.

## 2. SPDX in Cargo.toml

PASS. `[workspace.package] license = "GPL-3.0-or-later"`; each crate inherits via `license.workspace = true`. SPDX-valid.

## 3. LICENSE file at repo root

PASS. `LICENSE` (GPL v3 full text). No per-crate LICENSE needed under GPL.

## 4. Per-source SPDX headers

FAIL (advisory, not blocking). 0 source files carry an SPDX-License-Identifier comment:

```
crates/webapps-core/src/lib.rs
crates/webapps-manager/src/lib.rs
crates/webapps-exec/src/main.rs
crates/webapps-manager/src/main.rs
crates/webapps-viewer/src/main.rs
+ all submodules
```

GPL distribution accepts repo-root LICENSE coverage when `Cargo.toml` declares the license, so this is GPL-compliant as-is. Recommendation: add `// SPDX-License-Identifier: GPL-3.0-or-later` to the top of every `lib.rs`/`main.rs` (5 files) for tooling-friendliness (`reuse lint`, `licensee`). Skip non-entry-point files.

## 5. Vendored binary blobs

None. `find . -type f \( -name "*.so" -o -name "*.dll" -o -name "*.wasm" -o -name "*.onnx" -o -name "*.gguf" -o -name "*.safetensors" -o -name "*.bin" \)` returns empty outside `target/`. PASS.

## 6. Transitive license conflicts

Distribution license = **GPL-3.0-or-later**. Transitive deps inventory (cargo deny list):
- MIT / Apache-2.0 / BSD-3 / ISC / Zlib / 0BSD / Unicode-3.0 — all permissive, GPL-compatible.
- **LGPL-2.1-or-later** (1): `r-efi@5.3.0` — GPL-compatible.
- **MPL-2.0** (5): `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, `selectors` — MPL ≤ GPL is compatible per FSF.
- **BSL-1.0** (1): `ryu` — equivalent to MIT for compatibility.
- **Unlicense** (4): `aho-corasick`, `byteorder-lite`, `jiff`, `memchr` — public-domain dedication, compatible.
- **No GPL/AGPL/SSPL transitive deps that would conflict.**

PASS.

## 7. RustSec advisories

Not run (`advisories` section omitted from provisional config to avoid network fetch). Stage 13 CI will run `cargo deny check advisories` on every push.

## Actions

| # | priority | action |
|---|---|---|
| 1 | P0 | commit `deny.toml` at repo root (see §A below) |
| 2 | P0 | resolve the 3 wildcard-dep errors — add `version = "4.0.6"` to each intra path dep, or hoist `webapps-core` into `[workspace.dependencies]` |
| 3 | P1 | add `// SPDX-License-Identifier: GPL-3.0-or-later` to the 5 entry-point files |
| 4 | P2 | collapse `toml 0.8 → 1.x` to drop 4 duplicate-version warnings |
| 5 | — | no replacement / re-license / vendor-header / waiver needed |

## §A — proposed `deny.toml`

```toml
[graph]
all-features = false

[advisories]
db-urls = ["https://github.com/rustsec/advisory-db"]
yanked = "deny"

[licenses]
confidence-threshold = 0.93
allow = [
    "Apache-2.0", "Apache-2.0 WITH LLVM-exception",
    "MIT", "BSD-2-Clause", "BSD-3-Clause", "ISC",
    "Zlib", "0BSD", "BSL-1.0", "Unicode-3.0",
    "MPL-2.0", "LGPL-2.1-or-later", "Unlicense",
    "GPL-3.0-or-later",
]

[bans]
multiple-versions = "warn"
wildcards = "deny"
allow-wildcard-paths = true   # intra-workspace path deps

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

The `allow-wildcard-paths = true` knob (cargo-deny 0.13+) lets path-only deps slide. Otherwise add `version = "4.0.6"` to each intra dep.
