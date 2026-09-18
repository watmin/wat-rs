# DESIGN — A8: a cure maintained by convention, guarding against fact LOSS

> Drawn 2026-09-05 at HEAD `3a54d440d`. Source: vigilia 2026-09-05 A8 (`perspicere`).
> Every line verified on disk at THIS HEAD.

## The site says it itself

`src/rete/kernel/fire/pass/alpha.rs:72-74`:

> **THE CURE IS THE `bool` BELOW** — *"every fact of this class packed"*. A class batches only if it
> is uniform; a mixed class takes the activate path for ALL of its facts (the deferred loop after
> the batch), **so exactly one writer ever touches an aid.**

That bool is the cure for a prior double-write defect — the header records the original: the batch
*"replaced the whole `Arc<Vec<Element>>` and discarded"* the unpacked pushes, while `d_alpha[aid]`
kept slot indices that then *"index DIFFERENT elements."*

**And the cure is maintained by convention, across two disjoint `&mut` arms.**

## Three pieces of state, one fact

```rust
let mut class_ids: HashMap<String, (Vec<u32>, bool)> = HashMap::new();   // :85
let mut any_mixed = false;                                              // :91

if packed {
    if let Some((ids, _)) = class_ids.get_mut(class) { ids.push(i as u32); continue; }   // :139-142
} else if let Some((_, uniform)) = class_ids.get_mut(class) {
    *uniform = false;
    any_mixed = true;                                                   // :143-147
    continue;
}
…
if any_mixed { activate_deferred_mixed_classes(…) }                     // :217
```

- the packed arm pushes `ids` and **never touches `uniform`**;
- the else arm demotes `uniform` and **never touches `ids`**;
- **`any_mixed` is DERIVABLE** — `class_ids.values().any(|(_, u)| !u)` — and is stored beside the
  thing it summarises.

## Why this is worse than the D2 class it resembles

`any_mixed` gates `activate_deferred_mixed_classes` **entirely** (`:217`). A writer that sets
`uniform = false` without setting `any_mixed` does not double-count — **every fact of that class is
deferred and then never activated.** Facts silently vanish from the fire.

D2 shipped duplicates. A1 shipped missing rows. **This shape loses facts outright**, and the state
that prevents it is a `bool` assignment on the line after another `bool` assignment.

## LATENT — say it plainly

Today the two arms are correct: every `uniform = false` is followed by `any_mixed = true`. **I have
not constructed a violation and I do not believe one is reachable at this HEAD.** The defect is that
the cure is a convention, in a site whose own header explains that a convention was not enough last
time — which is the rung `session.rs:224-231` already ruled on: *"⛔ THE CURE IS STRUCTURAL, NOT
CONVENTIONAL… would have cured today's two writers and left a third free to appear."*

## The one contract decision, pinned

**One type owning the per-class plan, one door, and `has_mixed()` DERIVED — never stored.**

```rust
struct ClassPlan { … }                 // private map; no &mut escapes
impl ClassPlan {
    #[inline] fn observe(&mut self, class: &str, i: u32, packed: bool);  // push-or-demote, ONE act
    fn has_mixed(&self) -> bool;       // DERIVED from the map, not a field
}
```

Deriving `has_mixed` is the load-bearing half: a stored summary beside the thing it summarises is
the same defect one level up, and D1 (a count used as a prefix offset) is this session's example of
what that costs.

## ⛔ The hot-path constraint, and it is in the site's own words

`alpha.rs:131-137` argues the packed-arm-first ordering deliberately:

> ⚠ **THE PACKED ARM IS FIRST ON PURPOSE, and the duplicated `get_mut` is the price.** Written as one
> lookup with the `packed` test inside, this taxes the batch fast path — the very path the cure
> exists to preserve.

So `observe` must be `#[inline]` and branch on `packed` **before** the lookup, reproducing today's
instruction sequence for a packing fact. **A cure that buys correctness with the batch path's cost
is a failed strike**, and a moved `*_cost` number is the signal.

## Scope

**IN:** the type, the door, the derived predicate, and the proof. Floor GREEN.

**OUT, affirmatively cut:** `alpha.rs:86`'s `Vec::with_capacity(input_facts.len())` per class
(`temperare` L2-e: K classes × N facts reserved where the union is at most N) — a real row, a
different one. D2p, F2, A3, A4.
