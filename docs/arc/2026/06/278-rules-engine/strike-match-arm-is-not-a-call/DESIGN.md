# DESIGN — a `:then` walker that reads a match ARM as a constructor CALL

## Why

Work-list **D5**. `walk_nested_constructors` (`validate/mod.rs:774`) special-cases exactly one head —
`:wat::core::kwargs-construct` — and otherwise *"recurses into every item anyway"*. So it descends
into a `match` form's **arm patterns** as if they were value expressions:

```
:then [(:probe::Out :k ?k :ok (:wat::rete::core::match ?v (:probe::E::A true) (:probe::E::B false)))]
```

The arm `(:probe::E::A true)` has an enum-variant keyword at `items[0]`, so
`matcher::enum_variant_ctor` resolves it and the arity branch fires the variant's **0** declared
fields against the arm's **1** item. The diagnostic is `RhsArityMismatch` naming a `:then` insert of
`:probe::E::A` — **an insert that appears nowhere in the source.**

It survives only by coincidence. `((:probe::E::A) true)` puts a *List* at `items[0]`, keyword
extraction fails, and the form falls to the generic recursion untouched. **Whether a legal `match`
compiles in `:then` depends on which of two equivalent spellings the author picked**, and the same
expression is accepted unchanged in the `where` fence.

Banked and paired since 2026-08-30: `harness-experiri/experiri-then-match.wat` refuses,
`experiri-when-match.wat` loads.

## ⭐ THE ENUMERATION — DONE FIRST, AND IT DISCONFIRMED THE WIDER CLASS

The row names `match`. Three pattern-bearing forms exist (`match` 264 uses, `let` 485, `fn` 330), so
the strike opened by asking whether all three share the defect. **They do not, and the reason is
structural:**

`walk_nested_constructors` opens `let WatAST::List(items, span) = operand else { return }`. It walks
**Lists only**.

| form | pattern sits in | walked? |
|---|---|---|
| `let` | a **Vector** — `[k (:wat::core::ast-kind node)]` | no — returns immediately |
| `fn` | a **Vector** — `[cos <- :wat::core::f64]` | no — returns immediately |
| `cond` | a **List** — `((:wat::core::= …) 20)` | walked, but `items[0]` is a *call form*, so keyword extraction fails and it falls to generic recursion |
| **`match`** | a **List** — `(:probe::E::A true)` | **walked, and `items[0]` is a bare enum-variant keyword** |

**`match` arms are the only List position where a bare enum-variant keyword legitimately appears as
`items[0]` meaning something other than a constructor.** The class is one form, not three. Measuring
this before drawing is what kept the cure from being over-broad.

## The contract decision, pinned

**The walker recurses into a match form's SCRUTINEE and each arm's BODY, never into an arm's
PATTERN.**

An arm is `(pattern body…)`. The pattern is `items[0]` — a bare variant keyword
(`:probe::E::A`), a destructuring List (`(:wat::core::Some existing-id)`), or a literal (`true`).
None of those is a constructor call in that position. The body is `items[1..]` and **must** still be
walked, because a body can legitimately nest a constructor.

Head: `:wat::rete::core::match`, a `RETE_OPS` row (`vocabulary.rs:584`, `core_name`
`:wat::core::match`). **Which spellings are reachable in a `:then` operand is to be measured, not
assumed — an arm added for an unreachable spelling is a dead branch that no mutation can prove.**

## The repro is self-disposing, and that is part of the work

`experiri-then-match.wat` carries `rune:lint(red-by-design)` and says so in its own header: *"If this
file ever loads, D5 is cured and the rune must go with it."* Curing D5 makes that file load. It must
become a **regression gate** — the paired files then assert that both spellings compile and agree —
not a stale red-by-design marker whose rune has rotted.

## Files

- `src/rete/validate/mod.rs` — the walker.
- A gate carrying the paired repro.
- `docs/arc/2026/06/278-rules-engine/harness-experiri/` — the rune's disposition.

## Out of scope = REJECTED

- **D8** (`PersistentVector/length` unreachable as an accumulator head). The other L1 in the same
  banked harness, a different class — admits-by-one-registry / dispatches-by-another. Its own row.
- Teaching the walker `let`/`fn`/`cond`. **Measured immune above.** Adding arms for them would be
  three dead branches and three unprovable mutations.
