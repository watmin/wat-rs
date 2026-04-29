# Arc 095 — `Service<E,G>` paired channels — DESIGN

**Status:** in design 2026-04-29.

The substrate's `:wat::std::telemetry::Service<E,G>` carries a request
payload shaped `(Vec<E>, AckTx)` — the CLIENT packages its own reply
address into every request. The CLIENT bundles three channel ends
(`req-tx, ack-tx, ack-rx`) even though it only ever USES two
(`req-tx` to write, `ack-rx` to read); `ack-tx` is baggage that
travels with each request to the server, who pulls it out and
sends the ack back through it.

This arc retires the embedded-ack-tx pattern in favor of the same
**pair-by-index** model arc 089 slice 5 already shipped for
Console. After the refactor:

- **Client** uses exactly two ends: `(req-tx, ack-rx)` — block-write
  request, block-read ack.
- **Server** uses exactly two ends: `(req-rx, ack-tx)` — block-read
  request, block-write ack.
- The ack channel is always unit (`()`) — pure no-op signal.
- The request channel is always `Vec<E>` — could be a Vec of 1, but
  the wire shape is uniform.

The two ends each side holds are **opposite** — the most logical
separation. Removes the "client passes the client into the server,
and then the server uses the client's connection" weirdness the user
flagged mid-arc-091-slice-4.

## What we know

### Why the embedded-ack-tx pattern existed

Service<E,G> is **multi-client**. ONE worker drains N producers'
request channels via `crossbeam::select`. To ack the right
producer, the server has to know which client a given request came
from. The original design solved this by having each client supply
its own ack-tx in the request payload — stateless server, the
client carries its own reply address.

### Why pair-by-index is the better answer

The HandlePool that already hands out `req-tx` to each client (one
per connection, allocated at spawn time) is the natural place to
also pair an ack channel. Each "connection" is a (req, ack) pair:

- Client gets `Handle = (ReqTx<E>, AckRx)` from the pool.
- Server's loop holds `Vec<(ReqRx<E>, AckTx)>` paired by index — same
  index in both vectors.
- `select` over the rx ends; when index `i` fires, ack via the
  paired `ack-tx[i]`.

Already shipped for Console (arc 089 slice 5). Service<E,G> stayed
on the old pattern only because Console's migration was the worked
example for naming the pattern; Service<E,G> was the expected
follow-up but never landed.

### What the wire shape becomes

```scheme
;; Before
(:wat::core::typealias :wat::std::telemetry::Service::Request<E>
  :(Vec<E>,wat::std::telemetry::Service::AckTx))   ; (entries, ack-tx)

;; After
(:wat::core::typealias :wat::std::telemetry::Service::Request<E>
  :Vec<E>)                                          ; just entries
```

`AckTx` and `AckRx` typealiases remain (they're still real channel
ends), but they live PAIRED, not embedded.

### What `batch-log` signature becomes

```scheme
;; Before
(Service/batch-log<E>
  (req-tx :ReqTx<E>) (ack-tx :AckTx) (ack-rx :AckRx)
  (entries :Vec<E>) -> :())

;; After
(Service/batch-log<E>
  (req-tx :ReqTx<E>) (ack-rx :AckRx)
  (entries :Vec<E>) -> :())
```

Three params drop to two for the channel handles. Total: 3 args
including entries. Mirrors the user's clean intuition: "client uses
block-write followed by block-read".

### What the Pool shape becomes

```scheme
;; Before
(:wat::core::typealias :wat::std::telemetry::Service::ReqTxPool<E>
  :wat::kernel::HandlePool<wat::std::telemetry::Service::ReqTx<E>>)

;; After — pool hands out (ReqTx, AckRx) PAIRS
(:wat::core::typealias :wat::std::telemetry::Service::Handle<E>
  :(wat::std::telemetry::Service::ReqTx<E>,wat::std::telemetry::Service::AckRx))

(:wat::core::typealias :wat::std::telemetry::Service::HandlePool<E>
  :wat::kernel::HandlePool<wat::std::telemetry::Service::Handle<E>>)
```

The Handle name parallels Console's `Console::Handle` from arc 089
slice 5 — both crates settle on the same vocabulary for "the thing
a client pops from a pool to talk to a server."

### What the server's internal state becomes

```scheme
;; Before — server holds Vec<ReqRx>, ack-tx comes from each request
((rxs :Vec<ReqRx<E>>) ...)

;; After — server holds parallel vectors of paired endpoints
((rxs    :Vec<ReqRx<E>>) ...)
((acks   :Vec<AckTx>)    ...)   ; same length; paired by index
```

