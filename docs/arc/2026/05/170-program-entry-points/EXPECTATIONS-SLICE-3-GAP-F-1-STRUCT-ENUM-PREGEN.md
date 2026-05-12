# Arc 170 slice 3 Gap F-1 EXPECTATIONS (sonnet scorecard)

**One spawn.** Fourth iteration of preregister-fn-defs extension pattern (after Gap C V2 / D / E). Two helper arms + 4 probes. Closure-sync verification.

## Independent prediction

**Runtime band:** 30-60 min sonnet.

**Hard cap:** 120 min (2×).

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `preregister_fn_defs_in_do` has `is_struct_form` + `is_enum_form` arms (after existing `is_define_form` arm) | grep + read |
| B | `preregister_fn_defs_in_let` has matching arms | grep + read |
| C | New probes pass — 4 minimum (do/let × struct/enum) | cargo test |
| D | All existing 10 Gap C V2 + D + E probes still pass (no regression) | cargo test |
| E | `cargo check --release` green; workspace 2209 + N / 0 failed | full test |
| F | Closure-sync verified (Gap D mirror — runtime-time accessor write-back to sym.functions) OR closure-sync N/A documented | SCORE documents path |

**6 rows.** All must PASS.

## Implementation approach

1. **Identify predicates** (5 min): grep for `is_struct_form` / `is_enum_form`; mint if absent (mirror `is_define_form`)
2. **Probes baseline** (10 min): 4 probes (do/let × struct/enum) confirming failure baseline
3. **Extend `preregister_fn_defs_in_do`** (10 min): two new arms
4. **Extend `preregister_fn_defs_in_let`** (5 min): same arms
5. **Closure-sync** (5-15 min): verify Gap D pattern applies; if so, mirror the fix
6. **Verify** (10 min): all probes + workspace

## What sonnet produces

- `src/runtime.rs` modified (two helpers; possibly `register_runtime_defs_form` for closure-sync; possibly mint `is_struct_form`/`is_enum_form` predicates)
- Probes — 4 test files OR 1 combined file (sonnet picks)
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-F-1-STRUCT-ENUM-PREGEN.md` with:
  - 6-row scorecard
  - Pre-registration shape rationale (stub vs full)
  - Closure-sync verification path
  - Probe organization rationale
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify `is_struct_form` / `is_enum_form` / `parse_*_form` if they exist (only ADD usages)
- Modify `register_defines` / `register_stdlib_defines` (Gap C V2 territory)
- Modify any test call site outside the new probe files
- Touch `docs/arc/`
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Extend to Gap F-2 (resolver) or Gap F-3 (closure extraction) scope
- Use --no-verify or skip hooks

## Verification commands

```bash
# New Gap F-1 probes
cargo test --release --test probe_do_splice_struct 2>&1 | tail -3
cargo test --release --test probe_let_splice_struct 2>&1 | tail -3
cargo test --release --test probe_do_splice_enum 2>&1 | tail -3
cargo test --release --test probe_let_splice_enum 2>&1 | tail -3
# (Or 1 combined test file if sonnet chose that organization)

# Regression: all existing probes still pass
cargo test --release --test probe_do_splice_def 2>&1 | tail -3      # 3 expected
cargo test --release --test probe_let_splice_def 2>&1 | tail -3     # 3 expected
cargo test --release --test probe_do_splice_define 2>&1 | tail -3   # 2 expected
cargo test --release --test probe_let_splice_define 2>&1 | tail -3  # 2 expected

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2209 + N / 0 failed
```

## Expected workspace delta

- Baseline: 2209 passed / 0 failed
- Post-Gap-F-1: 2209 + N passed / 0 failed (N = number of new probes; expect 4 minimum)

## Honest delta categories (anticipated)

1. **Pre-registration shape** — stub-then-replace (Gap D pattern) vs full-generation (cleaner but more work). Surface choice + rationale.
2. **Closure-sync requirement** — does the runtime path through `register_runtime_defs_form` apply for struct/enum? If yes, mirror Gap D's fix. If no, document why N/A.
3. **Probe organization** — 4 separate files vs 1 combined. Existing probes are split (probe_do_splice_def + probe_let_splice_def + ...) suggesting separate. Surface choice.
4. **Sub-form coverage** — typealias-as-struct-field, parametric variants, etc. — surface what's covered + what's deferred to future probes.
5. **Anything unexpected** — particularly any layer-deeper substrate state (e.g., struct registration is via a different pipeline that needs its own extension).
