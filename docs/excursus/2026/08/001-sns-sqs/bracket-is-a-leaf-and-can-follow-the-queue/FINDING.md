# `bracket.wat` is a leaf, and it can load after the queue

**Measured 2026-09-19, HEAD `747640c12`.** A feasibility probe for the brackets-on-queue goal.
Nothing landed; the manifest edit was reverted.

## The question

The excursus' original goal: promote `queue`/`topic` into the stdlib so **brackets can be built on
queues** and inherit the service-to-service integration. Brackets consuming a queue's *types and
functions* is an order-BOUND reference, so `queue` must load before `bracket.wat`.

`bracket.wat` sits at manifest slot **23**. Queue is built over `:wat::query::Store` (≈180 real
code references — `Store::Op`, `Store::Reply`, `StoredRow`, `Key`, `IndexKey`, `IndexSchema`,
`IndexRow`, `Store/put`, `Store/scan`, `Store/delete`…), and `wat/query.wat` is at slot **49**.

So the obvious reading is: hoist the whole persistence stack — query (49–51), plus whatever it
needs — above slot 23. That is ~27 files and a cascade.

## ⭐ The obvious reading is wrong, and the move is one file

**Nothing in the stdlib depends on `wat/bracket.wat`.**

A census of `:wat::bracket::` in stdlib code (comments stripped) finds **82 refs in `bracket.wat`
itself and 2 in `wat/spawn.wat`** — and `spawn.wat:269` is a `defenum` that **DEFINES**
`:wat::bracket::PoolMsg`, with `:454` using it locally. The namespace is split across two files;
spawn does not consume bracket. That is why deporder is clean today despite 22 appearing to
reference 23.

A leaf can always move **later** — its own dependencies are all earlier by construction.

## The probe

Moved the `wat/bracket.wat` `WatSource` block from slot 23 to immediately after
`wat/query/sqlite-store.wat`, rebuilt release, and asked the authority rather than a grep:

| check | result |
|---|---|
| `(:wat::deporder::verify-stdlib)` with bracket at **51** | **`[]` — zero violations** |
| `cargo nextest -E 'test(bracket) or test(stdlib_load_order) or test(pool)'` | **49/49 passed**, including `verify_stdlib_has_no_load_order_violations` and `probe_arc170_c1_kwargs_bracket::c1_kwargs_workfn_invoked_with_dialed_peer` (the real dial path) |
| baseline, bracket at 23 | `[]` |

⚠ **This is a feasibility probe, not a landing.** No full floor was run; the manifest edit was
reverted (`git checkout src/load/stdlib.rs`). It establishes that the load-order obstacle to
brackets-on-queue **does not exist**, not that the reorder is finished work.

## What this means for the work list

1. **Promote `queue` + `topic`** — unblocked on cost. The parked work (`6136d144f`) was correct and
   is recoverable; it died on the load gate (375 s against a 600 s kill, 8% headroom). That gate now
   runs **56.65 s — 90% headroom**, after Tier A + Tier B/C took boot from 0.43 s to ~0.066 s.
   ⛔ Keep the parked ruling: do **not** promote the 13 "CLIENT API" panic-gate names — every
   failure arm is `assertion-failed!` and the two real consumers call zero of them.
2. **Move `bracket.wat` after the queue** — one manifest block, verified above.
3. **Rewrite bracket's transport onto queue** — the actual work, and the only genuinely open part.

## Method note, because it nearly went the other way

The namespace census over `wat/*.wat` is a **ceiling, not a census**: the glob does not recurse
(27 of 55 files), and an un-stripped grep counts comments and quoted templates. Both traps fired
during this probe. `wat/deporder.wat` is the authority — it classifies `defmacro` refs as
**order-free** (which is why `cache.wat` at 25 can declare a `defservice` while `service.wat` sits
at 35) and reports violations against a real baked order. Ask it; do not hand-derive from grep.
