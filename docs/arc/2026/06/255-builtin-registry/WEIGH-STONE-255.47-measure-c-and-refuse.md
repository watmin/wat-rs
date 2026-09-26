# WEIGH — STONE 255.47: where C and Refuse land — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `84654cea1`** (the SCORE only; the tree was clean, and grok removed its
worktree). Weighed by the orchestrator on 2026-09-26 against `main` @ `84654cea1`.

## Re-verified by the orchestrator

| claim | verified |
|---|---|
| surfaces are width subtyping by design: "No `:satisfies`, no `:parent`, no declaration at the use site" | `src/types/surface.rs:1-6` |
| an empty member list is vacuously satisfied | `src/types/surface.rs:58` `surface.members.iter().all(…)` |
| `Reason` is open on purpose: "any pure record satisfies it ambiently" | `wat/query.wat:73-76` |
| `(< bigint bigint)` is refused by the checker | `wat --check`, rc 1: *"expects an orderable type (i64, u8, f64, …); got :wat::core::bigint"* |
| `(< i64 f64)` checks; `=` on two different record types checks | `wat --check`, rc 0 |
| ordering two `Result.Ok` values leaves `E` unpinned; the test checks today | `tests/types/ord_result_ok_le_same.wat`, rc 0 |
| `is_type_param_letter` answers true for any `Var` | `src/check.rs:10553-10557` |
| the `HashMap`/`HashSet`/`PersistentVector` equality blanket delegates to `Value`'s total `PartialEq` | `src/check.rs:13582-13585`, `src/runtime.rs:5898-5902` |

I did not re-run the 2285-file instrumented census. It is weighed on its method, on the sites it names, and on
the one site I re-checked (`ord_result_ok_le_same.wat`).

## The reading

1. **Hole B is not a bug; it is the surface design.** Surfaces are structural (width subtyping). A surface with
   no members admits every aggregate that clears its nature floor. **C needs the opposite, declared membership.**
   The code already disagrees with itself: the featureless *parametric* `Spawned` refuses an undeclared record
   (255.46 probe), while the featureless *non-parametric* `Reason` admits one. 68 corpus sites rely on the open
   case, from **7 distinct types** (`Fault`, three `probe::*Reason`s, `Note`, `Opaque`, `env::Rec`).
2. **Refuse bites exactly one honest case:** 4 tests order two `Result.Ok` values whose `E` nothing pins. The
   other 11 hits are generic `defn`s (an author would add a bound), unpinned `eval-ast!`/`map/get` results, and
   three one-line tests with untyped free names.
3. **Refuse as worded does not reach p11.** In the 15 first-clause hits (`+` 3, `rete/insert` 7, `run!` 5),
   the clause binds the variable before the call ends. **The code already has the rule that does reach it:**
   `assignable`'s unique-solution arm, "TWO OR MORE → AMBIGUOUS, refused — never pick one" (`src/check.rs`,
   Stone 255.15). The same rule applied to clause dispatch refuses an argument that more than one clause could
   take. It has not been measured how many of the 15 are ambiguous and how many are unique.
4. **Two classes change when declared:** `bigint` and `rational` are ordered at runtime and refused by the
   checker, so declaring them is a correction. The equality blanket on hash containers is total at runtime,
   so it stays unconditional, and it is honest.
5. **What C does not yet say:**
   - tuples ("orderable when each element is", with open arity);
   - "every aggregate is equatable". Aggregates already register an edge to their nature root
     (`Nature::root_keyword`), so one edge on the root may say it. Inferred, not measured.
   - equality's subtype compatibility. A bound whose bound is the other variable, `(B <- A)`, may say it.
     Inferred, not measured.

## Decisions for the builder

- **Hole B:** a featureless surface's membership is declared only, or a separate closed-class declarator.
- **The unpinned `E`:** Refuse stands and the 4 tests annotate `E`, or an unconstrained variable defaults.
- **p11:** extend the existing unique-solution rule to clause dispatch.
