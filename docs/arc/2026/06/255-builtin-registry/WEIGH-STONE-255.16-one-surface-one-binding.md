# WEIGH — STONE 255.16: a type binds a parametric surface once — ACCEPTED

**Executor commit `129ac35c9`** on top of the brief `00ceb5f30`. Weighed by the orchestrator against
disk on 2026-09-24.

## Re-measured by the orchestrator, not taken from the report

| row | measured | result |
|---|---|---|
| the hole (`scratchpad/p/ambig-lie-run.wat`) | snapshot binary `58d7e0ad3` vs the new binary, `--check` | old **rc=0** → new **rc=1** `ParametricSurfaceBoundTwice ":probe::Both already binds (:probe::Loc :- [:probe::Shared]) … refused second binding (:probe::Loc :- [:probe::Wire])"` |
| a legitimate single bodiless extend-type | `--check tests/services/probe_arc170_c2_d_bodiless_edge_ok.wat` | rc=0 |
| floor | `.floor/2026-09-24T02-49-49Z/`, the Summary line | `6027 tests run: 6027 passed (9 slow), 22 skipped` (6023 + 4 new) |
| stale citation | `grep -rn 'types.rs:2151' src tests` | 0 |
| tree | `git status` | clean apart from the orchestrator's uncommitted FINDING (D2) |
| the diff | read `src/types.rs`, `src/check.rs` | the refusal sits in `register_parametric_extension` before either half of the fact is written; it keys on the surface head (`parametric_heads_unify`) and exempts `type_exprs_same`; the child is compared by denotation |

## Taken from the report, not independently re-run

- **Order independence, identical re-registration, different types:** the old/new table in the report,
  plus the committed rows `_bodiless_first.wat.bad`, `_identical.wat`, `_different_types.wat`.
- **The tests can fail:** with the refusal mutated out, the 255.15/16 filter gave 10 passed and 3 failed.
- **Clippy 0; census `no STOP-8` (non-zero 213 → 213); delta NEW 3 / RECOVERY 0.**
- **Reachability:** 255.15's `solutions.len() > 1` guard is **unreachable** through registration. A
  tripwire fired on the control (refusal disabled). With the refusal enabled it gave 0 hits over the
  full floor and over `--check` of 2545 tracked files. **The guard is kept.** Removing it is a separate
  decision, and the guard now stands as a second line of defence.

## Beyond the brief

- **Two spellings of one type** (`probe/Both` and `:probe::Both`) could each hold a binding. That was a
  second route to the same lie. The old binary accepted it; the new one refuses it.
- **Registration refusals surface as `StartupError::Type`,** so a program run directly exits rc=3 while
  `--check` exits rc=1.

## Open, carried forward

- Other comments cite line numbers (`types.rs:1402`, `types.rs:1987`). They are unchecked; the same
  kind of rot as `:2151`.
- The `derive`-chain hole (a separate mechanism).
- **D2** (`FINDING-a-matched-field-forgets-its-type-argument.md`): a runtime type lie of the same class,
  found by the alias probe. It is proposed as 255.17 and is now the stone that blocks the alias's
  candidate 2.
