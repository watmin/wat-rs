# Iteration Patterns

**Wat has no `loop`, no `while`, no `for`, no `recur`. This is intentional.**

This doc names the canonical pattern for every "do work repeatedly"
shape, and the reasoning behind each choice. The substrate is
engineered so the path-of-least-resistance is the path-we-want;
ambiguity-by-construction is rejected.

---

## The principle: one canonical path per task

Every iteration shape maps to exactly one wat construct. There is
no second way. Synonyms are LLM-hostile (an AI co-author picks
inconsistently across files); minimal forms are LLM-friendly
(zero ambiguity about which form to pick).

| Task | Canonical form |
|---|---|
| Iterate over a collection (side effect) | `for-each` |
| Transform & collect | `map` |
| Reduce to a value | `foldl` (left) / `foldr` (right) |
| Filter / select | `filter` |
| "Do N times" | `(for-each (fn ...) (range 0 N))` |
| Iterate to fixpoint with state | `defn` + tail call (TCO) |
| Lazy / infinite / paginated | `Stream::lazy` thunks (arc 118 — when shipped) |

Memorize this table. If your need maps to a row, use that form. If
it doesn't, you probably have a different problem (and the next
section addresses it).

---

## The patterns

### 1. Iterate over a collection (side effect)

```scheme
(:wat::core::for-each
  (:wat::core::fn [item <- :Item] -> :wat::core::nil
    (:my::log item))
  items)
```

When you want to walk a collection and produce side effects (log,
publish, write) without collecting transformed values.

### 2. Transform & collect

```scheme
(:wat::core::map
  (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
    (:wat::core::* x 2))
  numbers)
```

When you want a new collection of transformed values. `map` is
pure; the input collection is unchanged.

### 3. Reduce to a value (`foldl` / `foldr`)

```scheme
;; sum a list of i64
(:wat::core::foldl
  numbers
  0
  (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
    (:wat::core::+ acc x)))
```

`foldl` walks left-to-right (accumulator-first); `foldr` walks
right-to-left. Pick by the operation's associativity / direction
sensitivity.

### 4. Filter

```scheme
(:wat::core::filter
  (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
    (:wat::core::> x 0))
  numbers)
```

When you want a sub-collection of elements matching a predicate.

### 5. "Do N times"

```scheme
(:wat::core::for-each
  (:wat::core::fn [_ <- :wat::core::i64] -> :wat::core::nil
    (:my::tick))
  (:wat::core::range 0 N))
```

`(range 0 N)` produces `[0 1 2 ... N-1]`. Combined with `for-each`,
this is the canonical "do N times" shape. The index `_` is
discardable (or named if you need it inside the loop body).

### 6. Iterate to fixpoint with state (`defn` + tail call)

```scheme
;; countdown — accumulate while a condition holds
(:wat::core::defn :user::sum-to
  [n <- :wat::core::i64
   acc <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64
    acc
    (:user::sum-to (:wat::core::- n 1) (:wat::core::+ acc n))))
```

When the iteration's termination condition is dynamic (not just
"N times" or "for each in collection"), use a recursive `defn` with
a tail call. Wat has TCO — the stack does not grow regardless of
iteration depth.

The tail call must be the LAST expression in the body. Wrapped in
arithmetic / `if` branches is fine; the recursive call must be in
tail position (the value-producing position).

### 7. Lazy / infinite / paginated (`Stream::lazy`)

```scheme
;; pagination — yield items, defer next batch until consumer demands it
(:wat::core::defn :user::paginated
  [client <- :Client
   query <- :Query
   token <- :Option<:Token>]
  -> :Stream<:Item>
  (:wat::core::let
    [resp (:client::get-batch client query token)
     items (:Stream::from (:resp::items resp))
     next (:resp::next-token resp)]
    (:wat::core::match next
      [:wat::core::None
        items]
      [(:Some _)
        (:Stream::concat
          items
          (:Stream::lazy
            (:wat::core::fn [] -> :Stream<:Item>
              (:user::paginated client query next))))])))
```

The recursive call lives inside `(Stream::lazy (fn [] ...))` — a
thunk that's only forced when the consumer demands more. No stack
growth (the thunk is stored as a value, not called immediately).

**Status**: `Stream::lazy` thunks are arc 118 (DESIGN settled,
implementation deferred). Until shipped, paginated patterns use
`:wat::kernel::spawn`-based generators (heavier; thread per
generator).

---

## The non-patterns (and what to use instead)

### `loop` / `recur` — NOT IN WAT

