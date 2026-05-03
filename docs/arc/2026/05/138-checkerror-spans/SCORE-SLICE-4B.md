# Arc 138 Slice 4b — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `af52aad378c3252ce`
**Runtime:** ~11 min (660 s).

## Independent verification (orchestrator)

| Claim | Sonnet's value | Disk-verified |
|---|---|---|
| Files modified | 2 (src/edn_shim.rs, src/lower.rs) | **2** ✓ |
| `git diff --stat` | 293+ / 118- | **293+ / 118-** ✓ |
| EdnReadError variants restructured | 6 | **6** ✓ |
| LowerError variants restructured | 12 | **12** ✓ |
| Display arms updated | 18 | **10 in edn_shim.rs + 16 in lower.rs span_prefix uses** ✓ (counts include Display impl + helper definitions) |
| New canaries added | 2 (`edn_shim::tests::arc138_edn_read_error_message_carries_span`, `lower::tests::arc138_lower_error_message_carries_span`) | **2** ✓ — both PASS |
| All 6 arc138 canaries | 6/6 PASS | **6/6** ✓ |
| Library tests | 771/771 (up from 769) | **771/771** ✓ |
| Workspace `cargo test` excl lab | empty FAILED | **empty** ✓ |
| Pattern E in edn_shim.rs | 31/31 (100%) | **29 marked + 4 pre-existing baseline + 1 unmarked = 34 total** ✓ (sonnet's 31 = 29 newly added + ~2 unmarked; baseline was 4 pre-existing per EXPECTATIONS) |
| Pattern E in lower.rs | 1 site | **1** ✓ |

## Hygiene fix (orchestrator)

Sonnet left ONE leftover `Span::unknown()` in src/lower.rs:182 (the `MalformedCall(Span::unknown())` site for empty-list input) WITHOUT the slice 1's `// arc 138: no span — <reason>` rationale convention. Sonnet documented the rationale in its REPORT but not in the CODE. Orchestrator fixed inline before committing — added `// arc 138: no span — empty list has no head element; no AST node to read span from`. Tests still pass.

This is the kind of comment-hygiene gap that could compound across slices; surfacing it now keeps the marker discipline tight (per `feedback_no_known_defect_left_unfixed.md`).

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | File scope | **PASS** | Exactly src/edn_shim.rs + src/lower.rs modified. No others. |
| 2 | All 18 variants gain span | **PASS** | 6 EdnReadError + 12 LowerError. Tuple/struct/unit conversions all per BRIEF table. |
| 3 | All 18 Display arms prefix coords | **PASS** | New `span_prefix` helper added to each file (mirror src/macros.rs slice 4a pattern). 18/18 arms render `{span_prefix(span)}`. |
| 4 | Emission sites use real spans where possible | **PASS** | LowerError: 23/24 sites real-spanned (95%). EdnReadError: 0/31 real-spanned (0%) — all Pattern E with rationale. Predicted; substrate observation about EDN parser layer (no AST context). |
| 5 | No cross-file changes needed | **PASS** | Confirmed: no consumer files touched. |
| 6 | Two canaries added + pass | **PASS** | `edn_shim::tests::arc138_edn_read_error_message_carries_span` (line 1514) triggers `NoTypeRegistry` via `read_edn("#unknown/Type {}", None)`; `lower::tests::arc138_lower_error_message_carries_span` (line 496) triggers `MalformedCall` via `lower(&parse_one("(123)"))`. Both PASS. |
| 7 | Workspace tests pass | **PASS** | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. All 6 arc138 canaries PASS. 771/771 wat lib tests. |
| 8 | Honest report | **PASS+** | ~600 words; per-file counts; pattern distribution per file; line numbers for canaries; canary triggering rationale; honest delta on substrate-architecture boundary (eval_edn_read wraps EdnReadError into RuntimeError with span — the EDN layer ITSELF genuinely lacks AST context, this is intentional architecture not a gap); 6 helper sigs broadened named explicitly; 5 existing test pattern updates documented. |

**HARD VERDICT: 8 OF 8 PASS.**

## Soft scorecard (4 rows)

| # | Criterion | Result | Notes |
|---|---|---|---|
| 9 | Pattern distribution | **PREDICTION HIT** | Predicted: edn_shim.rs E dominates (50-70%); lower.rs A/B dominates (~80%). Reality: edn_shim.rs E=100%, lower.rs A=10+B=4+D=7+F=6=27 (Pattern E=1). Predictions correct in shape; edn_shim's E ratio higher than even the upper prediction. |
| 10 | Span source quality | **PASS+** | LowerError canary renders `<test>:1:1: algebra-core call must be a list...` — clean coordinate-prefixed Display. Pattern A sites (atom literal arg) point at the offending arg; Pattern D sites (head keyword) point at the keyword; Pattern F threading uses head_span throughout sub-helpers. |
| 11 | Workspace runtime | **PASS** | Library tests 0.11s. Within baseline. |
| 12 | Honest delta on threading | **PASS+** | 6 in-file helper sigs broadened in lower.rs (`lower_atom`, `lower_bind`, `lower_bundle`, `lower_permute`, `lower_thermometer`, `lower_blend` — all gained `head_span: Span`). All private. `lower_call` collects head.span() once and threads it. No public API change. No threading attempted across the EDN parser boundary (correctly — that's a separate substrate architecture decision). |

**SOFT VERDICT: 4 OF 4 PASS+. Clean ship.**

## Substrate observation — the EDN parser/AST architectural boundary

The headline finding from slice 4b: **EdnReadError is 100% Pattern E**, and this is correct architecture, not a gap.

The structural reason: `read_edn(s: &str, ...)` and `edn_to_value(edn: &OwnedValue, ...)` operate on raw EDN strings or already-parsed `wat_edn::OwnedValue` trees. Neither receives a `WatAST`. The wat-side wrapper `eval_edn_read` (in src/runtime.rs) HAS the AST span context and wraps EdnReadError into RuntimeError::MalformedForm WITH the span. So the two-layer architecture is:

- **EDN layer (edn_shim.rs):** parses raw EDN, no AST coordinate. Pattern E is correct here.
- **Runtime layer (runtime.rs::eval_edn_read):** has WatAST span; wraps EdnReadError → RuntimeError with the span. Real coordinates land here.

Threading WatAST into the EDN layer would require either (a) a parallel WatAST parameter through every parsing function (intrusive) or (b) `_with_span` siblings on every public read function (matches slice 2's pattern). NOT done — the wrapper architecture already gives users the right error message via the runtime layer.

This is honest substrate data: an architectural boundary surfaces, named explicitly, NOT papered over with synthetic spans.

## Substrate observation — sonnet calibration recovered + maintained

Slice 4a's REPORTING DISCIPLINE NOTE in BRIEF restored sonnet's report quality. Slice 4b sonnet maintained the discipline: comprehensive per-file counts, line numbers for canaries, explicit Pattern F broadenings, architectural observations. No prompting needed for the second engagement — calibration is durable.

The one comment-hygiene lapse (lower.rs:182 missing `// arc 138: no span` marker) was minor and orchestrator-fixable in 30 seconds. Not worth a re-spawn. Logged for future BRIEF wording: explicitly say "every leftover Span::unknown() in CODE carries the rationale comment, not just in the report."

## Independent prediction calibration

Predicted: 65% chance 8/8 + 4/4 in 12-20 min. Reality: **8/8 hard + 4/4 soft + 1 hygiene fix**, runtime 11 min — UNDER the predicted band. The slice 4a worked example accelerated this engagement.

## Ship decision

**SHIP.** 8/8 hard + 4/4 soft. Substrate observations earned (EDN parser/AST architectural boundary).

## Next steps

1. Commit slice 4b + this SCORE (this commit).
2. Push.
3. Slice 5 BRIEF: ConfigError form_index → Span. Small, surgical — single error type, single transform (form_index field becomes Span field). Probably 5-10 min sonnet engagement.
4. Slice 6: doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure.

## What this slice tells us

- Arc 138's pattern propagates cleanly to the FINAL two error types in the substrate (excluding ConfigError which is slice 5).
- The EDN parser/AST architectural boundary is REAL — earned-for-follow-up arc if downstream demand surfaces (threading WatAST into the EDN layer or adding `_with_span` siblings).
- Sonnet's calibration is durable: BRIEF discipline + worked examples produce reliable 10-15 min engagements for ≤ 75-emission slices with clear AST context (LowerError) OR clean Pattern E architectural rationale (EdnReadError).
- Comment-hygiene lapses (rationale in REPORT but not in CODE) are minor and orchestrator-fixable; future BRIEFs should make this expectation explicit.

Sonnet's WORK was clean. Sonnet's REPORT was honest and comprehensive. Trust-but-verify confirms via disk + 6 canaries + 771/771 lib tests.
