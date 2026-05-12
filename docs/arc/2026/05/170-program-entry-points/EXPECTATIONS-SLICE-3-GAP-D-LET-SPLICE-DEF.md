# Arc 170 slice 3 — Gap D EXPECTATIONS (sonnet scorecard)

**Mirror of Gap C V2 for `let`.** Sub-15-LOC substrate change + 3 probes.

## Independent prediction

**Runtime band:** 30-60 min sonnet. Pattern is established (Gap C V2 ran in ~10 min).

**Hard cap:** 120 min.

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `register_defines` extended with `let` arm | grep |
| B | `register_stdlib_defines` extended with `let` arm | grep |
| C | All three probes in `tests/probe_let_splice_def.rs` pass | cargo test |
| D | Workspace at 0 failed | full cargo test |
| E | `cargo check --release` green | clean |
| F | SCORE documents impl + `let*` gap status (surface only, don't fix) | manual review |

## Implementation approach

1. Write `tests/probe_let_splice_def.rs` with the three probes from the BRIEF (failing baseline)
2. Add `let` arm to `register_defines` mirroring Gap C V2's `do` arm shape; recurse into `items[2..]` (let body per arc 168 multi-form body) — mirror the `collect_splice_defs_ctx` let arm at check.rs:6853
3. Add the matching arm to `register_stdlib_defines`
4. Helper: generalize `preregister_fn_defs_in_do` to handle both `do` and `let` OR add a `preregister_fn_defs_in_let`. Sonnet picks.
5. Verify probes pass + workspace stays green
6. Grep for `let*` in `collect_splice_defs_ctx` to surface whether the same gap exists for `let*`; report in SCORE; DO NOT FIX

## What sonnet produces

- `src/runtime.rs` modified (`let` arm in two functions + helper)
- `tests/probe_let_splice_def.rs` (3 probes; committed as regression suite)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-D-LET-SPLICE-DEF.md`
- Do NOT commit; orchestrator atomic-commits

## Constraints (hard)

- DO NOT commit
- DO NOT touch deftest / deftest-hermetic, Layer 1/2 macros, run-sandboxed-*, BareLegacy* walker, spawn.rs, Process struct
- DO NOT fix `let*` parallel gap (surface only)
- DO NOT use deferral language in SCORE
- Workspace must stay at 0 failed

## Expected delta

- Baseline: 2202 / 0
- Post Gap D: 2205 / 0 (+3 probes)

## Honest deltas (anticipated)

1. Helper generalization vs duplication choice
2. `let*` gap status
3. Anything unexpected
