# FINDING — the reachability ledger enumerates its ROW axis from a type and its POSITION axis by hand

**Measured 2026-09-10 at `07eba8226`.** Every row below driven against `./target/release/wat`, not
read.

> # ⛔ AMENDED WITHIN THE HOUR — THE TITLE'S CLAIM IS STRUCK. THE COVERAGE TABLE STANDS.
>
> **"The position axis was chosen, not derived" is WRONG as a criticism, and I filed it without
> reading the record.** The axis was chosen **with an argument, at the site, and it is tracked.**
>
> `src/rete/reachability.rs:78-85`, verbatim: a `:from` clause *"its operand is the accumulated
> COLLECTION rather than a field, and its evidence is a differential against the wrapped control
> rather than a hit/miss discrimination. Stretching `Cell` over it would have made `synth` render a
> shape it cannot hold."* And: *"`:then` … remains deliberately unmodelled, with its own separate
> defect. An un-calibrated position would add a column of findings nobody can trust."*
>
> `docs/arc/2026/06/278-rules-engine/RETE-OPEN-WORK.md:432`: *"The `:then` position and the user
> accumulator fold — the other two call sites the vocabulary's module doc names — are deliberately
> NOT modelled yet… Each new position needs its own known-answer control first."*
>
> ⛔ **And the "eleven variants" framing is struck too.** `ReteClauseShape`'s variants are clause
> SHAPES (`Bind`, `And`, `Unrecognized`…), not places an OP can sit. The vocabulary module doc names
> **four** call sites; the ledger models two and tracks the other two. **2-of-4-with-a-tracker**,
> not 2-of-11-blind. Deriving `CallSite` from `ReteClauseShape` would be a category error.
>
> **⭐ WHAT SURVIVES, AND IT IS A DIFFERENT INSTRUMENT ANSWERING A DIFFERENT QUESTION.** The
> coverage table below is correct and driven. `accumulate`'s `:from` clauses are not
> **type-checked by the VALIDATOR** — which is the D10/D11/fence class, not the ledger's
> reachability question at all. **My finding conflated two instruments.** Grepped: that validator
> hole is tracked NOWHERE. That is the whole live finding, and it is one stone.
>
> **⭐⭐ THE LESSON, AND IT IS THE SECOND TIME TODAY.** Hours earlier I filed stone 251.8d's
> deliberate mid-campaign boundary as a "gap," and was corrected. Then I did it again, to an
> argued exclusion carrying its reason in a doc comment eight lines long. **A narrow instrument is
> not evidence of an oversight; the record says which it is, and the record is cheap to read.**
> Both times I measured carefully and read no design docs. Sibling of
> `[[an-inherited-work-list-is-a-claim]]` inverted: a *deliberate* scope cut also reads exactly
> like an omission from the code alone.

## The coverage table

One ill-typed predicate — `(:wat::rete::core::string::= ?k "nope")` where `?k` is bound to an
`i64` field — placed in each position a rule admits:

| position | verdict | in `CallSite`? |
|---|---|---|
| inline constraint | ✅ refused | ✅ `InlineConstraint` |
| `where` fence | ✅ refused (cured `6f1d93451`) | ✅ `WhereFence` |
| `not` inner | ✅ refused | ❌ |
| `exists` inner | ✅ refused | ❌ |
| `or` arm | ✅ refused | ❌ |
| `and` arm | ✅ refused | ❌ |
| `:then` RHS | ✅ refused (D10, cured 2026-09-02) | ❌ |
| ⛔ **`accumulate` `:from`** | ⛔ **ACCEPTED — starts up clean, rc=0** | ❌ |

⭐ **Six positions turn out to be correctly checked** — `not`/`exists`/`or`/`and` inherit validation
by recursion through `validate_when_entry`. The substrate is in better shape than the pattern
suggested. **One position is open.**

## The open position

`src/rete/validate/mod.rs:296`, three arms below the fence arm cured on 2026-09-10, carrying the
same `Design call 3` sentence:

```rust
// Design call 3 — accumulate's `:from` inner gets fact-type-HEAD validation only;
// its own clauses and the acc-form's reducer body are out of scope.
ReteClauseShape::Accumulate { from, .. } => {
    validate_fact_type_head_only(from, rule_name, types, errors);
}
```

`validate_fact_type_head_only` (`src/rete/validate/typing.rs:146`) checks exactly one thing: that
the fact type is registered. No clauses, no binds, no field refs.

⛔ **And `:wat::rete::accumulate` has ZERO uses in the entire `.wat` corpus** — `grep` over every
`.wat` outside `docs/` returns nothing. A declared surface nothing exercises, so no corpus-driven
method could ever have reached it.

## ⛔ THE STRUCTURAL FINDING — one axis derived, one axis chosen

`src/rete/reachability.rs:87`, the (row × call-site kind) ledger completed 2026-08-28 with all 74
rows verdicted and **six genuine defects found**:

```rust
enum CallSite {
    InlineConstraint,
    WhereFence,
}
```

**Two.** Meanwhile `ReteClauseShape` (`src/rete/clause.rs`) declares **eleven** variants — `Bind`,
`Constraint`, `And`, `Or`, `Not`, `Exists`, `Where`, `Accumulate`, `FactBind`, `Predicate`,
`Unrecognized`.

⚠ **This is not a criticism of the ledger.** It did its declared job well, on an axis it derived
honestly, and it found six rows that passed every static gate and could not execute. The defect is
that **the row axis comes from a table and the position axis comes from a judgement** — so the
ledger can be exhaustive and blind at the same time, and its green says nothing about the six
positions it never asks about. `peragrare`: *a green from an instrument that was never asked is
silence, not proof.*

## Why this is the class, not an incident

Four instances now, all the same shape — *a checker that stops at a position, with a comment
explaining why*:

| | position | found |
|---|---|---|
| D10 | `:then` RHS field values | 2026-09-02, cured |
| D11 | nested `:then` | 2026-09-03, cured |
| — | `where` fence interior | 2026-09-10, cured |
| — | `accumulate` `:from` | 2026-09-10, **open** |

Each was found by accident or by hand. None was found by the instrument built to find exactly this.

## ⭐ The cure tops the ladder, because the missing enumeration already exists as a type

`ReteClauseShape` **is** the position list. If `CallSite` were derived from it and matched
exhaustively, **adding a clause shape would not compile until someone supplied its cell.** Not a
convention, not a doc — a compiler error. That is the same rung `tests/lint/rete_compile_gate.rs`
reached, and the reason that gate holds.

⛔ **But the ledger's own header names the risk, and it is severe:** *"a template that silently
mis-renders one position would report REFUSED for every row there and read exactly like a
discovery — a whole column of false findings that look like the jackpot."* Six new positions × 74
rows is **444 new cells**. Any new position needs its own known-answer calibration — both
verdicts — **before its column counts for anything.**

## What is NOT measured here

- **The accumulate reducer body** — the other half of that same `Design call 3` sentence. Could not
  be driven cleanly in this pass. Unmeasured, not cleared.
- **Thoroughness within a position.** Each cell above used ONE ill-typed shape. That establishes
  the position is *reached* by the type checker, never that it is checked *completely*.
