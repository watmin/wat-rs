# DESIGN — `seen-ids` stops cloning

Small stone. Removes the quadratic term the backlog instrument introduced, and becomes the **first
production caller** of `:wat::set::`.

## The defect

`every tier reports its backlog` (`96a840db0`) added `seen-ids` to the queue's `:ephemeral`, to tell a
first delivery from a redelivery:

```
sqs.wat        seen-ids <- (:wat::core::HashSet :- [:wat::core::String])
sqs.wat:223    (:wat::hashset::conj seen id)          — grown unbounded, never cleared
eval.rs:333    let mut out: HashSet<Value> = (**s).clone();   — the WHOLE SET, per insert
```

8000 messages through the inbox is 8000 inserts into a set growing to 8000 — **O(N²)**.

**Measured honestly: no regression at n=2000** — 41 s vs 41 s, roughly 1 s inside a fill phase that
varies by 1.8 s run to run. My earlier "32 million clones" framing overclaimed a cost the numbers do not
show.

★ What survives is narrower and still decisive: **the instrument's own term is quadratic** (~1 s at
n=2000, ~4 s at 4000, ~16 s at 8000), and an instrument with a quadratic term cannot answer whether the
system has one. It is **blocking for measurement above n=2000**, which is the next thing we need.

## ⛔ Two fixes I proposed and then disproved by reading

**"Count the visibility transition instead of the membership."** I said this twice with conviction. The
disk says no: `:queue::Envelope` is `[id, body]` and the stored row is
`(:wat::query::StoredRow :pk :sk :data :index-keys)` — **no receive-count field anywhere**, and `data`
is a bare body string. Carrying a count means changing `StoredRow`, whose shape **every `:wat::query::Store`
user shares**, or wrapping every message body. Neither is a small self-contained fix.

**"Bound the set to a recent window."** That introduces a window size — another constant nobody chose,
in an arc that has already paid for two of those.

## The fix

Swap the type. `:wat::set::` shipped in `2fdcfefaa` for exactly this migration:

```
(:wat::core::HashSet :- [:wat::core::String])   →  (:wat::core::PersistentSet :- [:wat::core::String])
(:wat::hashset::conj seen id)                   →  (:wat::set::conj seen id)
```

`HashTrieSetSync::insert` is `O(log n)` and shares structure. The quadratic term becomes `O(N log N)`.

Memory stays `O(N)` — 8000 retained ids — and that is **honest and acceptable**: the store already holds
O(N) rows, so the set is not the thing that bounds the system.

★★ Of `seen-ids`' 32 mentions in `sqs.wat`, **28 are `:seen-ids (State/seen-ids s)` threading through
State constructors and do not change.** Three real edits: the declaration, the initial value, the `conj`.

## Why not simply delete the counter

`redeliveries` read **0 at the inbox** and **20 at one subscriber tier** on both graded runs. It is a
real diagnostic — *workers are not acking inside the visibility window* — and it is the only signal that
distinguishes a slow consumer from a slow queue. Keep it; make it cheap.

## ★ This is `:wat::set::`'s first production use

The type shipped with two unit tests and a corpus probe. **If it has a defect, a real caller at n=2000
finds it** — and that is a second reason to do this rather than deleting the counter.

## The migration worklist (census, this session)

Cloning-verb calls across the corpus, with a proximity heuristic for fold-adjacency. **A candidate list
needing per-site judgment, not a verdict:**

| file | calls | near a fold |
|---|---|---|
| `wat-scripts/fanout/circuit.wat` | 6 | **6** — includes the `:2075` distinct-fold already named in the tracker |
| `wat/telemetry/span.wat` | 8 | 2 |
| `wat/deporder.wat` | 2 | 2 |
| `wat/service.wat` | 12 | 1 |
| `wat/core.wat` | 2 | 1 |
| `wat-scripts/queue/sqs.wat` | 1 | 1 — **this stone** |
| scratch-pad probes | 1 each | gates, not hot paths |

⚠ `wat/` is the **stdlib and the builder's call** — listed, not scheduled. `circuit.wat`'s six are the
largest cluster and the obvious next candidate after this.

## OUT OF SCOPE — REJECTED

- **A lint over the cloning family.** Measured and declined: `no_rpds_rebuild_loop` scans `.rs` only,
  extending it would flag the intrinsics that clone *by design*, and the family is scheduled for
  replacement. The census above is the durable artifact instead.
- **`circuit.wat`'s six sites** and everything under `wat/`. Named above; not this stone.
- **Changing `StoredRow`, `Envelope`, or the body wire shape.** See the disproved fixes.

## Files

`wat-scripts/queue/sqs.wat` only.
