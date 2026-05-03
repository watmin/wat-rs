# Arc 138 F-NAMES-2/4-coords — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `ad1aeb65143cc17d5`
**Runtime:** ~9 min (540 s).

## Verification

| Claim | Disk-verified |
|---|---|
| Files modified | 8 (3 substrate + 5 test files) ✓ |
| diff stat 31+/22- | ✓ |
| Lambda sites updated | 6 (sonnet found 1 more than the BRIEF's 5) ✓ — runtime.rs ×3, freeze.rs ×2, check.rs ×1 |
| Lambda format | `<lambda@<file>:<line>:<col>>` via `format!("<lambda@{}>", body.span())` ✓ |
| Entry sites updated | 10 (8 test files + 2 freeze.rs internal) ✓ |
| Entry label | `concat!(file!(), ":", line!())` at each test caller ✓ |
| `<entry>` user-visible occurrences | **0** ✓ |
| 7/7 arc138 canaries | PASS ✓ |
| Workspace tests | empty FAILED ✓ |

## Substrate observation — compose.rs base_canonical is dual-purpose

Sonnet's most important finding: `base_canonical: Option<&str>` in `startup_from_source(src, base_canonical, loader)` serves TWO purposes:
1. The file label for parsed-AST spans
2. The base directory for ScopedLoader relative path resolution (e.g., `(load-file! "helper.wat")`)

Initially passing `Some("<compose-and-run>")` broke the `with-loader-example`'s helper.wat resolution because `Path::new("<compose-and-run>").parent()` returns `""` → wrong base directory.

**Decision:** compose.rs:187 keeps `None` and the `<entry>` fallback at src/freeze.rs:421 IS correct architecture for this site — no disk path, in-memory source. The label `<entry>` is honest for this case; the loader uses other plumbing (CARGO_MANIFEST_DIR via env) to resolve relative paths.

This surfaces a real substrate observation: the parameter conflates two concerns. Splitting it (e.g., `base_dir: Option<&Path>` vs `source_label: &str`) would be a separate substrate refactor. Out of scope for F-NAMES.

## Substrate observation — fork.rs:961 was pre-correct

Sonnet honestly noted the BRIEF was wrong about fork.rs:961 — that site already passes `canonical.as_deref()` (a real path), not None. Pre-existing correct architecture.

## Calibration

Predicted 10-20 min; actual 9 min. Sonnet's pattern application was clean and the dual-purpose-parameter discovery is a valuable honest delta.

## Hard scorecard: 6/6 PASS. Soft: 3/3 PASS+.

## Ship decision

**SHIP.** Every identity-style placeholder template in the substrate now pairs with real coordinates:
- `<lambda@<defining-file>:<line>:<col>>` for anonymous lambdas
- `<entry>` only fires for the compose.rs in-memory entry case (architecturally correct; the source genuinely has no disk path)

## Arc 138 status post-F-NAMES-2/4-coords

**ALL KNOWN TEMPLATE-STRING CRACKS CLOSED.** The remaining `<...>` strings in the substrate are:
- **Type/shape labels** (`<Vector dim=N>`, `<WatAST>`, `<Sender>`, `<lambda>` as Value type-name display) — describe value SHAPE, intentional UI.
- **Input-mode labels** (`<source>`, `<path>`, `<env>`, `<none>`, `<non-keyword-head>`) — describe input modes, intentional UI.
- **Field-absent indicators** (`<unknown>`, `<symbol>` in Frame display) — render when an Option<String> field is genuinely None; no source to point at.
- **Sentinel** (`<runtime>`) — Span::unknown() default file; suppressed by is_unknown() before render; never user-visible.
- **Architectural in-memory entry** (`<entry>` from compose.rs) — genuine no-disk-path case; dual-purpose parameter constraint documented.

None are missing identities that COULD pair with coordinates.

## Only slice 6 remains

- Slice 6: doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure.
