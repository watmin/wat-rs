# Arc 138 F-NAMES-1 — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `abf5fb35d02bd3d97`
**Runtime:** ~21 min (1293 s) — within 25-40 min prediction band.

## Verification

| Claim | Disk-verified |
|---|---|
| Files modified | 15 ✓ |
| diff stat 204+/192- | ✓ |
| `parse_one!` + `parse_all!` macros added at src/parser.rs lines 73 + 89 | ✓ — both #[macro_export]'d, importable as `crate::parse_one!` / `wat::parse_one!`, with `$(,)?` trailing comma support |
| Test caller sweep | ~132 macro substitutions across 14 files ✓ |
| Production caller updates | 4 sites: load.rs (fetched.canonical_path), stdlib.rs (file.path), runtime.rs::parse_program (`<runtime-eval>` self-id), lib.rs::eval_algebra_source (`parse_one!` macro auto-captures src/lib.rs:201) ✓ |
| `parse_one(src)` + `parse_all(src)` convenience wrappers deleted | ✓ — `grep "^pub fn parse_one(\|^pub fn parse_all(" src/parser.rs` returns empty |
| `<test>` placeholder eliminated | ✓ — only `src/lexer.rs:461` lex test fixture remains; everything else gone |
| All 7 arc138 canaries | 7/7 PASS ✓ |
| Workspace tests | empty FAILED ✓ |

## Hard scorecard: 8/8 PASS. Soft: 3/3 PASS+.

## Substrate observation — clean public-API workaround

Sonnet identified that `pub fn eval_algebra_source` (the BRIEF's `pub fn run` reference) had 14 external test callers in `tests/mvp_end_to_end.rs`. Adding a `source_label` parameter would have broken all 14. Sonnet chose to use `parse_one!(src)` INSIDE the function — the macro auto-captures `src/lib.rs:201` as the source label. Functionally equivalent to passing a label; no public API break. Honest call.

## Substrate observation — `<runtime-eval>` self-identification

src/runtime.rs::parse_program has no real source path (it parses dynamic strings from runtime eval forms). Sonnet used `"<runtime-eval>"` as the label — self-identifies the dynamic-eval context, distinct from `<test>` which was a meaningless placeholder. Honest framing.

## Substrate observation — wat-edn separation respected

wat-edn has its own `parse_all` function (different namespace, EDN parser). Sonnet correctly left it alone. Cross-crate scope discipline.

## Substrate observation — additional span-carrying tests updated

Two non-arc138-named tests in src/check.rs (`type_mismatch_message_carries_span`, `sandbox_scope_leak_fires_with_diagnostic`) checked `rendered.contains("<test>:")` — which would now fail after the placeholder is gone. Sonnet updated to `contains("src/") || contains(".rs:")`. Mandatory; preserves test intent.

## Substrate observation — macro hygiene fix

Initial macro definition didn't accept trailing commas (`$src:expr` only). Multi-line test calls with trailing commas hit 16 compile errors. Sonnet fixed by adding `$(,)?` to both matchers. Honest delta documented.

## Calibration

Predicted 25-40 min; actual 21 min. Within band. Largest sweep slice in arc 138 (132 mechanical substitutions + ~5 thoughtful updates + 2 wrapper deletions + macro defs + 2 unrelated test updates) shipped on first sonnet engagement. The simple-is-uniform-composition principle held — one big mechanical sweep is one slice.

## Ship decision

**SHIP.** `<test>` placeholder is gone from production paths. Every test panic now navigates to a real Rust file:line.

## Next per NAMES-AUDIT

- **F-NAMES-1c**: wat::test! deftest thread name (`<unnamed>` → `wat-test::deftest_name`). Single-file fix in crates/wat-macros/src/lib.rs. ~5-10 min.
- **F-NAMES-2**: `<lambda>` audit
- **F-NAMES-3**: `<runtime>` invariant check
- **F-NAMES-4**: `<entry>` investigation
- **Slice 6**: doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure
