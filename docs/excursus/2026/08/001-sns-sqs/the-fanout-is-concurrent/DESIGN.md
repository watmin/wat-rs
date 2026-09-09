# DESIGN — the fan-out is concurrent

Drawn 2026-09-02. **Not struck.**

## Why

Drain is 41.5 s of a 55.7 s wall — **5.18 ms per delivery**, of which ~0.47 ms is the accounted
residual rebuild. The other ~4.7 ms is the chain: topic → adapter → queue → store and back, three
nested round trips whose isolated cost is ~550 µs. An 8× gap, and the shape explains it.

`-deliver` fans out with a `foldl` that calls `:demo::Sub/deliver` on each subscriber **and awaits
each reply before sending the next**. So one message costs **sum of four chains**, not max. The box
has **12 cores and only ~4 are busy** during a delivery — one topic, one adapter, one queue, one
store. The other three quarters of the machine is idle while the topic waits in turn.

## What it delivers

`sum(4)` becomes `max(4)`. The four subscribers are genuinely independent — four adapters, four
queues, four stores, all separate processes — so the parallelism is real, not notional.

## The shape

The generated client `Sub/deliver` is a **send immediately followed by a recv**
(`wat/service.wat`'s `send-recv-form`, built at `:2197-2302`). Split them:

```
send p0 ; send p1 ; send p2 ; send p3      all four chains now running
recv p0 ; recv p1 ; recv p2 ; recv p3      collect
```

No `select` is needed. Collecting in fixed order costs nothing — the total is still bounded by the
slowest, and correctness only needs all four faced.

The raw forms are `(:wat::kernel::send p (:demo::Sub::Op::Deliver req))` and
`(:wat::kernel::recv p)` → `(RecvOutcome :- [:demo::Sub::Reply])`. The `Op`/`Reply` variant naming
is the generator's own: `service-op-str` + `::{variant-pascal}` (`service.wat:1534`), which is why
the queue's helper spells its internal op `(:queue::queue::Op::-Tick)`.

## The one contract decision — the client-side cap guard is bypassed, knowingly

The generated client checks `:max-request-bytes` **before** sending, gated on `peer-wire?`
(`service.wat:2298`). A raw `kernel::send` skips that check. The **server-side** guard is
unconditional (`service.wat:1946`) and still fires, so an oversized request is still refused — it is
refused *by the server with `RequestTooLarge`* instead of *by the client without a round trip*.

That is a real behavioural difference and it is accepted here, because: the circuit's payloads are
a few bytes against a 512 KiB cap, and the guard that protects the *service* is the one that still
runs. **If a reader ever needs the client-side check back, the honest fix is a generated send-only
client method, not hand-rolling the cap check at this call site.** Named so it is not re-derived.

## No exemplar exists — say so rather than pretend

There is raw `:wat::kernel::send` in `wat-scripts/probes/arc-170/`, but every one of those is to a
**raw spawned peer**, not to a `defservice` client. **This is the first user-level raw send to a
service peer in the tree.** The generator at `service.wat:2197-2302` is the specification to mirror;
there is no worked call site to copy, and the brief says so instead of inventing one.

## Out of scope = REJECTED

- **Batching.** The bigger structural win (8,000 chain traversals → ~800, and `Store::PutRequest`
  already takes a vector so store writes collapse too) — but it changes **two surfaces**,
  `Sub::DeliverRequest` and `Queue::SendRequest`, and every one of those is a place a message can be
  lost against an invariant that has held all day. Size it after this lands, when the chain is no
  longer sequential.
- **`select`-based completion-order collection.** Fixed-order collection is already `max(4)`.
- **The cursor** (~0.47 ms/delivery, ~9% at N=2000). Its ruling has an expiry, not a permanent no —
  it is ~27% at N=8000.
- **`wat/`, `src/`, `sqs.wat`, the worker, the adapter.** None of them change.
