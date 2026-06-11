# DESIGN — Arc 256: generic defclause (parametric clause dispatch)

**Status: SHIPPED 2026-06-10. Sonnet-built (13-line check-side instantiation hook + 1 visibility
keyword), orchestrator-weighed against the disk.** Probe `tests/probe_arc256_generic_defclause.rs`
5/5; 251.7 probe 5/5 (no regression); lib 949/0; `types` 83/0 serial-deterministic (the suite flakes
ONLY under parallel test-threads via the known arc-170 spawn-deadlock — green with `--test-threads=1`).
Home: `src/check.rs` (the defclause call-site dispatch loop). Ports 251.7 (SHIPPED `0c95ae2c`) to the
`defclause`/`ClauseSet` entity. Banked as task #198.

## The move

A `defclause` is wat's multimethod — a set of clauses dispatched by arg type. Make a clause
**generic**: a clause whose arg/return types contain bare type-variables (`(:wat::core::defclause
:user::firstof ([a <- :T b <- :T] -> :T a))`) must type-check — `(firstof 1 2)` returns `i64`,
`(firstof true false)` returns `bool`, `(firstof 1 "two")` is rejected. Today the checker spuriously
rejects all three (the clause never matches). This is the exact RED 251.7 flipped for functions,
now ported to the clause entity.

## The disk (grounded, 2026-06-10) — runtime already works; only the CHECKER is the gap

- **`Clause`** (`value/value.rs:367`): `args: Vec<(String, TypeExpr)>`, `return_type: TypeExpr`,
  `rest_param`, `guard`, `ensure_fn`, `body`. **Arg + return types are already `TypeExpr`** — so a
  parametric form `(wat.type/HashMap K V)` or a bare `:T` already PARSES + STORES (251.3a).
- **`ClauseSet`** (`value/value.rs:396`): `name`, `clauses`, `shared_return` — no `type_params`
  (none needed; see the strike — instantiation is per-clause, computed on the fly).
- **RUNTIME dispatch ALREADY handles generics** — `value_matches_type_pattern` (`runtime.rs:4721`):
  a `Path` that is bare (no `::`) + Uppercase-initial is *"Type variable — matches anything"*
  (`runtime.rs:4733`); a `Parametric` pattern matches by **container head**, inner type-vars are
  runtime wildcards. So at runtime a generic clause dispatches correctly. **No runtime change.**
- **The CHECK-side dispatch is the gap** — `infer_list`'s defclause arm (`check.rs:5350`–`5412`):
  it looks up `env.get_defclause_clauses` (a `Vec<(Vec<TypeExpr> arg_types, TypeExpr ret, bool
  has_rest)>`, `check/env.rs:102`), then per clause does, per position,
  `assignable(arg_ty, expected_ty, &mut clause_subst, env.types())` (`check.rs:5377`). When
  `expected_ty` is a bare `Path(":T")`, `assignable` → `unify(i64, Path(":T"))` → **fails** (a rigid
  path, not a `Var`). So no clause matches → `NoMatchingClauseAtCallSite`. **The checker rejects
  what the runtime would accept** — the asymmetry this arc closes.
- **Definition-side likely already works** — `infer_defclause` (`check.rs:7648`/`7681`) checks each
  clause body under `clause_locals` = arg→declared-type (rigid `:T`); the body returning `a:T`
  unifies with declared ret `:T` (rigid == rigid). The probe confirms the definition alone is
  accepted at HEAD (only the call is rejected). If the probe shows the definition is ALSO rejected,
  the body-check needs the same instantiation — STOP and report.

## The strike (check-side only)

The contract decision, pinned to ONE site:

