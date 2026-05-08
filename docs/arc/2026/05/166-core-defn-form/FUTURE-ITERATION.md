# Arc 166 — Future iteration ladder

**Status:** captured 2026-05-08 alongside arc 166 closure. Not a
commitment to ship; a record of the iteration path the user
sketched mid-arc-166. New arcs open when the user directs.

## End-state shape (user vision, 2026-05-08)

```scheme
(:wat::core::defn
  :user::add5-to-2-nums              ;; name keyword
  [x <- :wat::core::i64
   y <- :wat::core::i64]              ;; arg-vector with <- "consumes" arrow
  -> :wat::core::i64                  ;; top-level -> "produces" arrow
  (:wat::core::+ 5 x y))              ;; body
```

User direction (verbatim):

> "my justification for `[x <- i64]` is that its reverse of the ret
> type, its the accept type... `<-` consumes, `->` produces."

The semantic dual: `<-` and `->` arrows point FROM the type TOWARD
the named slot. Args have `<-` (slot consumes from a value source);
returns have `->` (slot produces to a value sink). Once you see the
symmetry, reading any wat function definition becomes mechanical.

## Where we are now (post-arc-166)

Defn ships with the current nested-sig shape inherited from `fn`:

```scheme
(:wat::core::defn :user::add5-to-2-nums
  ((x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
  (:wat::core::+ 5 x y))
```

Functional, type-safe, recursive-capable, reflection-visible. Path A
(macro composition) is locked in per arc 166 INSCRIPTION.

## Iteration ladder (Path B from the design conversation)

User direction 2026-05-08 confirmed Path B (defn evolves; def stays
as value-binding; fn stays as anonymous-function constructor):

| Future arc | Substrate work | Why it earns its keep |
|---|---|---|
| ≥A | Mint `<-` lexer recognition + `[name <- :T]` binding-vector parser shape | Substrate vocabulary; foundation for the new arrow semantics |
| ≥B | Extend `:wat::core::defn` to accept the flat shape `(defn name [args] -> :T body)` alongside the legacy nested-sig | Composition stays; surface gets ergonomic |
| ≥C | Extend `:wat::core::fn` similarly — anonymous fn gets the flat shape | Keeps consistency: both forms speak the same arrow language |
| ≥D | Migration sweep: flip wat sources from nested-sig → flat-shape | Single canonical form across the codebase |
| ≥E | Retire the legacy nested-sig | Closes the migration |

Each arc is its own ship. Each arc is testable in isolation. The
user sets pace and order; arc 166's INSCRIPTION does not commit to
any of these.

## Substrate readiness check (audited 2026-05-08)

- ✓ `wat-edn` already parses `[...]` as `Value::Vector` at the EDN
  layer (`crates/wat-edn/src/value.rs:50`). The wat language doesn't
  currently consume vectors as syntax — they parse but downstream is
  unused-or-erroring. Earliest arc must wire vector-consuming logic
  into the def/defn/fn signature parser.
- ✗ `<-` is not a token in the lexer. Earliest arc adds it.
- ✗ Top-level `->` placement (sibling of `[args]`, not nested) is
  not currently accepted at the def-form level. Earliest arc handles
  it.

## Open design questions (for whichever arc opens these)

1. **`<-` vs `:`** — should the legacy nested-sig `(name :T)` ALSO
   evolve to use `<-`, or stay with `:`? Consistency vs migration
   surface. Lean: keep `:` for nested-sig (legacy form) and use
   `<-` only inside `[...]` vector binding (new form). Migration
   pressure naturally drives consumers to the new form.
2. **Vector binding in `let`** — once `[name <- :T]` is the typed
   binding shape for function args, does it extend to `let`?
   Arc 159 made `let` untyped (`(let (name expr) ...)`); a typed
   variant `[name <- :T] = expr` could re-introduce optional
   annotations. Probably scope-out unless a workload demands.
3. **Multi-arity (`defn-clause`) shape under flat sig** — Erlang-
   style multi-clause: `(defn-clause name [args1] -> :T body1
   [args2] -> :T body2)`. The flat shape makes the pattern more
   readable than nested-sig.
4. **Closure capture explicitness** — `[x <- :T]` could extend to
   capture annotations: `[#capture y <- :T x <- :T]`. Probably
   over-engineering until a workload demands.

## What this document is NOT

- A commitment to ship any of the listed arcs.
- A blocker on shipping unrelated arcs that touch defn / fn / def.
- A spec — design happens IN the arc that opens, not here.

This document is a record-of-direction so future-us doesn't lose
the user's mid-arc-166 design intuition. The arrows-as-duals
insight (`<-` consumes, `->` produces) is the load-bearing piece —
without it, the iteration ladder loses its load-bearing
justification.

## Cross-references

- **Arc 166** (this arc) — defn ships in the current nested-sig
  shape. Macro composition over def + fn established.
- **Arc 159** — `let` per-binding `:T` retired. Same lesson the
  flat shape would apply at the function-arg layer if/when arc B+
  opens.
- **Arc 155** — `:wat::core::fn` mint. Flat shape extension would
  touch fn's sig-parsing to accept both shapes.
- **Arc 157** — `:wat::core::def` mint. Flat shape would NOT touch
  def (which stays as `(def name expr)` value-binding).
- **`crates/wat-edn/src/value.rs`** — vector support exists at EDN
  layer; ready for wat-language consumers.
