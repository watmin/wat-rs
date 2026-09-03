# DESIGN — the wire carries a batch

Drawn 2026-09-02. **Not struck.** The last structural lever on the running system.

## Why

With backpressure restored the circuit is **661 deliveries/s, deterministic**, and the phase that
holds all the real work is `publish` (12.0 s). Per lane that is **~6 ms per delivery against a
measured chain of ~5 ms** — the system is within ~20% of its per-message chain latency, doing **one
message at a time per lane**.

Nothing left touches that except putting more than one message in flight per chain traversal.
`setup`/`stop` are process lifecycle and explicitly **not** this arc's concern.

## What it delivers

One chain traversal carries K messages instead of one. Per ten messages:

| | today | batched |
|---|---|---|
| `Sub/deliver` hops | 40 (10 × 4 subs) | **4** |
| `Queue/send` hops | 40 | **4** |
| store puts | 40 single-row | **4 × ten-row** |
| chain traversals | 10 | **1** |

★ **A second win that needs no separate stone:** the queue's `send` arm serves its parked waiters
*once per call*. Batched, that is once per ten — so the wasted `take` scans measured earlier drop by
the same factor for free.

## The estimate, decomposed — 3–5×, not 10×

The chain amortises by K. The **per-message CPU does not**: each message still needs its own
`uuid::v4`, two `edn::write`s, and its own `StoredRow` built. Only round trips collapse.

Writing it down before the strike so it is not called a disappointment after: **3–5×**, i.e.
~2000–2600 deliveries/s. This is the first estimate this arc that is decomposed from measured
pieces rather than inferred from a neighbour.

## The one contract decision: no linger

`-deliver` sends **whatever the outbox holds, up to K**. It never waits to fill a batch.

That is what keeps this from being the buffering we just removed. A linger timer would trade the
~200 ms end-to-end we just won back for throughput — reintroducing, in a different place, exactly
the reservoir this arc spent the day discovering. **There is no timer in this design.**

## The surfaces that move, and the cost

```
Sub::DeliverRequest    msg   <- String   ->  msgs   <- (Vector :- [String])
Queue::SendRequest     body  <- String   ->  bodies <- (Vector :- [String])
```

`Store::PutRequest` already takes a vector, so the store needs nothing. Responses carry a count
rather than the echoed body.

**Every one of those is a new place a message can go missing**, against an invariant that has held
through nine stones today. `total=8000; distinct=8000; dup=0` is the whole guard, and it is why this
was cut twice while cheaper wins existed. They are now spent.

## `cap` and `K` interact — sweep, do not taste

Batching wants a deeper buffer; backpressure wants a shallow one. At `cap 16` and `K 10` the outbox
hovers near the cap, so batches are near-full — but that is a coincidence of two numbers nobody
chose together. **Sweep `cap ∈ {16, 64, 256}` at `K = 10` and report throughput AND e2e max for
each.** The knee is the finding; a single pair of constants is not.

## Out of scope = REJECTED

- **Process spawn cost.** `setup`+`stop` is 49% of wall and is boot time, tracked separately by the
  builder. **Wall time is not this stone's metric — deliveries/s is.**
- **A linger/window timer.** Above. Not a tuning choice; a design exclusion.
- **The outbox rebuild** (~0.47 ms/delivery) and the **`Full`-retry spin** (`nap-ms 1` in `accept!`,
  which a shallow cap makes hot). Both real, both smaller, both after this.
- **`wat/`, `src/`.** Neither changes.
