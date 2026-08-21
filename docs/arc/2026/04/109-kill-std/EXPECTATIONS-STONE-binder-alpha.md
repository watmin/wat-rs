# EXPECTATIONS — arc 109, binder strike α

Written BEFORE the strike, against `248240dec`. Scored after the orchestrator's own re-run.

## Scorecard

| # | what | expected |
|---|---|---|
| 1 | `(defenum :user::E :- [T] :wat::enum::Pure :A [f :- T])` | checks |
| 2 | `(typealias :user::A :- [T] (:wat::core::Vector :- [T]))` | checks |
| 3 | `(defsurface …)` · `(newtype …)` · `(typeunion …)` with a binder | check |
| 4 | ★ `(defrecord :user::Box<T> [item <- T])` still builds | `#user/Box {:item 42}` — **the additive control** |
| 5 | ★ every `<T>`-spelled declaration in `wat/` still loads | floor green |
| 6 | ★ `T` is still a VARIABLE, not a concrete type | a binder-declared field accepts an `i64` |
| 7 | both spellings at once → error | `(defrecord :user::Box<T> :- [U] [f :- T])` refused, naming the contradiction |
| 8 | `:- [:a :b]` → error | keyword values are not binder names |
| 9 | `:- [U [F :-> T]]` → error | a function type is not a binder name |
| 10 | `:- [T [f :- T]]` → error | a field vector is not a binder name |
| 11 | floor | **0 FAIL** |
| 12 | clippy `-D warnings` | 0 |

Row 6 is the one that matters and the one that can pass hollowly: `parse_declared_name`'s whole job
is turning `T` into a *variable*. A binder that populates `type_params` with the right strings but
reaches the wrong place would leave `T` a concrete type — and the symptom is
`"expects :T; got :wat::core::i64"`, which reads like a normal type error. **Verify by running a
binder-declared record with a concrete value, not by reading the parser.**

## Independent prediction

**20–30 minutes.** One new fn, seven near-identical call sites, and the validation predicate already
exists. The risk is not difficulty; it is row 6 passing hollowly.

## Trap-doors

1. **`type_params` populated but unused.** Rows 1–3 can go green on the parse alone. Row 6 is the
   only row that proves the params reached the type env.
2. **`"$bound/T"` in storage.** Would make row 6 fail confusingly and encodes the artifact 251.8b
   removes. `identifier.rs:145` says the namespace is derived today and stored after 8b.
3. **A hand-rolled binder check** instead of `is_reference()` — passes every row, and re-creates the
   four-way duplication 251.8a deleted. Verify by reading the code, not the floor.
4. **`.peekable()` changing an existing arity error.** Several parsers count `args.len()` for their
   "expected N args" diagnostics BEFORE iterating. If the binder is consumed after that count, a
   binder-bearing form may hit a stale arity message. Row 7's error must name the contradiction,
   not an arity.
5. **`parse_aggregate` serves two heads** (`recordtype`, `aggregatetype`). Both must keep working;
   only one is exercised by an obvious probe.

## Mode B

Any of: a `<T>` form stops parsing · `parse_declared_name`'s `<…>` path changed · a `.wat` file
edited · `defn` touched · a fifth hand-rolled is-a-binder check · cargo run by the rider.
