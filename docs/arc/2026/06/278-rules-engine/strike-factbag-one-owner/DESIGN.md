# DESIGN — FactBag: one owner for the fact base

**Strike 1 of 2. NO BEHAVIOUR CHANGE.** Strike 2 (`remove-one` + the redrawn proof axis + the TMS
fuzzer's model) rides on this and is drawn separately, so a floor red here is a pure migration bug.

## Why

`Session.facts` is a bare `:wat::core::PersistentVector` (`wat/rete.wat:192`) — the type does not
even say it holds Records. A list is neither a set nor a bag, so **every verb that touches it
invented its own multiplicity semantics**, and two of them contradict:

| site | reading |
|---|---|
| `insert$oracle` / `insert-all` (`insert.wat:22,68`) | conj → multiset add |
| `retract` (`insert.wat:100`) | filter ≠ → **set remove (removes N)** |
| `merge-facts` (`fire.wat:216`) | contains?-then-conj → set add, *for termination* |
| `retain-supported` (`fire.wat:237`) | plain filter → *"sub-MULTISET, order and multiplicity preserved"*, **MUST NOT DEDUP** |
| `acc::count` | multiset — **driven: `[1]` vs `[2]`** |
| `fire.wat:316-322` | the support-fixpoint's convergence test **depends on sub-multiset-ness** |
| `merge_facts` + `facts_membership` (`fire/rules.rs:575-577`) | the same rules again, **in Rust** |

And the field has **five doors** on the native side where it should have two:
readers `session_facts` (`session.rs:1447`), `require_session_facts` (`insert.rs:16`), raw
`session_named_field(.., "facts")` (`rules.rs:573`); writers `session_with_facts` (`session.rs:1509`),
raw `session_with_fields(.., [("facts", ..)])` (`rules.rs:185,277,423`).

This is **D3's shape a third time**, on the datum the whole engine rests on. `fire.wat:234`'s ⛔
comment is the convention rung and it has already failed — `retract` broke it from another file.

## What ships

**`wat/rete/factbag.wat`** — the one owner.

```
(:wat::core::defrecord :wat::rete::FactBag
  [items <- (:wat::core::PersistentVector :- [:wat::core::Record])])
```

Doors, all under the `:wat::rete::factbag::` prefix — **the whitelist the seal will name**:

| door | meaning |
|---|---|
| `empty` | `-> FactBag` |
| `add` | multiplicity **+1** — what `insert` means |
| `add-if-absent` | set-add — what `merge-facts` means, **named** instead of emergent from a `contains?` idiom |
| `remove-every-equal` | today's `retract`, **named honestly**. Strike 2 deletes it. |
| `retain` | predicate filter — sub-multiset **by construction**, what `retain-supported` needs |
| `count-of` / `size` | the multiplicity `acc::count` already observes |
| `items` | the read view — **the ONE place that unwraps** |

`Session.facts <- :wat::rete::FactBag`.

## THE ONE CONTRACT DECISION

**The Rust doors absorb the representation change.** `session_facts()` returns the **inner
PersistentVector** (unwrapping `FactBag`); `session_with_facts()` takes a **PersistentVector** and
wraps it. Every native caller therefore compiles unchanged, and the wrap/unwrap exists in exactly
two function bodies. The other three native doors are deleted and routed through these two.

## Enforcement — rung 2, and we say so

A corpus gate (the `no_raw_network_keys_in_oracle.rs` shape, **empty exemption list**):

- `:wat::rete::FactBag/items` and `:wat::rete::Session/facts` have **no form** in `wat/` outside
  `wat/rete/factbag.wat`.
- the string literal `"facts"` has **no form** in `src/rete/` outside `session.rs`'s two doors.

⛔ **This is a build gate, NOT the type system, and the strike may not claim otherwise.** Rung 3 —
`{:field-metadata {items {:restricted-to [:wat::rete::factbag::]}}}` on the record — is **blocked**:
a record's `:restricted-to` is parsed, stored and never enforced, and `defrecord` cannot even
express the map. Measured and filed at
`docs/arc/2026/04/109-kill-std/NOTE-a-records-restricted-to-is-stored-and-never-enforced.md`.
**Not this session's work.** When it closes, the record gains one line and **this gate is deleted** —
that deletion is the proof rung 3 arrived. Say this in the gate's header.

## Out of scope = REJECTED here

- **Any behaviour change.** `remove-every-equal` keeps today's semantics exactly. Strike 2 owns the cure.
- Closing GAP-5. Not rete.
- Touching `merge_facts`/`facts_membership`'s *algorithms* — they move behind the doors, unchanged.

## Method

`.wat` migration is a **wat-fix codemod** (`wat-scripts/fixes/`), never hand-edits, never sed:
`(:wat::rete::Session/facts X)` → `(:wat::rete::factbag::items (:wat::rete::Session/facts X))`
across the 10 read sites, and `:facts <expr>` → `:facts (:wat::rete::FactBag <expr>)` across the
11 reconstruction sites. Both are uniform wraps — codemod shape. Dry-run on a `/tmp` copy and
`diff` first. **Then** hand-refine the handful of sites that should call a *semantic* door
(`add-if-absent`, `retain`) rather than wrapping raw — that second pass is small and deliberate.

## STOP triggers

1. **STOP-1** — if any native caller of `session_facts`/`session_with_facts` needs a change beyond
   the two door bodies, the contract decision above is wrong. Report it; do not widen the strike.
2. **STOP-2** — if the codemod's dry-run diff touches a byte outside the intended wrap, stop and
   report the shape it hit.
3. **STOP-3** — if any observable behaviour moves (a differential, the grid, a fuzzer), stop. This
   strike is behaviour-neutral by construction; a moved value means a door is not equivalent.
