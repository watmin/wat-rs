# wat-rs arc 047 — Vec accessors / aggregates return Option

**Status:** opened 2026-04-24. Second wat-rs arc post-known-good.
Surfaced by lab arc 018 (standard.wat) — the heaviest vocab port,
window-based, needs `find-last-index` and "max of these
projections." Builder caught a deeper question while sketching it:

> "should first errory on empty?... in ruby [].first -> nil -
> why isn't our first Option<T>?"

The answer: it shouldn't. **`first/second/third` on a Vec
should return `Option<T>`, matching Rust / Ruby / Clojure.**
The current "errors on empty" form is the Haskell `head` wart —
type-time tells the caller "give me a T," runtime might fail.
Honest signature is `Option<T>`.

**Scope:** the polymorphism shift for `first/second/third` plus
the natural-form gaps surfaced by the lab arc 018 sketch:

- `:wat::core::first` / `second` / `third` — **on Vec, change
  return type from `T` to `Option<T>`.** On Tuple, unchanged
  (always returns `T` — tuples are fixed-arity, type-known).
- `:wat::core::last` — **new.** `Vec<T> -> Option<T>`. Natural
  pair to `first`; arc 018 needed it for "current candle in
  window."
- `:wat::core::find-last-index` — **new.** `Vec<T> × fn(T)->bool
  -> Option<i64>`. Universal combinator (Rust `.iter().rposition()`,
  Clojure last-keep-indexed). Arc 018 has three callers (last
  RSI extreme, last vol spike, last large move).
- `:wat::core::f64::max-of` / `min-of` — **new.** `Vec<f64> ->
  Option<f64>`. Reduce a Vec<f64> to its max/min. Arc 018 needs
  both for window high/low aggregates. Empty input → `None`
  (max/min are undefined).

The unifying theme: **Vec accessors and aggregates return
`Option` to honestly signal emptiness or no-match.** Tuple
accessors stay `T` (defined-by-type). The two contexts split
cleanly along the type boundary.

---

## The polymorphism shift in detail

`first/second/third` are polymorphic over Tuple and Vec (per
arc-019 user direction "both are index-accessed data structs").
Today's split:

| Container | Index in range | Index out of range |
|---|---|---|
| Tuple | T | runtime error (impossible at compile time per type) |
| Vec | T | runtime error |

The Vec case fails the "honest signatures" test — empty Vec is a
runtime fact, but the signature lies and says "always T."

After arc 047:

| Container | Index in range | Index out of range |
|---|---|---|
| Tuple | T | unreachable (type-prevented) |
| Vec | `Some<T>` | `None` |

The polymorphic dispatch in the type checker (`infer_positional_accessor`
in `src/check.rs:1614`) already inspects the argument type and
matches Tuple vs Vec. The Vec branch's return type changes from
`T` to `Option<T>`. The Tuple branch is unchanged.

The runtime dispatch (`eval_positional_accessor` in
`src/runtime.rs:4225`) already matches `Value::Tuple` vs
`Value::Vec`. The Vec branch wraps the `cloned()` result in
`Value::Option` (`Some` for in-range, `None` for out-of-range)
instead of returning the cloned T directly.

---

## Why ship the polymorphism shift now (vs leave first/second/third
## as-is and only add new primitives)

Three arguments for shipping it now:

1. **Honesty before convenience.** The arc shipping new
   primitives (`last`, `f64::max-of/min-of`) takes the
   right-shape stance (`Option<T>` for Vec aggregates). If
   `first/second/third` keep the wrong shape, the substrate has
   two competing styles. New code that uses `first` doesn't
   compose with new code that uses `last` — different return
   shapes.

2. **Arc 018 surfaces the question naturally.** The lab is
   about to ship a window-vocab using `current = last(window)`.
   The natural answer is `Option<T>`. If we ship `last` as
   `Option<T>` but `first` as `T`-with-error, future callers
   find the inconsistency immediately.

3. **The migration cost is bounded.** ~82 wat-rs callers + ~210
   lab callers, but most are on **tuples** (unchanged). The Vec
   callers needing migration are sparser. The sweep is
   mechanical: each Vec-using `(:first xs)` either stays as-is
   (and now type-checks as `Option<T>`, requiring downstream
   `match`) or migrates to `(:get xs 0)` (already returns Option,
   identical semantics).

---

## What the new primitives ship

