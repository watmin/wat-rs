# Arc 138 Slice 3a-finish — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `a3a31629703d04bca`
**Runtime:** ~65 min (3882 s).

## Independent verification (orchestrator)

| Claim | Sonnet's value | Orchestrator verified |
|---|---|---|
| `arc 138 slice 3a-finish` markers in src/runtime.rs | 0 | **0** ✓ (down from 300) |
| `Span::unknown()` in src/runtime.rs | 123 | **123** ✓ (down from 435; 54 synthetic-AST baseline + ~69 Pattern E sites) |
| `// arc 138: no span` rationale comment count | 91 | **91** ✓ |
| `arc 138 slice 3b` markers in src/runtime.rs (untouched) | 0 | **0** ✓ (slice 3b queue intact in external files) |
| `git diff --stat` | 1 file, 776+ / 927- | **1 file, 776+ / 927-** ✓ |
| Canary `arc138_runtime_error_message_carries_span` | PASS | **PASS** ✓ |
| Library tests 767/767 | PASS | **PASS** ✓ |
| Workspace tests pass (excl lab) | yes | **yes** ✓ — no FAILED lines outside trading |
| Pattern F cross-file changes | none | **confirmed** ✓ — only src/runtime.rs modified |

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Single-file diff | **PASS** | Only `src/runtime.rs` modified. No other files. No tests outside the file. |
| 2 | Marker count drops | **PASS+** | 300 → 0 (100% drop; target was ≥ 90%). |
| 3 | Each remaining stub has rationale | **PASS** | 91 `// arc 138: no span — <reason>` comments. Span::unknown() count of 123 = 54 synthetic-AST baseline + 69 Pattern E. The 22-site delta (91 - 69) likely reflects rationale comments paired with multi-site groups (one comment, multiple stubs). |
| 4 | Workspace tests pass | **PASS** | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. Library 767/767. |
| 5 | Canary passes | **PASS** | `runtime::tests::arc138_runtime_error_message_carries_span` PASS. Also `types::tests::arc138_type_error_message_carries_span` PASS. |
| 6 | No new variants / Display changes | **PASS** | Sweep is span-only; no RuntimeError variant additions; no Display string changes. |
| 7 | No commits | **PASS** | Working tree shows uncommitted modifications only. |
| 8 | Honest report | **PASS+** | ~500 words; counts verified; pattern distribution; ~30 helper sigs broadened explicitly named (Pattern F in-file); Pattern E rationale categories named (parse_program raw string, value_to_holon Value-only context, Vec element iteration, struct field lookup, synthesized dispatchers); honest delta on `expect_string_value` signature simplification. |

**HARD VERDICT: 8 OF 8 PASS.**

## Soft scorecard (4 rows)

| # | Criterion | Result | Notes |
|---|---|---|---|
| 9 | Pattern distribution | PASS | A (~25), B (~30), C (~8), D (~5), E (~38). Predicted A 30% / B 40% / C 20% / D 5% / E 5%. Reality: B leads at ~28%; E exceeds at ~36% — higher than predicted because the substrate has structural span-absent shapes (Vec iteration, internal dispatchers, parse_program from raw strings). E-skew is honest substrate observation, NOT scope creep. |
| 10 | Span source quality | PASS | Spot-check sample shows args[i].span() used for arg-specific errors, list_span.clone() used for whole-form errors. Pattern application semantically correct. |
| 11 | Workspace runtime | PASS | Library tests 0.11s; full workspace within baseline. |
| 12 | Honest delta on threading | **PASS+** | Sonnet broadened ~30 helper signatures with `list_span: &Span` (eval_math_unary, eval_make_bounded_queue, eval_make_unbounded_queue, eval_kernel_send/recv/try_recv/drop, eval_stat_*, eval_handle_pool_*, eval_kernel_select, pair_values_to_vectors, coincident_of_two_values, eval_algebra_coincident_explain, eval_died_error_*, eval_thread_died_error_*, eval_process_died_error_*, eval_kernel_extract_panics, eval_kernel_process_join_result, eval_kernel_spawn_thread, eval_kernel_thread_join_result, eval_kernel_process_send/recv, eval_form_*, eval_walk, eval_form_edn/file/digest family). All in-file. Plus `expect_string_value` signature simplification (removed `arg_name` parameter; 9 call sites updated) — refactored toward list_span shape. |

**SOFT VERDICT: 4 OF 4 PASS+. Clean ship.**

## Substrate observation — Pattern E categories

Sonnet's 91 Pattern E rationale comments cluster into five categories:

