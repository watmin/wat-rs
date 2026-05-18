# Arc 212 stone δ-bare-primitives — EXPECTATIONS

## Independent prediction

- **Runtime band:** 5-15 min Mode A. Mechanical migration (~10 line edit), three short cargo invocations.
- **LOC changed:** ~15 (delete 8 lines of duplicated List+Vector recursion, add 4 lines of children() call + comment)
- **New files:** 1 (SCORE-212-DELTA-BARE-PRIMITIVES.md)
- **Surprises expected:** 0-1 (a test invocation typo; an unexpected Keyword-arm subtlety the migration must preserve)

## Honest-delta watch

1. **The Keyword arm has a subtle control-flow detail.** Each diagnostic check has `return;` after `errors.push(...)` — that early-return prevents the type-expression check from firing on a known-legacy keyword. The migration preserves this: copy verbatim including the early-returns, then add a final `return;` at the end of the `if let WatAST::Keyword` block so Keyword nodes don't fall through to `node.children()` recursion (which would be a no-op anyway since children() returns &[] for Keyword, but explicit return is clearer + faster).

2. **A test passes pre-migration but fails post.** That's the substrate teaching that something in the migration is wrong. STOP-trigger 1 fires: revert + report which test broke. Do not theorize.

3. **A test that wasn't in the named three fails post-migration.** STOP-trigger 3 fires: workspace failure count is not your concern; do not investigate. Return.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `walk_for_bare_primitives` uses `node.children()` for recursion | YES |
| 2 | Keyword arm body preserved verbatim (every line, every early-return) | YES |
| 3 | `cargo test --release --test wat_arc154_kill_let_star` green | YES |
| 4 | `cargo test --release --test wat_arc153_nil_rename` green | YES |
| 5 | `cargo test --release --test wat_arc155_fn_rename` green | YES |
| 6 | `cargo build --release` clean | YES |
| 7 | SCORE file written at named path | YES |
| 8 | Zero other code edits anywhere in repo | YES |

## Mode classification

- **Mode A:** all eight criteria satisfied; SCORE clean.
- **Mode B:** migration applied; one or more named tests fail; sonnet REVERTED + inscribed honestly which test broke. Mode B is acceptable failure-engineering data — tells the orchestrator the walker rule needs a subtler change than mechanical migration.
- **Mode C:** STOP rule broken (touched another walker; "improved" Keyword logic; investigated unrelated failures).

## Calibration metadata

- **Orchestrator confidence:** VERY HIGH. The walker already has explicit Vector arm; the migration is collapsing duplicated recursion into a single `children()` call. The risk surface is minimal: preserve the Keyword arm verbatim, add the early return, route recursion through children().
- **Risk factors:** the Keyword arm's early-return pattern (one per diagnostic check); ensuring the `return` at end of `if let` block doesn't accidentally fall through to children() recursion for Keyword (which is a no-op but worth being explicit about).
- **Why this matters:** δ-bare-primitives is the SIMPLEST L1 stone. It validates the per-stone trust gate before larger stones (δ-refuse-mutation, ζ-newtype-wall) ship. If this one delivers Mode A clean, the discipline scales to the remaining 5-7 stones in the L1 phase.

## Stone-discipline validation

This stone validates the post-halt-reframe BRIEF shape:
- ONE walker named
- THREE wat-tests as proof gates (precise, named)
- STOP triggers VERBATIM (5 of them)
- NO "workspace failure count" framing
- Constraints prohibit scope-creep explicitly

If sonnet delivers Mode A, the discipline scales. If sonnet delivers Mode B, the stone discipline still held (honest stop + clean report) — only the empirical migration needs reconsideration. If sonnet delivers Mode C, the BRIEF needs tightening further before next stones.

## Cross-references

- Arc 212 DESIGN § "Locked stone chain (L0 → L4 trajectory)" — where δ-bare-primitives sits
- SCORE-212-GAMMA-1-AUDIT-CATALOG.md — the audit that catalogued this walker as the BRIEF-known pending one
- INTERSTITIAL § 2026-05-18 (post-compaction, mid-arc-212) — the stone discipline this stone proves operationally
- BRIEF-212-DELTA-BARE-PRIMITIVES.md — the brief itself
