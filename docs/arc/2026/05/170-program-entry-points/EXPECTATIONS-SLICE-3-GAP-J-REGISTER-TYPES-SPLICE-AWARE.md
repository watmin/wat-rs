# Arc 170 slice 3 Gap J EXPECTATIONS (sonnet scorecard)

**One spawn.** Extend `register_types` to recurse into top-level `do`/`let` body, registering nested type declarations. Then apply the V4 BRIEF target shape to deftest. Verification: all 13 V5-previously-failing tests pass.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

**Hard cap:** 180 min (2×). ScheduleWakeup at T+10800s.

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `register_types` recurses into top-level `do` body; nested type decls register in TypeEnv | grep + read |
| B | `register_types` recurses into top-level `let` body (items[2..] per arc 168); nested type decls register | grep + read |
| C | `register_stdlib_types` mirrors the same splice-recursion for substrate-baked stdlib forms | grep + read |
| D | 7+ probes in `tests/probe_register_types_splice_aware.rs` pass (do_typealias/struct/newtype/enum + let_body_typealias + nested_do_typealias + do_typealias_usage_typechecks) | cargo test |
| E | Phase E V5 deftest macro rewrite applied (target shape from V4 BRIEF); all 13 previously-failing tests now pass | cargo test |
| F | Workspace at 2243 + N (new probes) / 0 failed; no regression for ANY existing test | full test run |

**6 rows.** All must PASS.

## Implementation approach (sonnet's path)

1. **Crawl the surface** (5-10 min) — read `register_types` + `register_stdlib_types` + `classify_type_decl` (~50 lines); read `preregister_fn_defs_in_do`/`_in_let` as the splice-recursion pattern to mirror
2. **Mint the splice helpers** (15-20 min) — `process_do_for_types` + `process_let_for_types` (per BRIEF Phase 1); integrate into `register_types` + `register_stdlib_types`
3. **Probes** (10-15 min) — 7+ regression probes per BRIEF Phase 2
4. **V5 retry** (15-20 min) — apply V4 BRIEF target shape to deftest defmacro body in `wat/test.wat`
5. **Verify** (15-25 min) — Gap J probes + V5 retry workspace pass + 4 Gap G probes + 11 prior substrate probes

## What sonnet produces

- `src/types.rs` modified (`register_types` + `register_stdlib_types` extended with splice-recursion)
- `wat/test.wat` modified (deftest defmacro body uses V4 target shape; documentation header updated)
- New probe file `tests/probe_register_types_splice_aware.rs`
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-J-REGISTER-TYPES-SPLICE-AWARE.md` with:
  - 6-row scorecard with PASS/FAIL per row
  - Splice-recursion design (form preservation; nested-do termination; error span preservation)
  - V5 retry result (which previously-failing tests now pass; any honest deltas)
  - Stdlib coverage rationale
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify `expand_alias` / `reduce` / `unify` — substrate machinery is correct
- Modify `preregister_fn_defs_in_do` / `_in_let` — existing splice machinery for fn defs stays
- Modify Gap F-3's `extract_closure` type-registry inheritance
- Retire `run-sandboxed-*` (Phase F)
- Touch deftest-hermetic — already shipped (`5d82e92`); leave alone
- Touch `docs/arc/` (FM 11)
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Use --no-verify or skip hooks
- Ship Gap J fix without V5 retry passing — V5 retry IS the load-bearing proof; if V5 still has failures, STOP and report

## Verification commands

```bash
# New Gap J probes
cargo test --release --test probe_register_types_splice_aware 2>&1 | tail -10

# V5 retry — the load-bearing test
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2243 + N / 0 failed (N = new probes; expect ≥ 7)

# Regression checks
cargo test --release --test probe_deftest_hermetic_isolation 2>&1 | tail -5  # Gap G probes
cargo test --release --test probe_closure_body_prelude_lift 2>&1 | tail -5    # Gap H probes
cargo test --release --test probe_declaration_form_lift 2>&1 | tail -5         # Gap I-A probes
cargo test --release --test probe_def_not_special 2>&1 | tail -5               # Gap I-B probes
```

## Expected workspace delta

- Baseline (post-deftest-hermetic Path E, commit `5d82e92`): **2243 passed / 0 failed**
- Post-Gap-J + V5 retry: **2243 + N passed / 0 failed** (N = new probes; ≥ 7)
- **Critically:** workspace stays at 0 FAILED. V5 retry's 13 previously-failing tests now pass.

If failed > 0 after Gap J + V5 retry:
- Probe failure in Gap J's own new tests → expected during iteration; sonnet fixes; final must be 0 failed
- V5 retry still has failures → STOP and report; the hypothesis may be incomplete; surface as honest delta
- Regression in existing tests → STOP and report

## Honest delta categories (anticipated)

1. **Form reconstruction after type-decl extraction** — what does the do form look like when type decls are stripped? Surface the reconstruction shape + edge case "all children were type decls" (the do degenerates).
2. **Nested do termination** — recursion handles do-in-do naturally; surface termination guarantee.
3. **Error span preservation** — when nested decl errors arise, span should point to the actual decl. Verify via probe.
4. **V5 retry honest deltas** — if any of the 13 originally-failing tests STILL fail, document each. The hypothesis says all 3 patterns close from one fix; surface any that don't.
5. **Stdlib coverage** — does `register_stdlib_types`'s splice-recursion need different rules? Probably no; same recursion. Surface confirmation.
6. **Anything unexpected** — particularly around classify_type_decl's interaction with the new helpers, or order-sensitivity in `register_with_span` for nested decls

## Constraints — orchestrator-side discipline mirror

- FM 9 baseline check pre-spawn: workspace 2243/0 verified post-Path-E
- FM 12 model: "sonnet" explicit on Agent call
- FM 16: no Bash/cargo/tool-availability preamble in BRIEF
- Time-box: ScheduleWakeup at T+10800s (2× 90-min upper bound)
- Atomic-commit after scoring
- Push after commit