> **In the defclause call-site dispatch loop (`check.rs:5362`), INSTANTIATE each clause's
> type-variables to fresh unification vars BEFORE the per-position `assignable` loop — exactly as
> the normal-scheme path does `instantiate` (`check.rs:5421`+/`13553`) and as 251.7 generalizes for
> fns. The fresh-var mapping is per-clause and SHARED across that clause's positions + return (so
> `[a :- :T b :- :T] :- :T` ties all three to one var). Keep the ORIGINAL clause types for the
> `attempted`/`NoMatchingClauseAtCallSite` error formatting (don't surface `?n3` to users).**

Sketch:
1. **Reuse the var machinery.** Make `runtime::collect_free_type_vars` (built in 251.7) `pub(crate)`
   — OR inline the identical uppercase-bare test (`runtime.rs:4726`/251.7) — to collect a clause's
   type-var names from `clause_arg_types ∪ clause_ret`. Reuse `check::rename` (`check.rs:13577`) +
   `InferCtx::fresh` (`check.rs:13409`) to build the `{name → fresh_var}` mapping and rename the
   clause's arg types + ret.
2. **Instantiate at the dispatch loop** (`check.rs:5362`–`5389`): for each clause, before the
   `assignable` zip-loop, compute the fresh mapping and rename `clause_arg_types` + `clause_ret` to
   `inst_arg_types` + `inst_ret`. Unify `arg_tys` against `inst_arg_types`; on full match,
   `matched_ret = apply_subst(&inst_ret, subst)`. The `attempted` Vec (line 5364) keeps the ORIGINAL
   `clause_arg_types` for the error message.
3. **No registration schema change** — `defclause_registrations` stays `(Vec<TypeExpr>, TypeExpr,
   bool)`. The instantiation is computed on the fly from the stored types (the var-test is total).
   (If a future need arises to precompute, add `type_params` to the tuple — NOT needed now.)
4. **No runtime change** (`value_matches_type_pattern` already correct).
5. **No definition-side change** expected (probe confirms; STOP if not).

## The probe (RED at HEAD)

`tests/probe_arc256_generic_defclause.rs`:
- **C01 (def-only, fact):** a generic defclause `(:wat::core::defclause :user::firstof ([a <- :T
  b <- :T] -> :T a))` DEFINED but not called — accepted at HEAD (proves the gap is call-side only;
  if rejected, definition-side needs work too → STOP).
- **C02 (RED→GREEN, load-bearing):** define + `(:user::firstof 1 2)` returning `:i64` — **FAILS at
  HEAD** (`assignable(i64, :T)` fails → NoMatchingClause). The build flips it GREEN.
- **C03 (really-unified, not tolerated):** define + `(:user::firstof 1 "two")` — must be REJECTED
  (T:=i64 then b=String). After the build, must stay rejected.
- **C04 (two instantiations):** `(:user::firstof 1 2)`→i64 and `(:user::firstof true false)`→bool
  both check — distinct fresh vars per call site.
- **C05 (parametric, if expressible):** a clause over `(wat.type/Vector T)` or a parametric head —
  dispatch matches by container head + inner var. Only if constructible with current syntax in
  ~10 min; else SKIP + note.

## Out of scope (named — affirmative cuts)

- **Re-clause-ifying the kernel intrinsics** (`+`/`-`/`*`/`/`, `<`/`>`/`<=`/`>=`, `=`/`not=`) —
  out of scope, and here is the precise reason (the two-pillar analysis, 2026-06-10). The intrinsic
  boundary (`infer_ordering`/`infer_equality`/arith as Rust check-side arms) stood on TWO pillars:
  1. **Type-expressibility over parametric types** — *"a finite clause list cannot express
     `Vec<T> < Vec<T>`, `Option<T>`"* (the Stone 245.8 retirement rationale). **Arc 256 KILLS this
     pillar** — a generic clause now expresses `∀T. (Vector T) → (Vector T) → bool` in one clause
     (probe C05 proves parametric-container dispatch).
  2. **The bounded constraint `∀T : Orderable`** — `infer_ordering` gates on `is_type_orderable`
     (`check.rs:11514`): i64/f64/String/… + recursively `Vector<orderable>`, but NOT
     HashMap/HashSet/functions/user structs. A 256 generic clause is **unbounded** `∀T` — it would
     wrongly accept `(< some-hashmap other)`. wat has NO type-class / bounded-polymorphism
     mechanism (only subtyping via `register_subtype`). **256 does NOT touch this pillar.**
  So the boundary SURVIVES, now resting on a single remaining pillar — the absence of **bounded
  polymorphism (`∀T : Orderable`)**. To truly retire the intrinsics you need bounded `∀`, a
  DISTINCT, meatier arc. Two notes for whoever picks it up: (a) the hard pieces already exist — the
  `Orderable` predicate IS `is_type_orderable`, and the impl needs NO dictionaries (`eval_compare`,
  `runtime.rs:7203`, is already a universal structural comparator); the unknown is constraint
  PROPAGATION through generic-over-generic chains (HM qualified-types on wat's rank-1 unifier —
  probe it before committing). (b) FULL user-declarable typeclasses + method dispatch is a big arc
  with NO clean faithful-Clojure precedent (core.typed has no typeclass story; protocols are runtime
  dispatch) — an ADT-compass red flag; resist. (c) It is OFF the 251 critical path and is optional
  *uniformity*, not capability — kernel-primitive arithmetic is normal; program-over-the-wire does
  not need `+`/`<` to be user-defined. Decide deliberately, not by momentum.
- **`<T,U>` name-suffix on defclause names** — defclause type-vars are read from the signature only
  (same as 251.7's faithful form); no suffix needed.

## Lineage

Ports 251.7 (`DESIGN-STONE-251.7-implicit-generics.md`, the fn version — SHIPPED `0c95ae2c`) onto
the clause entity. Same recipe (instantiate type-vars at the call site), same var-test (bare +
Uppercase-initial), same engine (`rename`/`fresh`/`unify`), different dispatch path. Pairs
[[project_typed_clojure_parity_pivot]].
