# SCORE — STONE A-1: one rule for "can this value go here"

No commit. Floor and clippy left to the orchestrator. Lands on P-1 / P-2 prereq / P-3.
Nothing reverted. `unify` and `assignable`'s arms were not changed.

## The rule

Unify when either side still contains a type variable; subsume (`assignable`)
when both are concrete. Decide first, call once (`unify` mutates subst).

`if`: the form's type is the one both branches can be seen as — then <: else
→ else; else <: then → then. Not then-authoritative. Swapped branches
(`Parent` then, `Child` else) `--check` EXIT=0.

`send` / `try-send`: payload supplied TO I, same direction as a parameter.
`try-send` is the same I-slot as `send` (its own comment says so); it uses
the same helper, not a third rule.

## Helpers

- `contains_type_var` in `src/declare/typevar.rs` — visitor over
  `walk_type_expr`. Nested Vars count. `fn walk_` stayed **2**.
- `relate_value_to_slot` — send family.
- `join_if_branches` — if.
- Both call existing `assignable`. `grep -c is_subtype src/check.rs` stayed
  **30**. No new arm.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 capability guard | **held.** `if_still_solves_a_type_var` EXIT=0 |
| STOP-2 unrelated goes green | **held.** EXIT=1 |
| STOP-3 new subtyping test | **held.** `assignable` reused unchanged |
| STOP-4 fifth walker | **held.** `walk_type_expr` gained a `visit_var` callback; still one recursion |
| STOP-5 send direction ≠ parameter | **held.** payload → I, not channel-to-channel |

## Expectations

| # | result |
|---|---|
| 1 | subtype-related `if` EXIT=0 |
| 2 | parameter subsumes EXIT=0 |
| 3 | unrelated branches EXIT=1 |
| 4 | type-var solving EXIT=0 |
| 5 | `test(a1_one_rule)` **4 passed, 0 skipped** |
| 6–8 | P-1 10 / P-2prereq 4 / P-3 5, all 0 skipped |
| 9 | `is_subtype` count unchanged (30) |
| 10 | `fn walk_` still 2 |
| 11 | same pair accepted at parameter and at `if` |

## Targeted checks

```
./target/release/wat --check …__parameter_subsumes.wat           EXIT=0
./target/release/wat --check …__if_branches_subtype_related.wat  EXIT=0
./target/release/wat --check …__if_branches_unrelated.wat        EXIT=1
./target/release/wat --check …__if_still_solves_a_type_var.wat   EXIT=0
cargo nextest run --release -E 'test(a1_one_rule)'  4 passed, 0 skipped
```

Floor **orchestrator**. Clippy **orchestrator**. Permissive direction: sites
that used to unify-error may now widen. No `.wat` edited.

## Working tree

```
src/declare/typevar.rs   walk_type_expr visits Var; contains_type_var
src/check.rs             join_if_branches; relate_value_to_slot; if + send + try-send
tests/types/probe_arc296_a1_one_rule_for_assignability.rs  subject un-ignored
```

Do not commit unless a later brief says to.