```scheme
;; ❌ NOT WAT — this doesn't exist
(loop [acc 0 i 0]
  (if (< i 10)
    (recur (+ acc i) (inc i))
    acc))
```

**Use instead**: `defn` + tail call (pattern 6 above). The named
recursive function carries the same semantics with no new construct.
Names ARE documentation; the `defn` form is profileable, testable
in isolation, debuggable by stack trace.

Why no `loop`/`recur`?
- It's a synonym for what `defn` already does
- Clojure has it because the JVM doesn't TCO; we have native TCO
- Synonyms are LLM-hostile — an AI co-author picks inconsistently

### `while` — NOT IN WAT

```scheme
;; ❌ NOT WAT
(while (< i 10) ...)
```

**Use instead**: `defn` + tail call. The "condition holds" check
becomes the `if` test in the recursive function; the body's update
is the recursive call's args.

Why no `while`?
- `while` requires mutation (changing `i`); wat is mutation-free
- The pure-functional equivalent IS `defn` + tail call
- Adding `while` would need adding `set!` which would break the
  algebra-immutable doctrine

### Anonymous local recursion — NOT SUPPORTED

```scheme
;; ❌ NOT POSSIBLE — fn body parsed before fn-name binding
(:wat::core::let
  [iter (:wat::core::fn [x <- :T] -> :R
          (:wat::core::if ... (iter ...) ...))]
  (iter 0))
```

The `fn` body is parsed before `iter` is bound; the recursive
reference can't resolve.

**Use instead**: `defn` (top-level named recursion). If your
function deserves to recurse, it deserves a name. Naming makes the
intent visible to future-you and to LLM co-authors.

Why no anonymous local recursion?
- Forces meaningful naming — names are documentation
- Recursive functions are usually load-bearing enough to deserve
  a name; anonymous ones tend to hide intent
- Top-level `defn` is profileable / testable / debuggable;
  anonymous local recursion is not

### `letrec` — NOT IN WAT

Mutual recursion via `letrec` is a Scheme/Clojure construct that
binds a group of names where each can reference the others.

**Use instead**: top-level `defn` for each function. Mutually
recursive functions reference each other through the module
namespace.

Why no `letrec`?
- Empirically rarely needed (5+ years of serious Clojure
  architecture experience without reaching for it)
- Top-level `defn` covers the case; the names are visible at
  module scope and can call each other freely
- `letrec` adds binding-shape complexity for a feature that
  rarely earns its keep

### Mutation-based iteration — NOT POSSIBLE

```scheme
;; ❌ NOT WAT — no set!, no mutable bindings
(let [i 0]
  (while (< i 10)
    (set! i (+ i 1))))
```

Wat has no `set!`, no `var`, no mutable bindings. The algebra is
immutable by design (see FOUNDATION § "The Algebra Is Immutable").
Mutation would break the substrate's value-oriented contract.

**Use instead**: pure-functional patterns. The new state is the
return value of the recursive call (pattern 6) or the accumulated
value of `foldl` (pattern 3).

---

## Why these constraints

Wat is engineered LLM-first. The brutal honesty + minimal forms +
one-canonical-path-per-task is purposeful pedagogy for AI
co-authors. The substrate is shaped so that:

- The path-of-least-resistance IS the path-we-want
- Synonym features are rejected (an LLM picks inconsistently
  across files; mixed-style codebases are harder to maintain)
- Names ARE documentation — force `defn` for recursive functions
  so every iteration is named, traceable, profileable
- Constraints reduce decision surface — fewer constructs to learn,
  fewer footguns to step on

Memorize the seven canonical patterns above. If your need fits a
row, use that row. If your need DOESN'T fit a row, you have a
different problem; surface it before reaching for a workaround.

---

## Cross-references

- `docs/FOUNDATION.md` (in the wat language spec) — the algebra-
  immutable section that rules out mutation-based iteration
- Arc 118 — `Stream::lazy` thunks for lazy/paginated patterns
  (DESIGN settled, implementation deferred)
- `:wat::core::for-each` — operator-position special form
- `:wat::core::map` / `:wat::core::filter` / `:wat::core::foldl` /
  `:wat::core::foldr` — sequence stdlib forms
- `:wat::core::range` — produces a sequence of integers
- `:wat::core::defn` — named function (arc 166)
- TCO — implemented in `eval_let_tail` / `eval_call_tail` paths
  in `src/runtime.rs`

---

*Wat doesn't take loops away to make life harder. It takes them
away because every iteration shape is already covered by something
simpler and more honest. Use the seven patterns. Name your
recursion. Trust the substrate.*
