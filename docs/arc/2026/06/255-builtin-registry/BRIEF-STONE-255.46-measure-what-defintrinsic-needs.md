# BRIEF — STONE 255.46: MEASURE what `defintrinsic` needs to type every function row (nothing lands but the SCORE)

**Drawn 2026-09-26 against `main` @ `0d9b82ccd`.** **Executor: grok, via pulsare.** **MEASUREMENT ONLY.** If you must
change code to measure, work in a `git worktree` under the session scratchpad (symlink `../holon-rs` beside it if the
build needs it); never edit the main tree. Write the SCORE on `main`, commit **only** that file, then
`pulsare_yield kind=scored`.

## The ruling this serves

**The builder ruled A:** an intrinsic's signature gets ONE source of truth, a bodiless wat declaration
(`defintrinsic`) of one or more clauses, resolved by defclause's check-time dispatch, with Rust as the body. The
ruling holds only if **every function row** gets a declared type. A row left on a hand `infer_*` arm is a second
way of typing functions. The special forms (`let`, `if`, …) and the form-shaped intrinsics are out of this set;
they are forms.

255.45 (`SCORE-STONE-255.45-…`, `WEIGH-STONE-255.45-…`) left these function rows uncovered. This stone measures
exactly what closes each.

## Read first

- `WEIGH-STONE-255.45-measure-declarations-and-drops.md` and its SCORE (class (iv), "the two gaps").
- `FINDING-declared-signatures-for-intrinsics.md` (probes p6, p10, p11).
- `src/check.rs:81-89`: `TypeScheme.type_params` is a plain `Vec<String>`. No bound can be written.
- `src/check.rs:17937` `instantiate`, `:17969` `instantiate_with_args`, `:16698` `unify`.
- `src/check.rs:~5615-5720`: defclause check-time dispatch (`None => true` at ~5668; rest zip at ~5641/5665;
  `src/check/env.rs:181` `has_rest`).
- `src/check.rs:13634` `is_type_orderable`, `:13505` `is_type_equatable`, `:13693` `infer_ordering`,
  `:13342` `infer_equality`, `:4093-4104` the ordering arm.
- `src/check.rs:4410-4424`, `12233-12280` (`select`/`poll`), `17549-17589` (same-head invariance, `Peer` exception).
- `src/check.rs:10066-10073` (`first`/`second`/`third`/`nth`).
- `src/check.rs:11230` `satisfies_spawned`; `src/types.rs:1723` `family_extends`; the 255.22 `extend-type`
  binder edges (`wat/spawn.wat:258-259` is a live example of a binder that names a parameter).

## Part A — does a family-bounded type variable close these rows?

The candidate capability: a clause's type variable may carry a bound, e.g.

```
(:wat::core::defintrinsic :wat::kernel::select
  ([peers <- (:wat::core::Vector :- [E])] :- [(E <- (:wat::spawn::Spawned :- [S R]))] -> …))
(:wat::core::defintrinsic :wat::core::<
  ([a <- T] [b <- T] :- [(T <- :wat::core::Orderable)] -> :wat::core::bool))
```

(The syntax is illustrative. Report the syntax the reader and the `extend-type` binder parser can already take.)

For each row, say **closed by a bound / closed by something else (name it) / not closed**, with the reason, measured:

1. **`<` `>` `<=` `>=`.** `is_type_orderable` is a *structural* class (it recurses into containers). List every arm
   of it. A family bound works only if every arm can be stated as family membership. Measure whether the family
   machinery (`extend-type`, `family_extends`) can say **conditional** membership: "`(Vector :- [T])` is
   `Orderable` when `T` is". If it cannot, say what is missing. Record the `TypeExpr::Var(_) => true` arm and what
   it means for a bound.
2. **`=` and `not=`.** 255.45 put them in class (i). Does `infer_equality` gate on `is_type_equatable`? If yes,
   they belong with ordering: measure the same way.
3. **`select`, `poll`.** With `E` bounded by `Spawned` (and by `Peer` for the peer clause), does unifying
   `(Vector :- [(Thread :- [S R])])` against `(Vector :- [E])` bind `E` and then satisfy the bound, **with no
   `Vector` covariance at all**? Measure by reading `unify`, and by a probe if you can. Then measure whether the
   corpus ever passes a **mixed** vector (a `Thread` and a `Process` in one `select`): can a vector literal even
   hold both today? If vectors are homogeneous, the bound replaces covariance; say so or refute it.
   `poll`'s listener argument: what clause states it?
4. **`first`, `second`, `third`, `nth`.** A bound does not obviously help. Measure what does: a clause per
   container in a finite set, a tuple type rule, an index-literal rule? For `nth` on a tuple with a non-literal
   index, what does today's arm return? Classify honestly, even if the answer is "a form".

## Part B — can unification carry a bound?

1. Where would a bound live: on `TypeScheme` (beside `type_params`), on the fresh variable in `InferCtx`, or on the
   substitution? Cite what exists.
2. When is it checked: when `unify` binds the variable, or after inference against the final substitution?
   Measure what `unify` does today when it binds a `Var`, and whether there is any hook at that point.
3. What happens when the variable is **still unresolved** at the end of the call? Relate this to gap p11 (an
   unresolved receiver silently takes the first clause): does a bound make that refusal natural, or is it still
   a separate rule?
4. Do user `defn`/`defclause` binders parse a bounded type parameter today, and are `defn` binders enforced at all
   (C-c says they are not)? A bound is not intrinsic-specific; measure whether one mechanism serves all three.

## Part C — the remaining gaps, sized as work

For each: the file:line that would change, and whether it is still needed once bounds exist.

1. Rest elements unchecked at check-time dispatch.
2. The unresolved receiver taking the first clause (`None => true`).
3. `Vector` covariance under surface satisfaction.

## Part D — the 12 fresh-return rows

`fresh-symbol`, `macro-error`, `str`, `struct-field`, `type-equal?`, `type-params-used-in`, `peer-pid`,
`linkedlist/contains?`, `empty?`, `get`, `length`, `metadata-of`. For each, read the runtime implementation and give
the type it actually returns, as a clause. Say which one cannot be given a clause, and why.

## Output

1. One table over **every function row 255.45 left uncovered** (class (iv) minus the form-shaped rows, plus the 12):
   row · what closes it · measured or inferred.
2. The **minimal capability list** `defintrinsic` needs for every function row to be declared, in dependency
   order.
3. Any form-shaped row you think is actually a function, or the reverse, with the reason.

## Doctrine

- Read, measure, cite file:line. Say measured or inferred for every claim.
- Capture `rc=$?` on the **next** statement; never `$?` inside a string containing `$(…)`.
- Scratch `.wat` probes go in the worktree's `wat-scripts/scratch-pad/`.
- If you run a floor in a worktree: `scripts/floor.sh`. There is no known flake; never re-run to green; quote any
  failing block verbatim.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- **STOP-1:** if a question here can only be answered by designing the bound's semantics (not by measuring what
  exists), write down the open design choice and its measured consequences, and stop there. Do not pick one.
- Write `SCORE-STONE-255.46-measure-what-defintrinsic-needs.md` on `main` beside this brief and commit **only** it
  (`git add -- <that path>`). Remove any worktree you created. **Do not push.**
