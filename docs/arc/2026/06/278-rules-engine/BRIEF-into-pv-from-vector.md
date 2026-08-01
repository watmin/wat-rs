# BRIEF — `into (PersistentVector, Vector)` + retire the nine hand-rolled `vec->pvec` folds

Give `:wat::core::into` its missing fourth clause, backed by the native bulk extend that already
exists, then delete the interpreted workaround from the nine grid axes. Design + full grounding:
`DESIGN-STONE-into-pv-from-vector.md` (read it first — the RED gate and the mechanism are proven
there, not assumed).

## Read in order — the rooms, and why you are being sent to each

1. **`src/check.rs:18628-18640`** — the `:wat::core::Vector/concat` `TypeScheme` registration
   (`∀T. Vec<T> × Vec<T> -> Vec<T>`). This is the shape to MIRROR for the new op. Register
   `:wat::core::PersistentVector/concat` beside it with TWO schemes' worth of coverage:
   `PV<T> × Vector<T> -> PV<T>` and `PV<T> × PV<T> -> PV<T>`.
2. **`src/rete/kernel.rs:3620-3632`** — `eval_insert_all_native`'s single `vector_concat_inner(
   facts_val, &new_facts_vec)` call, where `facts_val` IS a PersistentVector. This is your proof
   the runtime already does the right thing with a PV receiver; the new op is a thin wrapper over
   the same fn.
3. **`src/runtime.rs:5174`** — the `":wat::core::Vector/concat" => …eval_vector_concat(…)` dispatch
   arm. Add the `PersistentVector/concat` arm beside it.
4. **`src/runtime.rs:9672`** — a SECOND `Vector/concat` site (`ceval::vector_concat_inner`) in what
   appears to be a const/compile-time eval path. Determine whether the new op belongs there too and
   say which way you went and why. Do not cargo-cult it in without reading what that path is.
5. **`wat/seq.wat:99-110`** — `into`'s `defclause`, three clauses today. Add the fourth:
   `([to <- PV<T> from <- Vector<T>] -> PV<T> (:wat::core::PersistentVector/concat to from))`.
6. **`wat-scripts/perf/grid/*.wat` (nine files)** — each defines its own `vec->pvec` as a
   `foldl`+`conj`. Re-point each body to `(:wat::core::into (:wat::core::PersistentVector) v)`.
   Keep the function name and signature; only the body changes. (Nine files with one shared shape —
   if you prefer a `wat-scripts/fixes/` codemod per the corpus doctrine, that is welcome, but nine
   one-line bodies is also an honest hand-edit. State which you did.)

## Implementation sketch — fill this in, do not invent a different shape

```rust
// src/runtime.rs, beside the Vector/concat arm
":wat::core::PersistentVector/concat" =>
    crate::collection::eval::eval_vector_concat(args, list_span, env, sym),
```
```clojure
;; wat/seq.wat — into's fourth clause
([to <- :wat::core::PersistentVector<T> from <- :wat::core::Vector<T>] -> :wat::core::PersistentVector<T>
  (:wat::core::PersistentVector/concat to from))
```

Verify `eval_vector_concat` returns a PersistentVector when handed one (it delegates to
`vector_concat_inner`); if it coerces to a Vector, that is the real work of this stone — fix it at
`vector_concat_inner` so the receiver's kind is preserved, and say so.

## The RED gate — it is live right now, reproduce it before you change anything

```
(:wat::core::into (:wat::core::PersistentVector 1 2) (:wat::core::Vector :wat::core::i64 3 4))
```
Today: `NoMatchingClauseAtCallSite` listing the three existing clauses.
After: returns `#wat.core/PersistentVector [1 2 3 4]`.

Add a permanent `deftest'` for it in the corpus (a Vector source AND a PV source, and the
receiver-kind assertion: the result must be a PersistentVector, not a Vector).

## Blast radius

`src/check.rs`, `src/runtime.rs`, `wat/seq.wat`, the nine `wat-scripts/perf/grid/*.wat`, plus one
new test. No new types. Do NOT touch `Vector/concat`'s existing scheme, `insert-all'`, the rete
kernel, or anything that computes `:native-ns`.

## STOP triggers — these are rejection criteria; ship nothing and report

- **STOP-1** — if `vector_concat_inner` does NOT preserve a PersistentVector receiver and making it
  do so would change `insert-all'`'s behaviour, STOP. That is a shared-mechanism change with a
  differential behind it and it is the orchestrator's call, not a thing to work around.
- **STOP-2** — if adding the clause makes any EXISTING `into` call site newly ambiguous
  (`NoMatchingClause` becoming a multiple-match), STOP and report the sites. Do not disambiguate by
  reordering clauses.
- **STOP-3** — if any grid axis's `:derived` output changes by even one element after the sweep,
  STOP. The whole point is that this is observationally inert; a changed derived set means the
  workaround was not equivalent and that is a finding, not a fix.

## Gates you run yourself before reporting

- `cargo nextest run --release` — read the **Summary line**, never a piped exit code.
- `cargo clippy --release --workspace --all-targets -- -D warnings` — must be 0.
- `target/release/wat --check` on each of the nine touched grid files.

Do not commit, do not push, do not stash. The orchestrator weighs and lands.
