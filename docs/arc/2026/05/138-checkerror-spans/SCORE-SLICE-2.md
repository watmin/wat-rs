# Arc 138 Slice 2 — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `a6d3c60d0798776e0`
**Runtime:** ~10 min (601 s).

## Independent verification (orchestrator)

| Claim | Sonnet's value | Orchestrator verified |
|---|---|---|
| `Span::unknown()` BEFORE | 0 | 0 ✓ (pre-spawn baseline confirmed in EXPECTATIONS) |
| `Span::unknown()` AFTER | 4 | **4** ✓ (all in newly-added `_with_span` sibling wrappers; Pattern E with rationale) |
| `// arc 138: no span` comment count | 4 | **4** ✓ (one per leftover) |
| `git diff --stat src/types.rs` | 1 file, 272+ / 83- | **1 file, 272+ / 83-** ✓ |
| Canary `arc138_type_error_message_carries_span` | PASS | PASS ✓ (`types::tests::arc138_type_error_message_carries_span`) |
| Workspace `cargo test --release --workspace` | 0 failures (excl lab) | 0 failures (excl lab) ✓ |
| 10 variants gained span | yes | yes ✓ |
| 10 Display arms updated | yes; `MalformedVariant` unchanged | yes ✓ |

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Single-file diff | **PASS** | Only `src/types.rs` modified. No new test files; canary fits in existing `mod tests`. |
| 2 | All 10 variants gain `span: Span` | **PASS** | DuplicateType, ReservedPrefix, MalformedDecl, MalformedName, MalformedField, MalformedTypeExpr, AnyBanned, CyclicAlias, AliasArityMismatch, InnerColonInCompoundArg. MalformedVariant unchanged. |
| 3 | All 10 Display arms prefix coords | **PASS** | New `span_prefix` helper; uniform `{span_prefix(span)}` interpolation. MalformedVariant's `at <span>` shape preserved. |
| 4 | Emission sites thread real spans | **PASS** | ~31 sites real-spanned; 4 leftover Pattern E (each in a `_with_span` wrapper, each with rationale). 31/35 = 89% real-spanned. Exceeds 85% target. |
| 5 | Workspace tests pass | **PASS** | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. |
| 6 | Canary added + passes | **PASS** | `types::tests::arc138_type_error_message_carries_span` triggers `MalformedDecl` via `(:wat::core::enum :my::Empty)` and asserts `<test>:` substring in rendered Display. Verified passing. |
| 7 | No commits | **PASS** | Working tree shows uncommitted modification of `src/types.rs` only. |
| 8 | Honest report | **PASS+** | ~600 words; counts before/after; variant list; Display arms; emission distribution per variant; helper signatures broadened (8 helpers + 3 sibling functions); canary location + verification; substrate observations including the public-surface preservation pattern; AliasArityMismatch unreachability; trade-off on decl_span vs name-kw-span; four questions. |

**HARD VERDICT: 8 OF 8 PASS.**

## Soft scorecard (4 rows)

| # | Criterion | Result | Notes |
|---|---|---|---|
| 9 | Span source quality | PASS | Per-variant span source distribution matches the BRIEF table reasonably; trade-off on decl_span vs name-kw-span surfaced honestly (sonnet uses decl_span when the name keyword is consumed before register is called — pragmatic; both point to the same line). |
| 10 | Helper signatures broadened deliberately | **PASS+** | 8 helpers gained span params (parse_type_decl, parse_struct, parse_enum, parse_newtype, parse_typealias, parse_declared_name, parse_type_inner + 4 type-expr parsers, reject_any, check_alias_no_cycle, check_alias_reaches). Plus 3 NEW public sibling functions (`register_with_span`, `register_stdlib_with_span`, `parse_type_expr_with_span`) preserve backward compat for external callers. Same uniform-signature discipline as slice 1's `head_span: &Span`. |
| 11 | Workspace runtime | PASS | Test runtime within baseline. |
| 12 | Honest delta on AliasArityMismatch | **PASS+** | Sonnet correctly observed: variant DECLARED but never EMITTED in the current code path. Field reserved per BRIEF; Display arm prefixes correctly when constructed externally. Future emitters get the slot for free. |

**SOFT VERDICT: 4 OF 4 PASS+. Clean ship.**

## Substrate observation — public-surface preservation pattern

Sonnet introduced a NEW design pattern: when a public function needs broader signature (extra `span` parameter) but external callers exist, add a `_with_span` sibling that takes the new parameter; the original delegates with `Span::unknown()`. Three sibling functions added: `TypeEnv::register_with_span`, `TypeEnv::register_stdlib_with_span`, `parse_type_expr_with_span`.

This is the right shape for substrate evolution — backwards-compat preserved at the API boundary, new callers opt in. Same trade-off as slice 1's `SchemeCtx` trait gap: a real architectural boundary that earns follow-up work when downstream demand surfaces. Per arc 138's slice 3 (RuntimeError) work, the pattern is now reusable.

The 4 leftover `Span::unknown()` sites are all in these wrapper functions (where `Span::unknown()` is the SOURCE-OF-TRUTH absent — the original API's callers genuinely don't have span context). Each carries a rationale. Acceptable trade-off; documented.

## Substrate observation — AliasArityMismatch unreachable

The variant exists in the enum but no `Err(TypeError::AliasArityMismatch { ... })` site exists in `src/types.rs`. The variant + Display arm shipped (the field is reserved); when a future emitter adds it (via the alias arity check elsewhere), it gets the span slot for free. Honest naming in deltas.

## Independent prediction calibration

Predicted 65% chance of 8/8+4/4 with possible helper-signature broadening (25%). Actual: **8/8 hard + 4/4 soft + helper broadening**, both predictions in the bucket.

Sonnet runtime: 10 min (vs predicted 25-40). Faster than expected — the structural changes (variant + Display + threading) were tightly bounded, and the worked example (arc 130's MalformedVariant) lived in the same file.

## Ship decision

**SHIP.** 8/8 hard + 4/4 soft. The substrate observation about `_with_span` sibling functions becomes input to slice 3 design (RuntimeError spans likely need a similar pattern at apply_function / eval boundaries).

## Next steps

1. Commit slice 2 + this SCORE.
2. Push.
3. Write slice 3 BRIEF (RuntimeError variants in `src/runtime.rs` — ~22 user-facing variants identified in DESIGN). Likely larger sweep; sonnet runtime 30-50 min.
4. Score slice 3 → continue through slices 4-6.

## What this slice tells us

- Arc 138's pattern propagates across Rust files. Slice 1 (check.rs) and slice 2 (types.rs) both clean ship with similar substrate observations.
- The `_with_span` sibling pattern is a NEW substrate-evolution shape — backwards-compat at the API boundary, opt-in for new callers. Expected to recur in slice 3.
- The "earned for follow-up" trade-off (sites where threading would expand a trait surface or break public API) is the consistent honest shape — not papered over, named for future work.
- Sonnet's calibration stays strong: 7/8 → 8/8 → 8/8 → 8/8 (slice 1 finish) → 8/8 (here).

Sonnet's WORK was clean. Sonnet's REPORT was clean (no fabrication; counts match `git diff --stat`; substrate observations honest including a NEW design pattern named explicitly). Trust-but-verify confirms both.
