# NOTE — `:wat::core::when` does not exist, and adding it is a TYPE question, not sugar

**Filed 2026-08-02**, surfaced from arc 278 #56 (the rete form mirrors). Not scoped, not scheduled.
Recorded because the substrate currently carries a *classification for a verb that cannot be called*,
and because the obvious "just add Clojure's `when`" has a real design fork under it.

## What is actually on disk

`:wat::core::when` is **not registered anywhere.** Calling it:

```
#wat.runtime/UnknownFunction {:message "unknown function: :wat::core::when" …}
```

Its **only** trace in the whole tree is a purity classification — `src/rete/purity.rs:259`, in
`intrinsic_meta`'s pure list, sitting between `do` and `get`:

```rust
| ":wat::core::if"
| ":wat::core::let"
| ":wat::core::do"
| ":wat::core::when"      // <- classifies a verb that does not exist
```

That is an **unreachable arm**: a row nothing can ever consult, quietly asserting a fact about a verb
with no registration and no eval path. Per `[[feedback_an_unreachable_arm_accumulates_lies]]` it
should be deleted whether or not `when` is ever built — a classification is a claim, and this one
has no subject.

## ⚠ THE NAME IS ALREADY TAKEN BY SOMETHING ELSE

`:when` is a **`defrule` macro keyword** — and it marks the **LEFT**-hand side:

```clojure
(:wat::rete::defrule :weather::cold-and-windy
  :when [<cond1> <cond2> …]     ;; LHS — the conditions
  :then <insert1> <insert2> …)  ;; RHS — what fires
```

(`wat/rete.wat:2137-2139`; `wat/query.wat` decomposes rule values on the same `:when`/`:then` pair.)

It is pure syntax, consumed by the macro at expansion — not a verb, not callable, never inside a
`where`. **So `:when` (rule syntax, LHS) and a hypothetical `when` (one-armed conditional) are two
unrelated things sharing a name.** Any future `when` has to be weighed knowing a reader already
meets `:when` meaning "the conditions."

This also corrects the rete stone: `DESIGN-STONE-where-admits-only-rete-ops.md` prescribes minting
`:wat::rete::core::{if, let, do, when}`. That `when` is the Clojure macro, so the line asks for a
mirror **of nothing** — and minting it would put `:wat::rete::core::when` beside `defrule`'s `:when`
meaning something else entirely. The line should read `{if, let, do}`.

## The real question — one-armed conditionals in a typed language

Clojure's `(when test body…)` ≡ `(if test (do body…) nil)`. wat's `if` **requires both branches**
(3 args, enforced — `runtime.rs`: *"expected (:wat::core::if cond then else) — 3 args"*). So a
one-armed form must answer: **what type does it have when the test is false?**

That is not a convenience question. Three shapes, each honest, each with a different cost:

| shape | signature | cost |
|---|---|---|
| **(a) Option** | `(when test body…) -> (Option T)` | faithful to "maybe a value"; every caller unwraps, which is noise at the dominant call site |
| **(b) unit** | `-> :wat::core::nil` **always**; the body is evaluated for effect and its value discarded | clean, needs no union, and matches how `when` is overwhelmingly *used*. **Diverges from Clojure**, which returns the body's value — defensible under *"we are a clojure dialect, not a clojure impl"* (299) |
| **(c) statement-position sugar** | legal only where a value is discarded (a non-final `do` form) | no type question at all, but needs a position rule the checker must enforce — a new kind of constraint |

**What is NOT available: bare `T`-or-`nil`.** wat has no union type, and arc 179 has just finished
killing `()` precisely because *a second spelling of one thing is a second door around every wall
built on the first*. A `when` that is sometimes `T` and sometimes `nil` re-opens that shape at the
type layer.

**(b) is the one worth arguing for** if this is ever built — it is the only shape that is typed,
union-free, and matches the actual usage; and the divergence from Clojure's return value is a
deliberate, statable choice rather than an accident.

## Demand: ZERO, and structurally so

No corpus uses — there cannot be, since the verb does not exist. Nothing is blocked on it. **This
note is not an argument to build it**; it is the grounding so that whoever picks it up starts from
the type fork instead of re-deriving it, and does not mint a name that already means something else.

## Explicitly NOT a rete vocabulary candidate

A `where` is a **predicate** — it must yield `bool`. A conditional that yields `nil` (or an `Option`)
on the false branch has nothing to contribute there, and the rete fence would refuse it on the
totality/type axes anyway. If `when` is ever built it is a `:wat::core::` form only.

## Actionable now, independent of any decision

1. **Delete the phantom purity row** (`src/rete/purity.rs:259`). One line. It classifies nothing.
2. **Correct the rete stone's `{if, let, do, when}` → `{if, let, do}`** so #57 does not act on a
   prescription to mirror a verb that does not exist.

Both are small and are noted in arc 278 #57's scope.
