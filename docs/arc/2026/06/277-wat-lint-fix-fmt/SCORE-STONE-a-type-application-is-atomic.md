# SCORE — STONE: a type application is atomic (amended: declaration vs constructor)

No commit. Floor and clippy left to the orchestrator. Mid-strike AMEND applied: constructors glue type-args and explode values.

## The ruling

**Declaration (arity 3) — one line:**

```
[xs <- (:wat::core::Vector :- [:wat::core::i64])]
```

**Nested type inside type-args — still one line:**

```
[m <- (:wat::core::HashMap :- [(:wat::core::Vector :- [:wat::core::i64]) :wat::core::String])]
```

**Constructor — type-args glued, values exploded:**

```
(:wat::core::PersistentVector :- [:wat::core::i64]
  1
  2
  3)
```

`IDEMPOTENT=true` on all three.

## Count, printed

`claim-demo.wat`: `TYPE_DECLS=1 COLON_ARGS=1`
`type-ctor.wat`: `TYPE_DECLS=1 COLON_ARGS=2` (return-type decl + constructor)

Not a silent-empty predicate.

## Generic `fn` — not collapsed

```
(:wat::core::fn :- [:wat::core::i64] [acc <- :wat::core::i64 x <- :wat::core::i64]
  ->
  :wat::core::i64
  (:wat::i64::+ acc x))
```

`:- [T]` glued to the head (the AMEND said that is correct). Ret-spec and body on their own lines. Not a one-liner.

### Named: Slot index vs generic `fn`

Glue withholds child 2. The previous stone's registry Slot still says `fn` glued=3 — which is the ret-type **in the grammar without `:- [T]`**. On a generic `fn` that extra slot shifts indices, so glued=3 withholds the **params** instead of `->`'s type. Glue alone cannot re-index `fn`'s grammar; Slot is the honest place, as the AMEND said. Not a STOP: row 4's gate (not collapsed; ret-spec own line; body own line) holds.

## Previous stone

`foldl-bare.wat` still renders `-> :wat::core::i64` on one line.

## Walls

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` 0. `col` 0 in every rule file.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `run-types.wat` on `claim-demo` / `type-ctor` | counts printed, non-zero |
| declaration / nested / constructor fixtures | ruled shapes, **IDEMPOTENT=true** |
| `generic-fn.wat` | not collapsed |
| `foldl-bare.wat` | ret-spec one line |
| remaining fixtures | ruled + idempotent |
| `run.wat` on `wat/io.wat` | **COMMENTS=28** |
| kind-conflict sabotage | **raises** |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-05

**ACCEPTED for what it delivered. ⛔ AND THE NON-NEGOTIABLE IS VIOLATED ON ONE SHAPE** — reported
by the strike itself, and my acceptance row let it through.

| what | result |
|---|---|
| ★ a type DECLARATION is one line | `[xs <- (:wat::core::Vector :- [:wat::core::i64])]` ✓ |
| ★ a NESTED type stays inline | `(HashMap :- [(Vector :- [i64]) String])` ✓ |
| ★ a CONSTRUCTOR glues and explodes | `(PersistentVector :- [i64]` then `1` `2` `3` ✓ |
| the counts printed | `TYPE_DECLS=1 COLON_ARGS=2` — not a silent-empty predicate |
| ⛔ **a generic `fn`'s ret-spec** | **SPLIT ACROSS TWO LINES** |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** · clippy **0** |

```
    (:wat::core::fn :- [:wat::core::i64] [acc <- :wat::core::i64 x <- :wat::core::i64]
      ->
      :wat::core::i64          ⛔ "ret-spec is a single line... i will not accept otherwise"
      (:wat::i64::+ acc x))
```

## ★ THE STRIKE FOUND AND NAMED THE CAUSE. MY ROW LET IT PASS.

From the SCORE: *"On a generic `fn` that extra slot shifts indices, so glued=3 withholds the params
instead of `->`'s type."*

Exactly right. `fn`'s grammar is `(fn [params] -> :RetType body)` → `->` at 2, type at 3, so
`Slot glued=3`. A GENERIC `fn` has `(fn :- [T] [params] -> :RetType body)` — **the optional
param-spec shifts everything by two**, `->` is really at 4 and its type at 5. `glued=3` withholds
the wrong child.

⛔ **And my EXPECTATIONS row 4 said *"ret-spec on its own line"* — which `->` and the type each
having their own line satisfies.** The RULING is *"the ret-spec is a SINGLE line"*. The strike read
my row, not the ruling, and was right to. **The ambiguity is mine.**
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## ✅ THE CURE — and it is rule maturity, not architecture

> **Builder:** *"we just need to mature the rules - we have the full rete expression system to
> manage our complexity."*

The defect is index arithmetic against a grammar with an OPTIONAL slot. The cure removes the
arithmetic:

```
TODAY   Slot {head, glued}   — an index computed from the GRAMMAR
        breaks the moment a form has an optional slot the grammar's index does not account for

CURE    find the `->` CHILD IN THE FORM ITSELF; glue the child after it.
        No index. No arithmetic. Immune to any optional slot, present or future.
```

★ **This is the third independent piece of evidence for the lexical rule**, and the first two are
already on the record: the registry route yields only 3 slots, and it cannot reach type applications
at all. Now it also mis-indexes the one form it was built for.

⚠ **The registry work is not wasted and should not be torn out.** It proved the grammars parse, it
proved the head-spelling hazard real, and `Slot` remains the right home for anything a grammar knows
that syntax alone cannot. **But for `->`, the form is a better authority than the grammar** — because
the form is what is actually being laid out.

## Not disputed

The AMEND landed correctly: declaration, nested type and constructor all match the builder's three
examples, and all are idempotent. `foldl-bare.wat`'s ret-spec still renders on one line (the
NON-generic case). The three walls stand — disagreeing-kind sabotage raises, `ClaimedUnder` 0,
`col` 0 across every rule file. `wat/io.wat` still **COMMENTS=28**.