`select` returns the firing index; `acks[idx]` is the response
endpoint. Same pattern Console's `Vec<DriverPair>` uses (arc 089
slice 5).

## What we don't know

- **Performance impact.** None expected — bounded(1) channels are
  cheap; the protocol shape doesn't change throughput. The lab
  pulse benchmark stays at 45ms or this arc rolls back.
- **Whether `Console::Handle` and `Service::Handle` should be
  unified.** They have the same shape — `(ReqTx<E>, AckRx)`. A
  single substrate `:wat::kernel::ConnectionHandle<E>` typealias
  could subsume both. Not in scope for this arc; flag for a future
  housekeeping pass.
- **Migration path for external consumers.** Outside the workspace
  (if any), callers see a breaking signature change. There are
  none today, so this is theoretical.

## Slices

```
Slice 1 — Service.wat protocol pivot
  - Drop AckTx from Request<E>; payload becomes Vec<E>.
  - HandlePool hands out (ReqTx<E>, AckRx) pairs.
  - Service/run holds Vec<ReqRx<E>> + Vec<AckTx> by index.
  - drain-rest / ack-all updated to use the paired ack-tx.
  - Service/batch-log signature: (req-tx, ack-rx, entries) -> ().
  - Spawn return: HandlePool<Handle<E>>.
  - Substrate tests in wat-tests/std/telemetry/Service.wat updated.

Slice 2 — wat-sqlite consumer migration
  - Sqlite.wat's Sqlite/spawn and Sqlite/auto-spawn pass through
    the new pool shape.
  - auto-dispatch-batch unchanged (server-side; still per-batch
    BEGIN/COMMIT around per-entry dispatch).
  - wat-tests/std/telemetry/{Sqlite, auto-spawn, edn-newtypes}
    updated.

Slice 3 — Console.wat alignment (housekeeping)
  - Console::Handle and Console::DriverPair naming aligned with
    Service::Handle. Optional — depends on whether the unified
    ConnectionHandle from "what we don't know" lands.
  - Defer to follow-up if it bloats this arc.

Slice 4 — docs + INSCRIPTION
  - ZERO-MUTEX.md § mini-TCP: note Service<E,G> now follows the
    same pair-by-index pattern Console established in arc 089
    slice 5; the embedded-reply-tx case retires from the substrate.
  - INSCRIPTION captures the protocol shape change.
```

## What's NOT in this arc

- **Lab consumer migration.** External repo
  (`holon-lab-trading`); the lab tracks this arc as a known
  upstream change and migrates pulse.wat / smoke.wat / bare-walk.wat
  in its own next session.
- **Unified `:wat::kernel::ConnectionHandle<E>` typealias.** Above —
  a future housekeeping arc.
- **Backward-compat shim for the old protocol.** None ships. The
  workspace has zero external Service<E,G> consumers; cleaning up
  beats keeping a transition surface alive.

## Surfaced by

User direction 2026-04-29, mid-arc-091-slice-4:

> "hrm... why is wat::std::telemetry::Service::AckRx necessary on
> the server side... when do they use it?.. should they just block
> read on the req-tx and then block write on the ack-tx ?...
> the client block writes on req-rx and then block reads on ack-tx?....
> i'm confused...."

> "we debated the handles before and i was confused.. the server is
> using both ack handles... i'm confused...
> in my mind:
>   client uses (req-tx, ack-rx)
>   server uses (req-rx, ack-tx)
> client uses block-write followed up by a block-read
> server uses block-read followed up by a block-write
> when do they need more than 2?"

> "i think its best that we always use exactly 2?... how awful of a
> refactor is this... having the client pass they client into the
> server and then the server use's the clients connection feels
> /extremely/ messy"

> "make a new arc - fix this - i didn't realize it was done this
> way... client holds two opposite ends... server holds two opposite
> ends... that's the most logical separation... the ack channel is
> always a no-op message, the req channel always a vec of batches
> items (who could be a vec of 1)"

The "extremely messy" feeling is the load-bearing diagnosis. The
embedded-reply-tx pattern leaks the multi-client-routing concern
into the request shape; pair-by-index moves it to setup time
(HandlePool::pop) where it belongs.

Arc 091 slice 4 (wat-measure SinkHandles) is paused until this
arc lands — `SinkHandles` would otherwise inherit the three-handle
bundle, and slice 6's lab refactor would too. Better to fix the
substrate first.
