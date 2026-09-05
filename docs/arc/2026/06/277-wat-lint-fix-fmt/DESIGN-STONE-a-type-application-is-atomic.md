# DESIGN — STONE: a type application is atomic

> **Builder:** *"parametric arg-spec should be oneliner..."*
> ```
> [xs :- (wat.type/Vector :- [wat.type/i64])]
> ```
> *"this is how it should be rendered"*

## THE DEFECT — still visible in the output of the last stone

```
  [xs <- (:wat::core::Vector :-
           [:wat::core::i64])]          ⛔ a type torn across two lines
```

R11 fires on `(:wat::core::Vector :- [:wat::core::i64])`: `:-` is an atom and rides, `[…]` is a
compound and breaks. `:wat::core::HashMap` and friends carry no `@syntax`, so the slot mechanism
cannot reach them.

## THE RULE

> **A TYPE APPLICATION IS ATOMIC. It and everything inside it render on one line.**

Not "glue `:-` to its successor" — that leaves a nested type like
`(HashMap :- [(Vector :- [i64]) String])` free to break internally. **Atomic** is what the builder's
shape requires and it is simpler to state.

## ⛔ RECOGNISING ONE — measured, and the obvious predicate is WRONG

*"a list whose child 1 is `:-`"* matches **5,772 forms** corpus-wide, 90 distinct heads. The top of
the distribution is all types (`Vector` 3085, `Tuple` 733, `PersistentVector` 614, `HashMap` 263,
`Peer` 174, `Option` 172, …). **The tail is not:**

```
1  :wat::core::read-string        (:wat::core::read-string ":-")
1  :wat::eval-digest-string!      same class
1  :wat::core::fn                 wat/core.wat:1349 —
                                  (:wat::core::fn :- [~@binder-names-ch] ~params -> ~ret …)
```

- Two are **string literals**: `grep.wat` emits a `Named` fact for a `StringLit`, so the string
  `":-"` looks exactly like the symbol `:-`.
- One is a **generic `fn`'s param-spec** — a real form that the naive predicate would render entirely
  on one line.

★ **The tail is where the counterexamples were.** The top twenty were unanimous and would have
justified shipping the wrong predicate.
`[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]`

### The predicate that survives

> **A list with EXACTLY 3 children, whose child 1 is a symbol/keyword named `:-`, and whose child 2
> is a vector.**

```
(Vector :- [i64])                       3 children   ✓ TYPE APPLICATION
(fn :- [T] params -> R body)            7 children   ✗
(read-string ":-")                      2 children   ✗   (and child 1 is a STRING)
```

Arity alone excludes all three counterexamples; the kind check on child 1 is belt-and-braces against
a hypothetical 3-child form carrying the string.

## MECHANISM — atomic means the emitter renders it verbatim

The emitter already renders leaves with `ast->source`. **A type application becomes a leaf for layout
purposes**: emit `ast->source` and descend no further. No `Break` is asserted inside it, so no rule —
default or specific — can tear it.

⚠ **This is NOT a subtree claim returning.** Ownership answers *"which rule positions this node's
children"*. This answers *"does this node have laid-out children at all"* — and a type application
does not; it is a single rendered token as far as layout is concerned.

## THE ACCEPTANCE

```
1  ★ [xs <- (:wat::core::Vector :- [:wat::core::i64])]  renders on ONE LINE
2  ★ a NESTED type is also one line:  (HashMap :- [(Vector :- [i64]) :wat::core::String])
3  ★★ a generic `fn` is NOT collapsed — wat/core.wat:1349's shape still lays out normally
4  every fixture idempotent; ruled shapes hold; three walls stand
5  the count of type applications recognised is PRINTED — a green over zero proves nothing
```

Row 3 is the counterexample from the census, promoted to a gate.

## OUT OF SCOPE

- **The 120 lint.** A very long type application will now exceed it; that is the lint's business,
  and the builder ruled the exploded form over compression.
- **`Slot`** — landed last stone, unchanged. This is the case `@syntax` could not reach.