### `:wat::core::last`

Mirrors `first`'s shape on Vec post-polymorphism-shift:

```scheme
:wat::core::last : ∀T. Vec<T> -> Option<T>
```

Empty Vec → `None`. Non-empty → `Some(items[items.len() - 1])`.

### `:wat::core::find-last-index`

```scheme
:wat::core::find-last-index : ∀T. Vec<T> × fn(T) -> :bool -> Option<i64>
```

Iterates the Vec, returns `Some(i)` for the last `i` where the
predicate holds, or `None` if no element matches (or the Vec is
empty).

The `find-last-` prefix matches Rust's `.iter().rposition()`
intent. The `-index` suffix distinguishes from a hypothetical
`find-last` returning the matching element. Lab arc 018 needs
the index to compute `since-X = (n - last-idx)`, so index is the
right-shaped primitive.

### `:wat::core::f64::max-of` / `min-of`

```scheme
:wat::core::f64::max-of : Vec<f64> -> Option<f64>
:wat::core::f64::min-of : Vec<f64> -> Option<f64>
```

Reduce a `Vec<f64>` to its max / min. Empty → `None`. Non-empty
→ `Some(extreme)`.

Composes with `map` for the "max/min of these projections"
pattern: `(:f64::max-of (:map xs (lambda (x) (project x))))`.

Why placed in `:wat::core::f64::*` rather than `:wat::std::math::*`
or `:wat::core::*` (generic): the family matches `f64::+/-/*//`,
`f64::max`/`min` (binary, arc 046). `:wat::core::f64::*` is the
strict-f64 namespace; max-of and min-of are reductions in the
same family. A future `:wat::core::i64::max-of`/`min-of` would
mirror.

---

## Migration shape — wat-rs sweep

Each wat-rs `(:wat::core::first|second|third xs)` callsite:
- If `xs` is a Tuple: no change. Still returns T.
- If `xs` is a Vec: now returns `Option<T>`. Caller must `match`
  or assume non-empty by structural reasoning.

The sweep happens in arc 047's slice 3 — find every call,
inspect the type context, migrate.

A quick sample of likely Vec-call sites from the survey:

- `wat-tests/std/service/Console.wat:49` — `(:first stdout)`
  where stdout is `Vec<String>` from console capture. Migrate
  to match.
- `tests/wat_dispatch_e1_vec.rs:63,79` — explicitly Vec tests.
  Migrate.
- `tests/wat_hermetic_round_trip.rs:86` — `(:first lines)`
  where lines is captured stdout. Migrate.
- `tests/wat_names_are_values.rs:204` — `(:first collected)`
  where collected is a Vec.
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — most uses
  are on tuples (`:first state` where state is `(Body, Sender)`),
  but a few may be Vec. Inspect each.

Most other survey hits are tuple uses (pair, emission,
con-state, reply-pair) — no change.

---

## Why one arc, not two

Bundling the polymorphism shift + new primitives:
- One unifying narrative: **Vec accessors/aggregates return Option**.
- One sweep across wat-rs callers (the breaking change).
- One USER-GUIDE Forms appendix update.
- Lab arc 018 inherits a cohesive substrate state.

Splitting would produce a "polymorphism-only" arc plus a "new-
primitives-only" arc — two sweeps, two doc updates, two
INSCRIPTIONS. The bundling honors the one-coherent-decision rhythm.

---

## Non-goals

- **Sweep lab callers in this arc.** Lab is downstream (different
  repo, different INSCRIPTION trail). Arc 047 ships substrate
  + wat-rs sweep. Lab arc 018 sweeps lab-side callers as part
  of consuming the new primitives.
- **Generic `find-last`/`find` returning the matching element.**
  Arc 018 needs the index, not the element. If a future caller
  needs the element, open a small arc.
- **`max-by`/`min-by` (max/min via projection function).**
  Composes from `f64::max-of (map xs f)`. Add only if the
  composition becomes painful enough to warrant a primitive.
- **`first/second/third` overloading via type-class trait.**
  The current ad-hoc polymorphism (Tuple vs Vec) is intentional
  per arc 019. Adding a trait would be substantial substrate
  surgery — out of scope.
- **Sweep `(:get vec 0)` callers to use `(:first vec)` for
  uniformity.** Both now return `Option<T>` — caller picks
  whichever reads better. Don't churn for stylistic uniformity.
