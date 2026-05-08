# Arc 161 — Slice 1 EXPECTATIONS

**Drafted 2026-05-07.** Pre-spawn predictions for sonnet's
arc 161 slice 1 substrate fix.

## Independent prediction

**Mode A.** ~10-15 minutes wall-clock.

Single-site fix in `src/check.rs::infer_list`. The reference pattern
(`infer_spawn` 7556-7589) is on disk; sonnet adapts ~50 LOC into
the no-op branch at lines 4606-4613. Helpers (`reduce`,
`apply_subst`, `unify`) are existing.

The 1 currently-failing test repros from a minimal probe; the fix
shape is dictated by the diagnostic. No design freedom for sonnet.

## Hard scorecard

| Row | Pass criterion |
|---|---|
| R1 | Workspace pre-fix: 1 failed (`deftest_wat_telemetry_test_svc_tel_null_translator`); confirm with `cargo test --release --workspace 2>&1 \| grep "test result"` |
| R2 | Workspace post-fix: 0 failed |
| R3 | Specific test passes: `cargo test --release -p wat-telemetry deftest_wat_telemetry_test_svc_tel_null_translator 2>&1 \| grep "test result"` shows `1 passed` |
| R4 | Edit limited to `src/check.rs` (1 file) |
| R5 | Working tree dirty (no commits from sonnet) |
| R6 | Sonnet's report names the helper functions used (`reduce`, `apply_subst`, `unify`) and confirms the value-head branch mirrors `infer_spawn` 7556-7589 |
| R7 | Sonnet's report flags any unexpected diagnostics surfaced during `cargo test --release` |

## Path classifications

- **Mode A**: clean fix, all rows pass, workspace 0-failed. ~10-15 min.
- **Mode B**: fix lands but a pre-existing test broke. Sonnet stops
  per "STOP at unexpected red" and reports. Orchestrator decides
  whether to scope-extend.
- **Mode C**: fix doesn't compile or doesn't clear the failing test.
  Sonnet stops and reports diagnostic state.

## Time-box wakeup

2× upper-bound = 30 min. Wakeup at T+30 min.

## Honest deltas to flag

- **If inline-expression heads need different handling than Symbol
  heads:** flag in the report. The unified `infer(head)` path
  *should* cover both (`infer` dispatches on AST shape internally),
  but if sonnet finds a case the unified path misses, surface it.
- **If `reduce` is the wrong canonicalization helper for this site:**
  `infer_spawn` uses `reduce`; this site mirrors it. If a probe
  reveals `reduce` strips information the application needs, flag.
- **If `unify` panics on Var-vs-Var arg patterns:** the keyword-headed
  branch already handles this; the value-head branch should be
  symmetric. If sonnet hits a panic, treat as substrate gap (Mode C).

## Substrate assumptions verified

- `infer` returns `Option<TypeExpr>`; for Symbol head, returns
  `locals.get(&ident.name).cloned()` (line 3641). Confirmed.
- `reduce(ty, subst, types)` performs full canonicalization
  (Var-walk + alias expansion). Confirmed at line 9762.
- `unify(arg, expected, subst, types)` mutates `subst` in-place;
  returns `Result<_, _>`. Confirmed (used at line 4481).
- `apply_subst(ty, subst)` produces a fully-substituted type.
  Confirmed at line 9719.
- `TypeExpr::Fn { args, ret }` is the only Fn shape (no
  variadic/generic-Fn at this layer). Confirmed via grep.

## Cross-references

- `BRIEF-SLICE-1.md` — the brief sonnet executes
- `DESIGN.md` — context and rationale
- Arc 160's BRIEF-SLICE-2 — sibling pattern (substrate inference
  fix mirroring an existing branch)
