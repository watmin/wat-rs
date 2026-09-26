# WEIGH — STONE 255.46: what `defintrinsic` needs — ACCEPTED (measurement)

**Executor: grok via pulsare, commit `ceb4f6702`** (the SCORE only; the tree was clean and grok left no
worktree). Weighed by the orchestrator on 2026-09-26 against `main` @ `ceb4f6702`.

## Re-verified by the orchestrator

| claim | verified at |
|---|---|
| `family_extends` answers existence only, arguments ignored | `src/types.rs:1708-1722` (its own doc says so) |
| a `defn` binder peel keeps only symbols, so `(T <- X)` is dropped | `src/function/metadata.rs:48-62` `peel_type_binder` |
| `defclause` clause functions carry `type_params: vec![]` | `src/function/parse.rs:863` |
| `unify` binds a `Var` with no hook beside the insert | `src/check.rs:16714-16719` |
| same-head parametric args unify invariantly, so a `Var` element binds with no covariance | `src/check.rs:16808-16814` |
| `is_type_orderable`: `Var(_) => true`; `Vector`/`Option` recurse; `Result` both; tuple all; `Fn` false; others false | `src/check.rs:13634-13690` |
| `infer_equality` = compatibility (unify / subtype / both records / both numeric) **then** `is_type_equatable` | `src/check.rs:13386-13408` |
| `infer_ordering`'s `both_numeric` exception | `src/check.rs:13735-13746` |
| dispatch: `None => true` ("inference failed for that arg; be permissive") | `src/check.rs:5665-5669` |
| `struct-field` returns `fields[index]` at a runtime index; `@ret :T` | `src/intrinsic/record.rs:290-320` |
| `macro-error` always returns `Err` while its `@ret` says `nil` | `src/intrinsic/macro_error.rs:78-93` |
| **an annotated `(Vector :- [(Spawned :- [S R])])` holds a `Thread` and a `Process`** | **probe, `wat --check` rc=0:** `wat-scripts/scratch-pad/255-46/mixed-spawned-vector.wat`. Control: the same body typed `(Vector :- [(Thread :- [S R])])` is refused, *"parameter #3 expects (Thread :- [:S :R]); got (Process :- [:S :R])"* |

## The reading

**A flat family bound closes two rows, not ten.** `select` and `poll` close: unify binds `E`, then a
`family_extends` check. `Vector` covariance is not needed for any function row. One detail the SCORE did not
say: a mixed `select` vector binds `E` to the surface `(Spawned :- [S R])` itself, so the bound check must
be reflexive (a surface satisfies itself).

**Ordering (4) and equality (2) are not a flat bound.** Their class is a *structural walk*: "`(Vector :- [T])`
is orderable when `T` is". `family_extends` ignores arguments, so an edge cannot say *when*. There are two
honest shapes, and the four questions are the builder's:

- keep `is_type_orderable`/`is_type_equatable` as the meaning of two named bounds, which are hand predicates
  wearing a declared name; or
- **conditional membership:** an `extend-type` whose binder carries the bound,
  `(extend-type :- [(T <- Orderable)] (Vector :- [T]) Orderable)`. That is the **same bounded-binder
  capability**, used on an edge instead of a clause. One mechanism, two positions.

Either way, the equality compatibility rule (subtype, both records, both numeric) and the ordering numeric-pair
exception are not bounds. They are rules of their own.

**`first`/`second`/`third`** need an index-literal tuple rule. **`nth`** closes with the homogeneous
clauses. **`struct-field`** cannot be closed: its result is the field at a runtime index. **`macro-error`** is
a function returning `Never`; its `@ret nil` is wrong.

## ⚠ A correction to the F5 finding, and to pending ruling N-a

F5 says *"wat has no Never type"*. **The checker has one:** `:wat::core::Never` is the universal bottom
(`src/types.rs:7074-7082`, `Never <: every type`), used as a timer peer's send type
(`src/check.rs:11387-11398`). It is **deliberately unregistered**, so no user can write it, and a test pins
that absence (`src/types.rs:8718-8740`, `stone_255b_never_is_deliberately_unregistered`, refused for having no
corpus citation). So **N-a is not "mint a Never type"; it is "register the existing bottom so a declaration can
name it"**. `macro-error`, `assertion-failed!` and a final `eprintln` are its first citations.

## Open design choices surfaced (STOP-1 held: the SCORE chose none)

1. An unresolved variable that carries a bound: **defer** (as `Var(_) => true` does) or **refuse** (which
   makes p11's refusal the bound's own answer).
2. Ordering and equality: named hand predicates, or conditional membership by bounded `extend-type` binders.
3. `struct-field`: it is reflective (a runtime index into any aggregate). It needs a ruling: a form, a
   `Value` return, or kept.
4. Rest-element checking at check-time dispatch is needed regardless (the six variadic rows).
