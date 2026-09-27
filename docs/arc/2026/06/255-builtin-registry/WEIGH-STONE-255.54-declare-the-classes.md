# WEIGH — STONE 255.54: declare `Orderable` and `Equatable` — ACCEPTED, with two rulings owed

**Executor: grok via pulsare, commit `0746b9307`.** Weighed by the orchestrator on 2026-09-26.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor | **6179 passed / 23 skipped** (6173 + 6 rows; the extra skip is the ignored collector) |
| `cargo build --release` | no warnings. The IDE's `unused_doc_comments` at `src/check.rs:13456` is stale (an ordinary doc comment on `fn is_type_equatable`) |
| `wat/class.wat` | read: 2 featureless surfaces, 44 edges, leaves then conditional edges then the `:..` tuple edge |
| `nil` is `Tuple([])` | `src/types.rs:2102-2106` (`register_builtin` Alias `:wat::core::nil` → `TypeExpr::Tuple(vec![])`) |
| the runtime cannot order `nil` or an aggregate | `values_compare` (`src/runtime.rs:5961-6082`) has no `Unit` and no `Aggregate` arm; it ends in `_ => None`. `values_equal` has `(Unit, Unit) => Some(true)` (`:5759`) |
| census / delta / clippy | grok's: pre/post census `no STOP-8`, NEW 2 / RECOVERY 0 same files, clippy rc 0. Not re-run |

The first floor went red on the tracked-stdlib lint (`wat/class.wat` was loaded but not in the index). Grok kept it,
named it and fixed it before the green run.

## ⚠ A correction to WEIGH-255.53

That WEIGH said the empty tuple "needs no ruling" because `(Tuple :- [])` has no values. **That was wrong.**
`:wat::core::nil` *is* `Tuple([])`, and `nil` has a value (`Value::Unit`). So the `:..` Orderable edge admits `nil`,
and the runtime has no ordering for it. The empty-tuple question is live, and it needs a ruling (below).

## STOP-2 fired twice, and was reported, not worked around

Grok named both rows, changed nothing to hide them, and committed the declarations. **The declarations are inert:**
`<`/`=` still use the predicates, so nothing admits `nil` or a newtype to `<` today. Accepted on that basis. The
condition came from my own wrong 255.53 claim (for `nil`) and from ruling N-R assuming a runtime arm that does not
exist (for newtypes). **255.55 must not switch `<` until both are ruled.**

## Agreement (71 corpus operand types)

66 agree. The ruled differences held: bigint/rational (correction); structs (Q1); Impure enums (EN-P); impure-inner
newtypes (N-R). No corpus row exercises the last three. Findings:

- `:T` (rigid, no bound): the class refuses and the predicate defers. 255.55 bounds `assert-eq` (`wat/test.wat:62`),
  `dedupe-walk` (`wat/seq.wat:560`), and the scratch probe.
- `:wat::core::Value`: the class refuses. One scratch probe compares two `Value`s; it stops doing that.
- **`_` (unresolved): `assignable(Var, Class)` unifies the variable into the class, so it admits.** Refuse must answer
  **before** the `unify` fallthrough. That is 255.55's design, already ruled. Sites: `wat/doctest.wat:118`,
  `wat/rete/acc.wat:68`/`:87`, the three `tests/resolve/…fix_source_local_rules__contract-0{6a,6b,7}` one-liners, and
  test and scratch sites.
- `Pt`/`HPt`: admitted by equality's both-records arm, not by the class. E-a drops that arm in 255.55.
- The subtype-arm sites are both covered by variant widening.
