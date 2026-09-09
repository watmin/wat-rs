# SCORE — STONE P-1b: a parametric annotation's HEAD is a named type too

No commit. Floor and clippy left to the orchestrator. Lands on P-1. Nothing
reverted. No change to `assignable`, `unify`, `resolve`, or any `.wat`.

## The walk

`walk_type_expr` gained a `visit_head` callback — the same shape as A-1's
`visit_var`. The `Parametric` arm visits the head as an FQDN through
`parametric_head_fqdn` (storage is colon-free; Path nodes already carry the
colon). `fn walk_` stayed **2**.

`first_unknown_named_type` passes the same `consider_named_path` to
`visit_path` and `visit_head`. No head special-case in the consumer: type
variable → accept; bound param → accept; four-store union → accept;
otherwise refuse.

`walk_free_type_vars` and `contains_type_var` ignore `visit_head`.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 `parametric_head_real` red | **held.** EXIT=0 |
| STOP-2 second walker / consumer special-case | **held.** One recursion, extra callback |
| STOP-3 free-vars collect heads | **held.** `collect_free_type_vars_ignores_parametric_head` — only `T` from `(:usr::TotallyMadeUp :- [:T])` |
| STOP-4 corpus red naming a real type | **held.** 845 files, 0 UnknownNamedType |

## Expectations

| # | result |
|---|---|
| 1 | parametric phantom head EXIT=1, path `:usr::TotallyMadeUp` |
| 2 | real parametric head EXIT=0 |
| 3 | bare phantom EXIT=1 |
| 4 | arg phantom EXIT=1 |
| 5 | `test(p1b_a_parametric)` **4 passed, 0 skipped** |
| 6–9 | P-1 10 / P-2prereq 4 / P-3 5 / A-1 4 |
| 10 | `fn walk_` still 2 |
| 11 | `declare::typevar::tests::collect_free_type_vars_ignores_parametric_head` |

## Targeted checks

```
./target/release/wat --check …__parametric_head_phantom.wat  EXIT=1  :usr::TotallyMadeUp
./target/release/wat --check …__parametric_head_real.wat     EXIT=0
./target/release/wat --check …__bare_head_phantom.wat        EXIT=1
./target/release/wat --check …__parametric_arg_phantom.wat   EXIT=1
cargo nextest run --release -E 'test(p1b_a_parametric)'  4 passed, 0 skipped
```

Corpus `wat/` `wat-scripts/` `wat-tests/`: **845 / 0**. Floor **orchestrator**.

## Working tree

```
src/declare/typevar.rs   visit_head; consider_named_path; free-var pin test
tests/types/probe_arc296_p1b_a_parametric_head_is_a_named_type.rs  subject un-ignored
```

Do not commit unless a later brief says to.
