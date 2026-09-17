# DESIGN — cure D7: the native engine drops a derived fact

## Why

**D7 is live.** Driven and re-driven: `native=2 oracle=3`. Three facts in, three derived facts
expected; the native engine produces two while `fire-rules$oracle` on the identical staged session
produces three. **A derived fact is lost with no diagnostic.** Repro:
`wat-scripts/scratch-pad/d7-two-writers-one-alpha.wat`.

## The mechanism, established

Two writers of `wm.alpha[aid]` in one seed pass:

| writer | site | operation |
|---|---|---|
| 1 | `fire/delta.rs:100`, inside `alpha_activate_fact` | `entry(aid).or_default()` → **push** |
| 2 | `fire/pass/alpha.rs:130` | `wm.alpha.insert(aid, els)` — **whole-entry replace** |

**Parametric records erase their type argument into one runtime class.**
`(:d7::Box :- [T] [k <- i64  v <- :T])` gives one class whose instances differ in *packability*:
`pack_i64_row` (`session.rs:309`) tests **runtime values**, so `Box{v:100}` joins the occupancy batch
and `Box{v:"…"}` falls to `alpha_activate_fact`. `arm.rs:334` files each node under exactly one
`pat.type_head`, so **both writers reach the same `aid`**, and writer 2's `insert` discards writer 1's
push. `d_alpha[aid]` still holds the pushed slot indices, which then index **different elements** —
the delta is aliased, not merely short.

⛔ **The code contradicts its own doc.** `session.rs:256` says *"`None` = not all **declared** fields
i64"*. The implementation tests runtime values. **That gap is the defect.**

## ★ THE INVARIANT, not a mechanism

> **No `aid` may receive both a push and a replace in one seed pass.**

This is deliberately not a prescribed cure. At least three shapes satisfy it and they trade
differently:

1. **Class-uniform batching** — a class batches only if *every* fact of it packed; otherwise all of
   them take `alpha_activate_fact`. Needs no schema access, and makes the collision unrepresentable.
   Costs a restructure: the decision cannot be made until every fact has been seen.
2. **Declared-schema packability** — decide from the record's declared field types, as
   `session.rs:256` already says. Closes the seam at its root and makes code match doc. ⚠ **The fire
   path holds no `TypeEnv`** (`FireSession` has none; `alpha_seed` takes only `sym`), so this needs
   new state threaded — measure that before choosing it.
3. **A non-replacing writer 2** — merge rather than overwrite. ⚠ **Riskiest**: `d_alpha` holds
   *indices* into the vector, so any change to element ordering re-points them. This is the shape
   that produced the aliasing already observed.

**Pick one, argue it against the other two, and state the cost.** A `★` that named a mechanism would
pre-commit the rider to a cure whose trade-offs are not yet measured — the D5 lesson.

## The gate is half the deliverable

This defect is a **native/oracle divergence** and the tree already has an oracle. Nothing on the
floor drives a parametric-record workload through both engines, which is why a fact-dropping bug
survived. **The cure ships with a differential gate over a parametric record**, or the next erasure
seam goes the same way.

⛔ **Do NOT gate this with the `leaf_occ` differential.** It is structurally blind here (**C16**):
`predicted` is built with the same predicate that decides batch membership, so it re-derives writer
2's output and compares it against writer 2's output. It read `extra=[]` while the fact was being
dropped.

## Files

`src/rete/kernel/fire/pass/alpha.rs`, `src/rete/kernel/fire/delta.rs`, possibly
`src/rete/kernel/session.rs`, and a differential gate with adjacent fixtures.

## Out of scope = REJECTED

- **C16** (the blind differential) and **C17** (`insert.rs:221`'s `if let`). Both were found driving
  D7; both are their own rows. C17 is the *second, unconstructed* route to this same collision — a
  cure for D7 does not close it.
- Reaping either writer. Both are live paths with real workloads.
