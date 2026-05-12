# Arc 170 slice 3 Gap H EXPECTATIONS (sonnet scorecard)

**One spawn.** A-wide substrate fix: extract_closure lifts fn-body prelude forms (define/struct/enum at do's prefix) into the closure's prologue. After Gap H ships, Path E shape becomes possible.

## Independent prediction

**Runtime band:** 45-90 min sonnet.

**Hard cap:** 180 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `extract_closure` extended with prelude-lift sweep (in src/closure_extract.rs) | grep + read |
| B | `is_fn_body_do` + `split_prelude_prefix` helper fns exist | grep + read |
| C | 5+ probes pass: define / struct / enum / mixed / prefix-terminating semantics | cargo test |
| D | All 25 prior substrate probes still pass (Gap C V2 + D + E + F-1 + F-3 + F-2 + G) | cargo test |
| E | `cargo check --release` green; workspace at 2227 + N / 0 failed (N ≥ 5) | full test run |
| F | F-3's whole-registry type-registry sweep still functions; F-3 probes pass | F-3 probe re-run |

**6 rows.** All must PASS.

## Implementation approach

1. **Audit extract_closure** (5-10 min): map current body capture + F-3's type-registry sweep
2. **Probes** (15-20 min): 5+ probes confirming failure baseline
3. **Helper fns** (10 min): `is_fn_body_do` + `split_prelude_prefix`
4. **Lift sweep** (15 min): integrate into extract_closure after F-3's type sweep
5. **Verify** (15-30 min): probes + workspace + F-3 regression check

## What sonnet produces

- `src/closure_extract.rs` modified
- New probe test file
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-H-PRELUDE-LIFT-TO-PROLOGUE.md` with:
  - 6-row scorecard
  - Prelude-prefix-termination rationale
  - Prologue ordering analysis
  - F-3 sweep interaction
  - Body-shape edge case coverage
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify `eval()` / `eval_do_tail` — substrate keeps rejecting define-at-expression; fix is upstream
- Modify deftest-hermetic macro body (separate slice)
- Modify any test call site outside the new probe file
- Touch `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Use --no-verify or skip hooks
- Break existing deftest-hermetic users (they use forms-quoted prelude → run-sandboxed-hermetic-ast, NOT the new closure path; lift should be transparent for them)

## Verification commands

```bash
# New Gap H probes
cargo test --release --test probe_closure_body_prelude_lift 2>&1 | tail -5

# Regression — all 25 prior substrate probes
cargo test --release --test probe_do_splice_def 2>&1 | tail -3
cargo test --release --test probe_let_splice_def 2>&1 | tail -3
cargo test --release --test probe_do_splice_define 2>&1 | tail -3
cargo test --release --test probe_let_splice_define 2>&1 | tail -3
cargo test --release --test probe_do_splice_struct 2>&1 | tail -3
cargo test --release --test probe_do_splice_enum 2>&1 | tail -3
cargo test --release --test probe_let_splice_struct 2>&1 | tail -3
cargo test --release --test probe_let_splice_enum 2>&1 | tail -3
cargo test --release --test probe_spawn_process_parent_type 2>&1 | tail -3
cargo test --release --test probe_resolver_quote_awareness 2>&1 | tail -3
cargo test --release --test probe_deftest_hermetic_isolation 2>&1 | tail -3

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline (post-Gap-G): 2227 passed / 0 failed
- Post-Gap-H: 2227 + N passed / 0 failed (N = new probes; expect 5+)

## Honest delta categories (anticipated)

1. **Prelude-prefix-termination** — first non-prelude form? Or some other rule? Surface choice + rationale
2. **Prologue ordering** — type-registry (F-3) before lifted-defines? After? Topological needs.
3. **F-3 interaction** — share a helper or stay distinct sweeps?
4. **Body shape edge cases** — non-do body, let-containing-defines body, nested fn bodies, etc.
5. **Anything unexpected**
