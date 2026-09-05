# DESIGN-STONE — the RHS compiles once; `build_insert_fact` stops re-deriving a static program

> **Origin (2026-08-01).** `DESIGN-STONE-compiled-conditions.md` cut its own scope with a hedge it
> was careful about: *"The same recompute-a-static-program shape **plausibly** appears there — **I
> have not read those hot paths, so this stone does not claim it.** Tracked as
> `DESIGN-STONE-compiled-rhs`, to be drawn only after the same grounding this one got."*
>
> ⚠ **That hedge was then lost twice, with no new evidence at either step.** `SEAM-2026-08-01` (a dated seam since pruned; the breadcrumb is now the single `SEAM.md`)
> restated it as *"shape fully known"*; this apparatus restated that as *"designed, never built."*
> A hedge, restated, loses its hedge — the third instance of that class in one session (R60's
> rationale-for-inaction became a verdict; the seam's *not deletable* became *not optimizable*).
> The disk never changed. This document exists because the hot path has now actually been read.

## What the hot path does — read, not assumed

`matcher.rs:517`, inside `resolve_operand`, called once per RHS field per derived fact:

```rust
census_count("match:key-alloc");
let key = Value::String(Arc::new(name.to_string()));   // String alloc + Arc alloc
bindings.get(&key).cloned()                            // hash the String, walk the trie
```

`?k` is fixed at rule-compile time. It is rebuilt from scratch on every field of every derived
fact. Above it, `build_insert_fact` re-does more static work per fact: re-validates the
`(:wat::rete::insert (:Type …))` form shape, re-detects kwargs-vs-positional, and re-allocates the
class `String` from the type keyword.

## Measured — counts, which are exact; NOT times, which here are not

`fanout_rhs_key_alloc_census`, the 40,000-derivation fanout cell:

```
match:key-alloc  (ALL of it the RHS — alpha is compiled)   120,000    = 3.00 per derived fact
match:calls      (interpreter entries — expect 0)                0    <- attribution closed
```

Each key-alloc is a `String` **and** an `Arc` — **240,000 heap allocations rebuilding three
constants** in one fire.

**Deliberately unsized in milliseconds.** The four `prod:*` marks fire once per derived fact —
160,000 mark pairs on this cell — so a large fraction of the 17.45 ms they report is the instrument
itself. Extrapolating a saving from them is exactly the error that made compiled-conditions'
original estimate wrong by 10×. The size comes from an A/B where both arms carry the same
instrument, run at the strike.

## ★ THE ONE CONTRACT DECISION

**Compile per RULE at setup, exactly where `compiled_conds` is already built; the produced `Value`
is byte-identical.**

```rust
struct CompiledRhs { class: String, ops: Vec<RhsOp> }   // one per :then insert-form
enum   RhsOp       { Bind(Value),  Lit(Value) }         // Bind holds the PRE-BUILT key Value
```

Per derived fact the whole function becomes: walk `ops`, `bindings.get(k).cloned()` or `v.clone()`,
build the record. Nothing else.

This eliminates, per fact: the form validation, the kwargs detection, the class allocation, and
both key allocations per field. It keeps, because they are irreducible: N trie lookups, N `Arc`
bumps, the fields `Vec`, and the `AggregateValue`.

**It does NOT eliminate the class `String` per fact** — `AggregateValue::record` takes an owned
`String`, so one allocation stays. Interning that is a different stone and is not smuggled in here.

## Blast radius

`src/rete/matcher.rs` (the compiler + the executor beside `build_insert_fact`) and
`src/rete/kernel/` (build the compiled forms alongside `compiled_conds` at setup — `kernel/arm.rs`; call the
executor from the production pass — `kernel/fire/pass/production.rs`, where `rule_rhs_cache` is looked up today; that
name is gone, the arm field is `compiled_rhs`). **Nothing under
`wat/`** — the oracle stays naive by ruling.

`build_insert_fact` is **NOT deleted**: it stays as the reference implementation and the other half
of the differential, exactly as `alpha_match_inner` did. Whether it survives having no production
caller is a separate ruling (`feedback_no_consumers_does_not_mean_dead`).

## The gate

1. **The differential, on the VALUE — not on a count.** Over the grid's axes, `compiled_rhs(form,
   bindings)` must equal `build_insert_fact(form, bindings)`: same class, same field values, same
   order. A both-produced-a-record comparison would pass while deriving wrong facts.
2. **`match:key-alloc` goes to ZERO** on the fanout cell, asserted via the existing counter — the
   mechanism proven by count, not inferred from a timing change. `prod:class-alloc` stays at 40,000
   and that is expected, not a miss.
3. **A/B the fire in one batch** (stash between arms), reporting `production` and `THE FIRE` with
   ranges. Recorded, and honest about whether they overlap.
4. `:accuracy :match` on every grid axis; release floor; clippy 0.

## Out of scope = REJECTED (affirmative cuts)

- **Interning the class `String`.** Real, adjacent, and its own stone — see
  `109-kill-std/NOTE-keyword-storage-must-intern.md`, whose "NOT the rete lever" caveat is right
  for *lookup* and wrong for *construction*. Smuggling it in would destroy the attribution.
- **Changing `Token.bindings`.** Settled by measurement today (`NOTE-token-bindings-stays-a-trie.md`);
  the N trie lookups stay.
- **Deleting `build_insert_fact`.** Kept as the differential's other half.
- **`eval_test_core` / the accumulate fold.** The same shape may well be there. It has not been
  read, and this stone does not claim it — which is the hedge that started this document, kept
  intact this time.