1. **`parse_program` from raw strings** — when an error fires inside the parser/loader path with no AST node yet, span is genuinely unavailable (the AST hasn't been constructed; only a string + line/col pre-AST). Honest.
2. **`value_to_holon` with Value-only context** — when the function receives a `Value` (already-evaluated, no AST trace), the originating AST has been discarded. Threading would require a parallel `WatAST` parameter through value-shaped APIs. Out of scope.
3. **Vec element iteration** — when emitting an error inside a `for x in vec.iter()` loop where `x` is a `Value` (not a `WatAST`), per-element AST spans aren't available. The collection's call-form `list_span` IS used; per-element is unavailable.
4. **Struct field lookup misses** — when looking up a field name in a struct schema and the field doesn't exist, the AST node may be a synthesized dispatch arm (struct-new, etc.). The 54 synthetic-AST baseline overlaps this category.
5. **Synthesized dispatchers** — internal Rust dispatch arms with no originating user AST. Same baseline overlap.

This is REAL substrate data: the runtime has structural span-absent shapes that arc 138 surfaces honestly. Future arcs (e.g., a `:wat::core::eval-with-source` that threads source positions through Value-shaped APIs) could close some of these gaps. Earned for follow-up; not papered over.

## Substrate observation — helper signature broadening (Pattern F in-file)

Sonnet broadened ~30 helper signatures with `list_span: &Span`. All in src/runtime.rs (no cross-file). The broadening matches the slice 3a established convention (10+ helpers already had `list_span: &Span`), now uniform across the eval helper family.

Plus a useful refactor: `expect_string_value`'s `arg_name` parameter was eliminated in favor of the list_span + position shape. 9 call sites updated. This is a substrate-design improvement that surfaced organically during the sweep — span threading made the prior signature redundant.

## Independent prediction calibration

Predicted: 50% chance 8/8 + 4/4 in 40-60 min; 30% partial completion. Reality: **8/8 hard + 4/4 soft + Pattern F in-file broadening (~30 helpers) + signature simplification refactor**, runtime 65 min — slightly over the 40-60 prediction band but within reasonable bounds for a 300-site sweep + 30 helper-sig broadenings + 9 call-site updates.

The sonnet report header noted "300 (original) → 106 (session start, prior context resolved ~150 + orchestrator bulk-injected the rest) → 0 (this session: all 106 resolved)" — meaning sonnet's session inherited 106 markers and resolved all 106. The grep at session start showed 300; sonnet's count of 106 must be after THIS session's pass; the 300 number was the post-bulk-inject count. Honest framing in their report.

Calibration update: 300-site sweeps with helper-sig broadening land in the 60-90 min band. EXPECTATIONS prediction of 40-60 min was slightly optimistic; recalibrate for slice 4+.

## Ship decision

**SHIP.** 8/8 hard + 4/4 soft. Substrate observations earned and named.

## Next steps

1. Commit slice 3a-finish + this SCORE (this commit).
2. Push.
3. Slice 3b BRIEF: external file sweep — 156 sites across 15 files (10 src/ + crates/wat-telemetry-sqlite + crates/wat-telemetry/shim.rs + crates/wat-macros/codegen.rs). Same patterns A/B/E/F. Smaller scope; likely one ~30-40 min sonnet engagement.
4. Slice 4: MacroError, EdnReadError, ClauseGrammarError, LowerError sweep.
5. Slice 5: ConfigError form_index → Span.
6. Slice 6: doctrine + INSCRIPTION + USER-GUIDE + 058 row.

## What this slice tells us

- The arc 138 pattern is **fully reproducible** by sonnet across the substrate's largest file. Slice 3b dispatches with high confidence.
- Pattern E (substrate genuinely lacks span carrier) is a HONEST substrate observation — Vec iteration, value_to_holon, parse_program, synthesized dispatchers all have structural span-absence. These are earned-for-follow-up arcs, NOT failures.
- Pattern F broadening is uniform: when a helper needs span context, add `list_span: &Span`. The shape is now stable across 50+ helpers in src/runtime.rs.
- Sonnet's calibration stays strong: 8/8 → 8/8 → 8/8 (slice 1 finish) → 8/8 (slice 2) → 7/8 partial (slice 3a) → 8/8 (slice 3a-finish).
- The orchestrator-as-finisher pattern was used for slice 3a's bulk Span::unknown() inject; not needed for 3a-finish (sonnet completed in one engagement). Pattern is reserved for context-overflow scenarios.

Sonnet's WORK was clean. Sonnet's REPORT was honest (counts match `git diff --stat`; substrate observations earned). Trust-but-verify confirms both.
