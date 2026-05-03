# Arc 138 Slice 4a — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-SLICE-4A.md`
**Targets:** src/macros.rs (primary), src/form_match.rs (primary), src/check.rs (cross-file consumer)

## Setup — workspace state pre-spawn

- Baseline (commit `2fb3ef9`): wat-rs workspace `cargo test --release --workspace` exit=0 excluding lab. Slice 3b shipped — arc 138 RuntimeError sweep complete.
- Pre-sweep `Span::unknown()` in src/macros.rs: 0 (variants don't carry span yet).
- Pre-sweep `Span::unknown()` in src/form_match.rs: 0 (variants don't carry span yet).
- MacroError: 9 variants, ~37 emission sites in src/macros.rs.
- ClauseGrammarError: 7 variants, ~13 emission sites in src/form_match.rs.
- Cross-file consumers: src/freeze.rs (wraps MacroError, no destructure — invisible to restructure); src/runtime.rs (doc reference only); src/check.rs::grammar_error_to_check_error (destructures all 7 ClauseGrammarError variants — needs pattern updates).
- Existing canaries pass: `runtime::tests::arc138_runtime_error_message_carries_span`, `types::tests::arc138_type_error_message_carries_span`.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | src/macros.rs + src/form_match.rs + src/check.rs modified. No others. |
| 2 | All 16 variants gain span | 9 MacroError + 7 ClauseGrammarError. Tuple variants extended; struct variants gain field; unit variants converted to tuple. |
| 3 | All 16 Display arms prefix coords | Local `span_prefix` helper added to each of macros.rs / form_match.rs (mirror src/types.rs slice 2 pattern). 16/16 arms render `{span_prefix(span)}`. |
| 4 | Emission sites use real spans | ≥ 90% of ~50 sites use real spans. Each leftover Span::unknown() carries `// arc 138: no span — <reason>`. |
| 5 | Cross-file consumer updated | `grammar_error_to_check_error` patterns updated for all 7 ClauseGrammarError variants. check.rs compiles. |
| 6 | Two canaries added + pass | One for MacroError, one for ClauseGrammarError. Each triggers a representative variant and asserts `<test>:` substring in rendered Display. Both pass. |
| 7 | Workspace tests pass | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. All 4 canaries (2 existing + 2 new) PASS. |
| 8 | Honest report | ~400 words; counts; variant list; Display arm count; pattern distribution; cross-file consumer update; canary names; diff stat; honest deltas; four questions. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Pattern distribution | A (~50%); B (~30%); D (~10%); E (~5%); F (~5%). MacroError emission sites mostly in expand_form / expand_once with WatAST in scope; form_match.rs has ast param everywhere. |
| 10 | Span source quality | Spans point at the OFFENDING node (the malformed defmacro form, the unknown clause head, etc.), not always reaching for the enclosing function's span. |
| 11 | Workspace runtime | `cargo test --release --workspace` runtime ≤ baseline + 5%. |
| 12 | Honest delta on threading | If sonnet broadens any helper sig with `span: Span`: count and name them. If sonnet uses `_with_span` sibling-API pattern (slice 2's invention): name it. |

## Independent prediction

Smaller than slice 3a (123 emissions vs 489); but introduces variant restructuring + Display + emission + cross-file consumer pattern updates + 2 canaries — closer to "two slice 2's" in shape.

- **Most likely (~60%):** 8/8 hard + 4/4 soft. Sonnet ships in 25-40 min. ≥ 45 of 50 sites real-spanned. Canaries land on first try. check.rs patterns updated mechanically.
- **Helper-sig broadening (~15%):** sonnet adds `span: Span` to 2-4 helper functions in src/macros.rs (e.g., expand_template, walk_template). All in-file. Hard 8/8 + soft 12 PASS+.
- **Test regression in canary (~10%):** the wat snippet sonnet uses to trigger a variant doesn't compile or doesn't trigger the expected variant. Sonnet investigates; iterates.
- **check.rs pattern miss (~10%):** sonnet misses a destructure pattern in `grammar_error_to_check_error`; cargo build catches; quick fix.
- **Cross-file regression (~5%):** sonnet accidentally edits freeze.rs/runtime.rs (the no-touch cross-file consumers); cargo build catches; revert.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → 3 files modified (src/macros.rs, src/form_match.rs, src/check.rs).
3. `grep -c "span: Span" src/macros.rs` and `src/form_match.rs` → 9 + 7 = 16 variant-field counts (plus emission usage).
4. `grep -c "span_prefix" src/macros.rs` and `src/form_match.rs` → ≥ 9 + 7 (16) Display arm uses.
5. `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` → empty.
6. Run all 4 canaries by name (2 existing + 2 new).
7. Spot-check 5-10 emission sites for pattern correctness.
8. Score each row; write `SCORE-SLICE-4A.md`.
9. If clean → commit + push, queue slice 4b (EdnReadError + LowerError).

## What this slice tells us

- All clean → arc 138's pattern propagates to two MORE error types in one engagement. Slice 4b dispatches with confidence.
- Pattern E ratio matches prediction → small (<10%); these error types have clear AST context.
- Helper-sig broadening surfaces → real substrate observation about macro/form_match helper threading depth.

## What follows

- Score → commit slice 4a → write slice 4b BRIEF (EdnReadError 31 sites + LowerError 42 sites; no cross-file consumers).
- Spawn sonnet → score → continue to slice 5 (ConfigError) → slice 6 (doctrine + INSCRIPTION + USER-GUIDE + 058 row).
