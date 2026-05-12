# Arc 170 slice 3 Gap E EXPECTATIONS (sonnet scorecard)

**One spawn.** Tight mirror of Gap C V2 + Gap D. Two ~10-LOC additions; four new probes.

## Independent prediction

**Runtime band:** 15-30 min sonnet.

**Hard cap:** 60 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `preregister_fn_defs_in_do` (runtime.rs ~2246) has `is_define_form` arm | grep + read |
| B | `preregister_fn_defs_in_let` (runtime.rs ~2293) has `is_define_form` arm | grep + read |
| C | `tests/probe_do_splice_define.rs` — 2 probes pass | cargo test |
| D | `tests/probe_let_splice_define.rs` — 2 probes pass | cargo test |
| E | Existing `probe_do_splice_def.rs` + `probe_let_splice_def.rs` still pass (no regression) | cargo test |
| F | Workspace at 2209 / 0 failed (2205 baseline + 4 new probes) | full cargo test |

**6 rows.** All must PASS.

## Implementation approach

1. **Write probes** (5 min): 4 probes across 2 new test files (mirror existing probe_do_splice_def + probe_let_splice_def). Confirm failing baseline.
2. **Extend `preregister_fn_defs_in_do`** (5 min): add `is_define_form` arm; run do-probe.
3. **Mirror into `preregister_fn_defs_in_let`** (5 min): same arm.
4. **Verify** (5-10 min): all probe sets + full workspace.

## What sonnet produces

- `src/runtime.rs` modified (two ~10-LOC additions, one per helper)
- `tests/probe_do_splice_define.rs` (2 probes; new file)
- `tests/probe_let_splice_define.rs` (2 probes; new file)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-E-DEFINE-IN-DO-LET.md` with:
  - 6-row scorecard
  - Define-form recognition order rationale
  - Closure-sync verification (should be no-op)
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify `is_define_form` / `parse_define_form` / `try_parse_fn_shape_def`
- Modify `register_defines` / `register_stdlib_defines` (Gap C V2 territory)
- Modify any test call site outside the 2 new probe files
- Touch `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Add new substrate features
- Run hooks bypass / `--no-verify`

## Verification commands

```bash
cargo test --release --test probe_do_splice_define 2>&1 | tail -5
cargo test --release --test probe_let_splice_define 2>&1 | tail -5
cargo test --release --test probe_do_splice_def 2>&1 | tail -5
cargo test --release --test probe_let_splice_def 2>&1 | tail -5
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2 + 2 + 3 + 3 passes; 2209 / 0 workspace
```

## Expected workspace delta

- Baseline: 2205 passed / 0 failed
- Post-Gap-E: 2209 passed / 0 failed (+4 probes)

## Honest delta categories (anticipated)

1. **Arm order** — `is_define_form` before or after `try_parse_fn_shape_def`? Surface rationale (likely AFTER, since def/defn are the new canonical).
2. **Closure-sync verification** — Gap D needed closure-sync fix in `register_runtime_defs_form` because let-body fns capture let-local bindings. Does `define`-in-let have the same issue? Verify (likely not — define forms don't close over let bindings; they're top-level by position).
3. **Probe shape parity** — any deviation from existing probe file shapes
4. **Workspace impact** — should be zero behavior change for any existing test
5. **Anything unexpected**
