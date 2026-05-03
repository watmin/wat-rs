# Arc 138 Slice 4b — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-SLICE-4B.md`
**Targets:** src/edn_shim.rs (primary), src/lower.rs (primary). NO cross-file consumers.

## Setup — workspace state pre-spawn

- Baseline (commit `3b999c2`): wat-rs workspace `cargo test --release --workspace` exit=0 excluding lab. Slice 4a shipped (MacroError + ClauseGrammarError).
- Pre-sweep `Span::unknown()` in src/edn_shim.rs: 4 (pre-existing in test code, leftover from slice 3b's thread-through).
- Pre-sweep `Span::unknown()` in src/lower.rs: 0 (variants don't carry span yet).
- EdnReadError: 6 variants, ~31 emission sites in src/edn_shim.rs.
- LowerError: 12 variants, ~42 emission sites in src/lower.rs.
- No cross-file consumers — both error types are only re-exported via src/lib.rs; pattern-match destructuring lives only in their own files (and tests).
- Existing canaries pass: `runtime::tests::arc138_runtime_error_message_carries_span`, `types::tests::arc138_type_error_message_carries_span`, `macros::tests::arc138_macro_error_message_carries_span`, `form_match::tests::arc138_clause_grammar_error_message_carries_span`. 4/4 PASS at baseline.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | src/edn_shim.rs + src/lower.rs modified. No others. |
| 2 | All 18 variants gain span | 6 EdnReadError + 12 LowerError. Tuple variants extended; struct variants gain field; unit variants converted to tuple. |
| 3 | All 18 Display arms prefix coords | Local `span_prefix` helper added to each of edn_shim.rs / lower.rs (mirror src/types.rs / src/macros.rs slice 4a pattern). 18/18 arms render `{span_prefix(span)}`. |
| 4 | Emission sites use real spans where possible | LowerError: ≥ 90% real-spanned (AST in scope throughout `lower`). EdnReadError: Pattern E may dominate (no AST context when parsing raw EDN). Each leftover Span::unknown() carries `// arc 138: no span — <reason>`. |
| 5 | No cross-file changes needed | Confirmed by inventory: lib.rs re-exports only. |
| 6 | Two canaries added + pass | One for EdnReadError, one for LowerError. Each triggers a representative variant and asserts `<test>:` substring in rendered Display. Both pass. |
| 7 | Workspace tests pass | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. All 6 canaries (4 existing + 2 new) PASS. |
| 8 | Honest report | ~400 words; per-file counts; variant list; Display arm count; pattern distribution per file; canary names + line numbers; diff stat; honest deltas; four questions. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Pattern distribution | edn_shim.rs: E expected to dominate (~50-70%) — raw EDN parsing has no WatAST. lower.rs: A/B expected to dominate (~80%) — AST in scope throughout `lower`. |
| 10 | Span source quality | LowerError sites point at offending forms (e.g., the malformed Atom call's args[0], the BindArity call's list_span). EdnReadError sites use whatever upstream span exists (likely Pattern E). |
| 11 | Workspace runtime | `cargo test --release --workspace` runtime ≤ baseline + 5%. |
| 12 | Honest delta on threading | If sonnet broadens any helper sig with `span: Span`: count and name them. EdnReadError's `edn_to_value` may need broadening if Pattern F is preferred over Pattern E. |

## Independent prediction

Slice 4a calibration: 16 variants + 50 sites + 3 files + cross-file consumer = 10 min. Slice 4b: 18 variants + 73 sites + 2 files + NO cross-file consumer.

- **Most likely (~65%):** 8/8 hard + 4/4 soft. Sonnet ships in 12-20 min. LowerError mostly real-spanned; EdnReadError mostly Pattern E.
- **Pattern E ratio higher than expected for EdnReadError (~20%):** the raw-EDN-parse context genuinely lacks AST. Predicted; not a failure. Substrate observation about EDN parser/AST disconnect documented.
- **Helper-sig broadening (~10%):** `edn_to_value` or `lower_call` gain `span: Span` parameter. All in-file. Hard 8/8 + soft 12 PASS+.
- **Test regression in canary (~5%):** the wat snippet sonnet uses doesn't compile or trigger expected variant. Sonnet investigates; iterates.
- **Test pattern updates (~5%):** existing tests in edn_shim.rs/lower.rs use destructure patterns that need updating for new variant shapes. Sonnet handles mechanically (slice 4a precedent).

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → 2 files modified (src/edn_shim.rs, src/lower.rs).
3. `grep -c "span: Span" src/edn_shim.rs` and `src/lower.rs` → 6 + 12 variant-field counts.
4. `grep -c "span_prefix" src/edn_shim.rs` and `src/lower.rs` → ≥ 6 + 12 (18) Display arm uses.
5. `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` → empty.
6. Run all 6 canaries by name (4 existing + 2 new).
7. Spot-check 5-10 emission sites for pattern correctness.
8. Score each row; write `SCORE-SLICE-4B.md`.
9. If clean → commit + push, queue slice 5 (ConfigError form_index → Span — small, surgical).

## What this slice tells us

- All clean → arc 138's pattern propagates cleanly to two MORE error types. Slice 5 (ConfigError) dispatches with confidence.
- High Pattern E in EdnReadError → real substrate observation about EDN parser/AST disconnect (the `wat_edn` parser doesn't emit WatAST; spans only emerge when EDN is embedded in wat source — earned-for-follow-up).
- Helper-sig broadening surfaces → calibration data for substrate.

## What follows

- Score → commit slice 4b → write slice 5 BRIEF (ConfigError form_index → Span; small, surgical).
- Spawn sonnet → score → continue to slice 6 (doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure).
