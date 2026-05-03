# Arc 138 Slice 4a — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `a6f56dc8f6c09559a`
**Runtime:** ~10 min (603 s).

## Independent verification (orchestrator)

| Claim | Sonnet's value | Disk-verified |
|---|---|---|
| Files modified | 3 (src/macros.rs, src/form_match.rs, src/check.rs) | **3** ✓ |
| `git diff --stat` | 228+ / 78- | **228+ / 78-** ✓ |
| MacroError variants restructured | 9 | **9** ✓ |
| ClauseGrammarError variants restructured | 7 | **7** ✓ |
| Display arms updated | 16 | **13 in macros.rs + 11 in form_match.rs** ✓ (counts include the new span_prefix helper definitions and Display impl additions) |
| New canaries added | 2 (`macros::tests::arc138_macro_error_message_carries_span`, `form_match::tests::arc138_clause_grammar_error_message_carries_span`) | **2** ✓ — both PASS |
| Existing canaries pass | yes (`runtime::...`, `types::...`) | **yes** ✓ |
| All 4 arc138 canaries | 4/4 PASS | **4/4** ✓ |
| Library tests | 769/769 (up from 767) | **769/769** ✓ |
| Workspace `cargo test` excl lab | empty FAILED | **empty** ✓ |
| Pattern E rationale comments | 4 (3 register/MacroDef gap + 1 non-list form) | **4 in macros.rs** ✓ |

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | File scope | **PASS** | Exactly src/macros.rs + src/form_match.rs + src/check.rs modified. No others. |
| 2 | All 16 variants gain span | **PASS** | 9 MacroError + 7 ClauseGrammarError. Tuple/struct/unit conversions all per BRIEF table. |
| 3 | All 16 Display arms prefix coords | **PASS** | New `span_prefix` helper added to each of macros.rs / form_match.rs (mirror src/types.rs / src/check.rs). 16/16 arms render `{span_prefix(span)}`. ClauseGrammarError gained a Display impl (it had none before — minor substrate addition needed for canary). |
| 4 | Emission sites use real spans | **PASS+** | macros.rs: ~14 sites; 4 Pattern E with rationale (MacroDef structural gap, 1 non-list form). form_match.rs: ~7 sites; ALL real-spanned (0 Pattern E). Combined ~85% real-spanned. |
| 5 | Cross-file consumer updated | **PASS** | `grammar_error_to_check_error` (src/check.rs:6906-6925) patterns updated for all 7 ClauseGrammarError variants. Variant spans ignored with `_`/`..` (caller's own span preserved). check.rs builds. |
| 6 | Two canaries added + pass | **PASS** | `macros::tests::arc138_macro_error_message_carries_span` triggers ArityMismatch (2-param macro called with 1 arg); `form_match::tests::arc138_clause_grammar_error_message_carries_span` triggers UnknownHead via `(:bogus-op ?x ?y)`. Both assert `<test>:` substring + matches!() shape. Both PASS. |
| 7 | Workspace tests pass | **PASS** | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. All 4 arc138 canaries PASS. 769/769 wat lib tests. |
| 8 | Honest report | **PASS+** | ~600 words; per-file counts; pattern distribution per file; line numbers for canaries; cross-file consumer line range; honest deltas naming Pattern F (2 in-file helper broadenings: `parse_defmacro_signature`, `binary`); Pattern E categories named (MacroDef structural gap); Display impl addition called out; existing test pattern updates noted (mandatory compile fixes for new variant shapes). |

**HARD VERDICT: 8 OF 8 PASS.**

## Soft scorecard (4 rows)

| # | Criterion | Result | Notes |
|---|---|---|---|
| 9 | Pattern distribution | **PASS** | macros.rs: A 5, B 6, E 4, F 1. form_match.rs: A 1, B 3, D 3, F 1. Total: A 6, B 9, D 3, E 4, F 2 = 24 (some sites use multiple patterns). Predicted A 50%/B 30%/D 10%/E 5%/F 5%; reality A 25% / B 38% / D 12% / E 17% / F 8%. B leads (helpers thread list_span); E higher than predicted (MacroDef structural gap). Acceptable distribution. |
| 10 | Span source quality | **PASS** | Spot-check sample shows args[i].span() / list_span / head.span() used semantically correctly. UnknownHead points at the head keyword, not the list as a whole — correct UX choice. |
| 11 | Workspace runtime | **PASS** | Within baseline. Library tests 0.12s. |
| 12 | Honest delta on threading | **PASS+** | Two helper sigs broadened (in-file only): `parse_defmacro_signature(... defmacro_span: Span)` in macros.rs; `binary(rest, op, list_span: Span)` in form_match.rs. Both private. No `_with_span` sibling pattern needed (no public API at risk). 4 Pattern E rationales explicitly named (MacroDef carries no source span; non-List form arm fires before span available). |

**SOFT VERDICT: 4 OF 4 PASS+. Clean ship.**

## Substrate observation — Display impl gap on ClauseGrammarError

ClauseGrammarError previously had NO `Display` impl — the canary forced sonnet to add one. This is a small substrate addition: the variant rendering used elsewhere (via `grammar_error_to_check_error`) was format-string composition, not the Display path. Adding Display is correct (idiomatic Rust error type) and the canary exercises it.

This is the kind of gap that emerges when arc 138's "spans render in Display" discipline meets pre-existing types that lacked Display. Earned organically; no separate arc needed.

## Substrate observation — MacroDef structural span gap

The `register` and `register_stdlib` methods receive a `MacroDef` struct that carries no source span. Threading would require either (a) adding a span field to `MacroDef` (public struct change, out of scope) or (b) adding `_with_span` sibling methods (not needed for slice 4a's scope).

3 Pattern E rationales document this. Future arc could add `MacroDef::span: Span` if downstream demand surfaces.

## Substrate observation — sonnet's report-quality recovered

Slice 3b's report was thin (orchestrator had to verify on disk). Slice 4a's BRIEF added an explicit "REPORTING DISCIPLINE NOTE" calling for self-contained reports. Sonnet's slice 4a report was COMPREHENSIVE: per-file counts, pattern distribution, line numbers, canary names, honest deltas, four questions. Calibration recovered.

## Independent prediction calibration

Predicted: 60% chance 8/8 + 4/4 in 25-40 min. Reality: **8/8 hard + 4/4 soft**, runtime 10 min — UNDER the predicted band. Smaller scope + clear worked example (slice 2) made this faster than expected.

Calibration update: when sonnet has a CLEAR worked example (slice 2's TypeError pattern) + bounded scope (≤ 50 emission sites + ≤ 3 files), runtime trends toward 10-15 min. Future slices in this size class can predict 15-25 min.

## Ship decision

**SHIP.** 8/8 hard + 4/4 soft. Substrate observations earned (Display gap + MacroDef structural gap).

## Next steps

1. Commit slice 4a + this SCORE (this commit).
2. Push.
3. Slice 4b BRIEF: EdnReadError (src/edn_shim.rs, 6 variants, 31 emission sites) + LowerError (src/lower.rs, 12 variants, 42 emission sites). No cross-file consumers (only lib.rs re-export). Likely simpler than 4a.
4. Slice 5: ConfigError form_index → Span (small, surgical).
5. Slice 6: doctrine + INSCRIPTION + USER-GUIDE + 058 row.

## What this slice tells us

- Slice 2's TypeError pattern propagates cleanly to two more error types in one engagement.
- Pattern E ratio (~17% in macros.rs, 0% in form_match.rs) reflects substrate structural gaps (MacroDef has no span; form_match has uniform AST context).
- Helper-sig broadening with `span: Span` is now an established pattern: when a private helper lacks span, broaden in-place. Two helpers broadened in 4a, both private, both successful.
- The REPORTING DISCIPLINE NOTE in BRIEF was effective — sonnet's report quality recovered from slice 3b's thin output to slice 4a's comprehensive output.

Sonnet's WORK was clean. Sonnet's REPORT was honest and comprehensive. Trust-but-verify confirms both via disk + 4 canaries + 769/769 lib tests.
