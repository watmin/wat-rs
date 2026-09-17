# DESIGN — two writers of `wm.alpha[aid]`, and one armed test between them

## Why

Work-list **D7**. Two code paths write the same map entry in one seed pass, and one of them
**replaces** what the other **appends**:

| writer | site | operation |
|---|---|---|
| 1 | `fire/delta.rs:100` — inside `alpha_activate_fact` | `Arc::make_mut(cx.wm.alpha.entry(aid).or_default())` then **push** |
| 2 | `fire/pass/alpha.rs:130` — the occupancy batch | `wm.alpha.insert(aid, Arc::clone(&els))` — **replace** |

⚠ **The row puts both in `alpha.rs:85-98,129-132`. They are in two different files**, and only
writer 2 is in the one the row names.

## How they are kept apart, exactly

`pass/alpha.rs`'s seed loop sends each fact down **one** path:

- not an `Aggregate`, or `Nature::Struct` → `_ =>` arm → **writer 1**
- `packed` **and** its class is in `leaf_aids` → deferred into `class_ids`, later → **writer 2**
- otherwise → **writer 1**

So a given *fact* takes one path. The invariant that matters is stronger: **no `aid` may receive both
a push and a replace in the same pass** — because writer 2 replaces the whole `Arc<Vec<Element>>`,
discarding anything writer 1 appended.

That holds while **packability is constant across the facts of a class**. It is decided by
`pack_i64_row` (`session.rs:309`), which returns `None` unless **every field is `Value::i64`** — a
test of **runtime values**, though its own doc at `:256` describes it as a property of the
***declared*** fields: *"`None` = not all declared fields i64, or wider than `I64_ROW_CAP`"*.

**Declared-vs-runtime is the seam.** If any class can hold an i64 in one instance and a non-i64 in
another for the same declared field, packability varies within the class, both writers fire for the
same `aid`, and the batch silently discards the pushed elements.

## What already checks this, and how thin it is

`fire/delta.rs:118-170` computes `predicted` occupancy against `actual` and reports `extra`/`missing`
— a real differential for exactly this invariant. But it is gated on `leaf_occ_armed()`,
`record_leaf_occ_diff` is `#[cfg(test)]`, and the arming helper `with_leaf_occ_diff` has **one call
site in the whole tree** (`rank_and_instrument.rs:626`). **In production it never runs.**

So the invariant has one check, in one test, only when explicitly armed.

## ⛔ ACT ONE IS A DRIVE, NOT A CURE — AND IT MAY CLOSE AS A NEGATIVE

**Whether any constructible input makes one `aid` receive both writes is UNDETERMINED.** This row is
labelled a *shape* finding, and this arc has already closed one of those — **D2** — as a **bounded
negative**: the code asymmetry was real, no constructed input reached it, and *the audit was the
deliverable*. D7 may go the same way.

So the first act is to **construct the trigger, or fail honestly and say what was tried**. A cure
drawn before that is a cure for a defect nobody has shown exists.

⚠ **"I could not construct a trigger" is NOT "there is no trigger"** — D2's own ruling. If the drive
comes back empty, the finding is *latent*, the row closes as bounded, and the code **must not be
reaped** on the strength of it.

## The contract decision, pinned

**Whatever the drive returns, the invariant stops depending on one armed test.**

- If reachable: cure the double-write, and gate it.
- If not reachable: the disjointness becomes an **assertion at the write site** — writer 2 refuses to
  replace an entry writer 1 has already touched this pass — so the latent case cannot become live
  silently later. A `debug_assert!` is acceptable and its choice must be argued.

## Files

`src/rete/kernel/fire/pass/alpha.rs`, `src/rete/kernel/fire/delta.rs`, and a gate.

## Out of scope = REJECTED

- Un-arming or widening the `leaf_occ` census generally. Its cost is why it is armed; that is a
  separate question.
- C14 (`compiled:calls` is not a call count) — it is incremented in `pass/alpha.rs` two lines from
  writer 2 and will be tempting. Separate row.
