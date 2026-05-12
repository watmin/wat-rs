# Arc 170 slice 3 Gap I-A EXPECTATIONS (sonnet scorecard)

**One spawn.** Mint `is_declaration_form` in `src/freeze.rs` as the source-of-truth predicate for the 8 declaration forms; route the prelude lift through it; retire `is_prelude_form`. Closes the drift Gap H left.

## Independent prediction

**Runtime band:** 30-60 min sonnet.

**Hard cap:** 120 min (2×). ScheduleWakeup at T+7200s.

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `pub fn is_declaration_form` minted in `src/freeze.rs` adjacent to `is_mutation_form`; covers exactly the 8 declaration keywords (def, define, defmacro, define-dispatch, struct, enum, newtype, typealias); no loads, no config setters | grep + read |
| B | `is_prelude_form` in `src/closure_extract.rs` retired (deleted, NOT kept as wrapper); `split_body_prelude` consumes `is_declaration_form` via head-keyword extraction | grep + read |
| C | 6+ probes in `tests/probe_declaration_form_lift.rs` pass (def / defmacro / define-dispatch / newtype / typealias / mixed) | cargo test |
| D | All 5 Gap H probes (`probe_closure_body_prelude_lift`) still pass — regression confirms `is_declaration_form` covers define/struct/enum identically to retired `is_prelude_form` | cargo test |
| E | All 11 prior substrate probes still pass: do_splice_def/define/struct/enum, let_splice_def/define/struct/enum, spawn_process_parent_type, resolver_quote_awareness, deftest_hermetic_isolation | cargo test |
| F | `cargo check --release` green; workspace at 2232 + N passed / 0 failed (N ≥ 6 new probes) | full test run |

**6 rows.** All must PASS.

## Implementation approach (sonnet's path)

1. **Mint the predicate** (5-10 min) — `pub fn is_declaration_form` in src/freeze.rs after line 1269; docstring naming the two callers (closure_extract for I-A; check::validate_def_position_with_wrapper for I-B future-slice)
2. **Route the lift** (5-10 min) — retire `is_prelude_form` (delete lines 1762-1775); rewrite `split_body_prelude`'s `take_while` closure to consume `crate::freeze::is_declaration_form` via head-keyword extraction. Optional inline closure vs factored `head_keyword` helper — surface the choice
3. **Probes** (15-25 min) — 6+ probes; positive-case-only is fine (Gap H's probe set is the failing-baseline precedent — those probes prove the mechanism; I-A's probes prove additional coverage)
4. **Verify** (10-15 min) — Gap H regression probes + 11 prior probes + full workspace

## What sonnet produces

- `src/freeze.rs` modified (predicate addition)
- `src/closure_extract.rs` modified (`is_prelude_form` retired; lift routes through new predicate)
- New probe file `tests/probe_declaration_form_lift.rs`
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-I-A-IS-DECLARATION-FORM.md` with:
  - 6-row scorecard with PASS/FAIL each row
  - Head-keyword extraction choice rationale
  - `is_prelude_form` retirement strategy rationale (delete vs wrapper)
  - Probe-shape rationale (failing-baseline vs positive-only)
  - `defn`-absence docstring decision
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify `is_mutation_form` itself — it stays as the union over three categories
- Modify `refuse_mutation_forms` — its scope is the union; I-A doesn't affect it
- Extend `validate_def_position_with_wrapper` — that's Gap I-B's job
- Add loads (`load-file!` family) or config setters (`config::set-*`) to `is_declaration_form` — out-of-scope by architectural intent
- Modify error variants (`DefNotTopLevel` / `DefineInExpressionPosition`) — Gap I-A is purely additive
- Modify deftest-hermetic macro shape — separate slice
- Touch `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Use --no-verify or skip hooks
- If a Gap H regression surfaces (any of the 5 prior probes fails), STOP and report — the routing through `is_declaration_form` must be transparent for the original 3 forms

## Verification commands

```bash
# New Gap I-A probes
cargo test --release --test probe_declaration_form_lift 2>&1 | tail -10

# Gap H regression (CRITICAL — load-bearing for I-A correctness)
cargo test --release --test probe_closure_body_prelude_lift 2>&1 | tail -10

# All 11 prior substrate probes
cargo test --release \
  --test probe_do_splice_def --test probe_let_splice_def \
  --test probe_do_splice_define --test probe_let_splice_define \
  --test probe_do_splice_struct --test probe_do_splice_enum \
  --test probe_let_splice_struct --test probe_let_splice_enum \
  --test probe_spawn_process_parent_type \
  --test probe_resolver_quote_awareness \
  --test probe_deftest_hermetic_isolation 2>&1 | tail -5

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline (post-Gap-H, commit `36030c3`): **2232 passed / 0 failed**
- Post-Gap-I-A: **2232 + N passed / 0 failed** (N = new probes; expect ≥ 6)

If failed > 0 after I-A's changes, EITHER:
- Regression in Gap H's lift mechanic (the routing broke the original 3 forms) → STOP and report
- Probe failure in I-A's own new tests → expected during iteration; sonnet fixes; final report must be 0 failed
- Cascade from elsewhere → STOP and report; do NOT proceed

## Honest delta categories (anticipated)

1. **Head-keyword extraction shape** — inline closure vs factored `head_keyword(&WatAST) -> Option<&str>` helper? Pattern recurs; surface choice + rationale.
2. **`is_prelude_form` retirement strategy** — delete entirely vs keep as thin wrapper? Doctrine answer: delete (no aliases; one source-of-truth). Confirm.
3. **Probe shape** — positive-case-only (recommended; Gap H's failing-baseline precedent stands) vs before/after pair (more thorough but doubles probe count). Surface choice.
4. **`defn` absence documentation** — should `is_declaration_form`'s docstring note WHY `defn` is intentionally absent (macro that expands to `def`)? Recommendation: yes; future readers will ask.
5. **Anything unexpected** — particularly around head-keyword extraction edge cases (List with non-Keyword head; nested forms with declaration-form heads inside non-do bodies; etc.)

## Constraints — orchestrator-side discipline mirror

The orchestrator runs:
- FM 9 baseline check pre-spawn: workspace 2232/0 verified (just post-Gap-H)
- FM 12 model: "sonnet" explicit on Agent call
- FM 16: no Bash/cargo/tool-availability preamble in BRIEF (BRIEF doesn't mention them)
- Time-box: ScheduleWakeup at T+7200s (2× 60-min upper bound)
- Atomic-commit after scoring (FM 11 pre-INSCRIPTION grep NOT applicable — I-A is not closure paperwork)
- Push after commit (per `feedback_push_on_commit.md`)
