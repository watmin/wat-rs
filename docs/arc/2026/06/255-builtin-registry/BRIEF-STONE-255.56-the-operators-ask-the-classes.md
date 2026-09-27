# BRIEF — STONE 255.56: `<` and `=` ask the declared classes; the predicates are deleted

**Drawn 2026-09-26 against `main` @ `e8646678e`.** **Executor: grok, via pulsare.** A strike: `src/`, `wat/`, tests,
`wat-scripts/`. Commit locally on `main`; **do not push**. Then `pulsare_yield kind=scored`.

## The rulings this completes (builder)

- **A / C / `:..` / B1 / Q1 / EN-P / N-R**: built in 255.48–.55. `wat/class.wat` declares `:wat::core::Orderable` and
  `:wat::core::Equatable`; 255.54's agreement test proves them against the predicates.
- **Refuse:** a bounded or class-gated operand that is still an **unresolved variable** is a check error. It is
  **not** unified into the class.
- **E-a:** equality is one class-gated `(T, T)` after **variant widening**, plus the **numeric cross** pair
  (`i64`/`f64`/`bigint`/`rational`). Equality's both-records and subtype disjuncts are dropped.
- **Z1 (2026-09-26):** `:..` matches **one or more** slots, the tuple constructor's own rule (*"tuple must have at
  least one element"*). `nil` (an alias of `Tuple([])`, `src/types.rs:2102-2106`) joins `Equatable` by its own edge,
  and is **not** `Orderable`.

## Read first

- `WEIGH-STONE-255.54-declare-the-classes.md`: the agreement findings and **the work list below**; 255.54's SCORE
  § "What 255.55's switch meets" (file:line for every site).
- `WEIGH-STONE-255.55-a-newtype-is-tagged-and-ordered.md`: `values_compare` is `pub` only for its test.
- `src/check.rs`: `infer_equality` (~13342, compatibility ~13386-13430, the both-records arm ~13421), `infer_ordering`
  (~13693, `both_numeric` ~13735-13746), `is_type_equatable` (~13505), `is_type_orderable` (~13634),
  `widen_to_enclosing_enum` (17214), `is_numeric_check_path` (13338), `assignable` (its final `unify` fallthrough,
  ~18083).
- `wat/class.wat`; the `:..` edge match (255.53).

## The work

1. **The gates.** `infer_ordering`: after widening, each operand must satisfy `:wat::core::Orderable` by the class.
   `infer_equality`: after widening, `unify`, or the numeric-cross pair; then each operand must satisfy
   `:wat::core::Equatable` by the class. A miss keeps a clear error naming the operand type and the class (reuse
   `MembershipBound`'s names where a conditional edge failed).
2. **Refuse.** Before any class check, an operand that is still a `TypeExpr::Var` is refused, naming the operator
   and saying the operand's type is unresolved. It must not reach `assignable`'s `unify` fallthrough.
3. **Z1.** The `:..` match requires at least one slot. Add `(:wat::core::extend-type :wat::core::nil
   :wat::core::Equatable)` to `wat/class.wat`.
4. **Delete** `is_type_orderable`, `is_type_equatable` and every use. Keep `is_numeric_check_path` only if the
   numeric-cross rule still reads it.
5. **The sites 255.54 named**, each fixed at its logical home:
   - `wat/test.wat:62` `assert-eq`, `wat/seq.wat:560` `dedupe-walk` (and any caller that must carry the bound up):
     `[T :< :wat::core::Equatable]`. The scratch `probe-eq-generic-instantiation.wat` gets the same bound.
   - The unresolved stdlib operands: `wat/doctest.wat:118`, `wat/rete/acc.wat:68` and `:87`. Pin the **real** type at
     its source (an annotation or a typed binding that states what the value is). Not a cast to a class.
   - The four `tests/types/ord_result_*` tests: annotate `E` (ruled).
   - `tests/types/probe_arc237_sC3_macro_split.wat:41` `(= Pt HPt)`: under E-a it is refused. Move that row to a
     `.wat.bad` whose Rust test asserts the refusal with names. (`Record/same-data?` stays the cross-type tool.)
   - `wat-scripts/scratch-pad/255-probe-metadata-of-one-shape.wat:63` compares two `Value`s: stop comparing `Value`.
   - Any other unresolved-operand test or scratch site from 255.54's list: pin its type.
6. **Tests.** Extend 255.54's test: `<` and `=` now accept exactly the class members. Drive the operator in wat for:
   a newtype declared `Orderable` (`(< (:u::T 1) (:u::T 2))` → true, reaching 255.55's arm); `nil` under `=`
   (admitted) and `<` (refused); a struct under `=` (refused, Q1); an Impure enum under `=` (refused); an unresolved
   operand (refused, Refuse); `(< 1 2.0)` (admitted, numeric cross); an enum against its own variant under `=`
   (admitted, widening). Then return `values_compare` to `pub(crate)` and drive 255.55's ordering row through `<`.
   The agreement test's allow-list shrinks to nothing: the predicates are gone. Delete or retire it, and say which.

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh` | all passed; the count against 6186 at `55fe36769` ± the retired and new rows, each named |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census / delta | pre-census on the unmodified draw; `scripts/replay/census.sh --diff`; `scripts/replay/delta.sh` | no STOP-8; every rc flip named and attributed to a site in step 5; NEW 2 / RECOVERY 0, same `new.txt` |
| the predicates | `grep -n 'is_type_orderable\|is_type_equatable' src` | only comments, if any |

## STOP triggers (checked against the work list: each fires only outside the sites step 5 names)

- **STOP-1:** a site **not** in step 5 goes red (a check refusal, a census rc flip, or a floor red). List it, with its
  operand type and why, and STOP. This includes the three `tests/resolve/…fix_source_local_rules__contract-0{6a,6b,7}`
  one-liners: they are the fix-source codemod's inputs, not step-5 sites, so their disposition is the builder's.
- **STOP-2:** a stdlib operand in step 5 cannot be pinned to a real type without a design decision (the value really is
  heterogeneous, or its type comes from outside the checker). Report the shape and STOP.
- A STOP means STOP: report, do not work around it.

## Doctrine

- There is no known flake. On a red: do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.56-the-operators-ask-the-classes.md` beside this brief. Commit the change and the SCORE on
  `main` (`git add -- <paths>`, never `-A`). **Do not push.**
