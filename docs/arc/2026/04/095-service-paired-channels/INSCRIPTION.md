# Arc 095 — `Service<E,G>` paired channels — INSCRIPTION

**Status:** shipped 2026-04-29. One slice, two-handle protocol everywhere.

The pre-arc protocol packaged the client's reply address (`AckTx`)
into every `Request<E>` payload — the worker pulled it out and
acked back through it. Client bundles `(req-tx, ack-tx, ack-rx)`
even though it only ever USES `req-tx` and `ack-rx`; ack-tx was
baggage that traveled with each request. The user named it
"extremely messy" and asked for the natural shape: each side
holds two opposite ends.

After this arc:
- **Client** holds `Handle = (ReqTx<E>, AckRx)` — block-write request,
  block-read ack.
- **Server** holds `DriverPair = (ReqRx<E>, AckTx)` paired by index —
  block-read request, block-write ack via the matching ack-tx.
- **Request<E>** is now just `Vec<E>` — the wire payload carries
  no reply-address.

The server's per-request routing (multi-client) lives in the
HandlePool's setup-time pairing, not in the wire payload. Same
**pair-by-index** pattern Console established in arc 089 slice 5.
Every Service<E,G> consumer now sees the cleaner shape.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped

### `wat/std/telemetry/Service.wat` — protocol pivot

- New typealiases: `Handle<E>`, `DriverPair<E>`, `HandlePool<E>` (replaces
  `ReqTxPool`), `IndexedDriverPair<E>`, `Connection<E>`.
- `Request<E>` retired the embedded ack-tx — payload is bare `Vec<E>`.
- `Spawn<E>` returns `(HandlePool<E>, ProgramHandle<()>)` — the pool
  hands out Handle pairs.
- Worker `Service/run` and `Service/loop` carry `Vec<DriverPair<E>>`
  internally; `pair-rxs` extracts the rx halves for `select`.
- `drain-rest` retired in favor of `drain-pairs` — single foldl over
  ALL pairs (including first-idx); the first-idx pair gets
  `first-entries` from select, the rest try-recv. Eliminates the
  out-of-band lookup that prompted the search for `:wat::std::list::nth`
  (which doesn't exist; `(get vec idx) -> Option<T>` is the path).
- `batch-log` signature: `(req-tx, ack-rx, entries) -> ()` — three
  args total, two channel ends. The user's mental model exactly.
- `spawn` builds N request channels + N ack channels; client gets
  Handles (req-tx + ack-rx); server gets DriverPairs (req-rx + ack-tx).

### `wat-tests/std/telemetry/Service.wat` — test sweep

Three deftests rewritten around the Handle / batch-log shape. Empty
prelude `()` retained per `:wat::test::deftest` arity-3 signature
(`name + prelude + body`).

### `crates/wat-sqlite/wat/std/telemetry/Sqlite.wat` — consumer migration

`Sqlite/run` takes `Vec<DriverPair<E>>` instead of `Vec<ReqRx<E>>`.
`Sqlite/spawn` builds the request + ack channel pairs, zips them
into Handles (client side) and DriverPairs (server side), pool
hands out Handles. `auto-spawn` rides the same protocol via
`Sqlite/spawn`.

### `crates/wat-sqlite/wat-tests/std/telemetry/{Sqlite,auto-spawn,edn-newtypes}.wat`

All three test files updated:
- `pool` typed as `HandlePool<E>` (was `ReqTxPool<E>`).
- `pop` returns a `Handle<E>`; tests destructure into `req-tx + ack-rx`.
- `batch-log` called with 3 args (was 4).
- No more inline `make-bounded-queue :() 1` for the per-call ack
  channel — ack channels are pre-paired by spawn.

### Substrate test for typealias-at-HashMap-constructor

Already shipped in the previous commit. Locks the rule: tuple
typealiases resolve at the HashMap constructor's first-arg site
(`:my::KV ≡ :(K,V)` → `(:wat::core::HashMap :my::KV ...)` works).

### Type-aliases that landed alongside

- `:wat::std::telemetry::Service::Handle<E>` — client-side bundle
- `:wat::std::telemetry::Service::DriverPair<E>` — server-side bundle
- `:wat::std::telemetry::Service::HandlePool<E>` — wraps
  `:wat::kernel::HandlePool<Handle<E>>`
- `:wat::std::telemetry::Service::Connection<E>` — `(ReqChannel<E>, AckChannel)`
  for the spawn step's zip
- `:wat::std::telemetry::Service::IndexedDriverPair<E>` —
  `(DriverPair<E>, i64)` for drain-pairs' foldl

The user enforced "we need type aliases" twice during the slice —
once at the Connection shape and once at the IndexedDriverPair
shape. Inline tuple types in lambda parameters degrade
readability fast; the pattern is now: alias on first repeated use.

---

## What's NOT in this arc

- **Lab consumer migration.** External repo (`holon-lab-trading`);
  the lab carries its own pulse.wat / smoke.wat / bare-walk.wat
  migrations to the new protocol in its next session.
- **wat-measure SinkHandles update.** The wat-measure crate still
  references the old SinkHandles shape (3 ends). Arc 091 slice 4
  resumes against the new protocol; SinkHandles becomes
  `(ReqTx<Event>, AckRx)` — two ends.
- **`wat-telemetry` crate consolidation.** Mid-arc the user
  surfaced the deeper architectural question: fold
  `:wat::measure::*` into `:wat::telemetry::*` and break out a
  dedicated `wat-telemetry/` crate (and a `wat-telemetry-sqlite/`
  crate, deleting `wat-measure/` and probably also the
  `wat-sqlite` → `wat-telemetry-sqlite` rename). That's a future
  arc — call it 096 — separate from this protocol-shape fix.

---

## Surfaced by

User direction 2026-04-29, mid-arc-091-slice-4:

> "we debated the handles before and i was confused.. the server is
> using both ack handles... in my mind: client uses (req-tx,
> ack-rx); server uses (req-rx, ack-tx); when do they need more
> than 2?"

> "i think its best that we always use exactly 2?... how awful of
> a refactor is this... having the client pass they client into
> the server and then the server use's the clients connection
> feels /extremely/ messy"

> "make a new arc - fix this... client holds two opposite ends...
> server holds two opposite ends... that's the most logical
> separation... the ack channel is always a no-op message, the
> req channel always a vec of batches items (who could be a vec
> of 1)"

The "extremely messy" diagnosis was load-bearing. The
embedded-reply-tx pattern leaked the multi-client routing concern
into the request shape; pair-by-index moves it to setup time
(HandlePool::pop) where it belongs. Console (arc 089 slice 5)
already did this; Service<E,G> just needed to follow.

---

## Test coverage

Workspace summary: `cargo test --workspace` zero failures across
all crates; all 85 test groups green. Pulse benchmark
verification deferred to lab next-session (cross-repo).

Specific lock points:
- `wat-tests/std/telemetry/Service.wat` — three deftests through
  the new batch-log path.
- `crates/wat-sqlite/wat-tests/std/telemetry/{Sqlite, auto-spawn,
  edn-newtypes}.wat` — every Sqlite consumer's lifecycle
  exercises Handle pop + 3-arg batch-log.

---

## Files changed

Substrate:
- `wat-rs/wat/std/telemetry/Service.wat` — protocol pivot.
- `wat-rs/wat-tests/std/telemetry/Service.wat` — test sweep.

wat-sqlite:
- `crates/wat-sqlite/wat/std/telemetry/Sqlite.wat` — Handle/DriverPair
  shape, batch-log 3-arg signature, paired channel construction in spawn.
- `crates/wat-sqlite/wat-tests/std/telemetry/Sqlite.wat`
- `crates/wat-sqlite/wat-tests/std/telemetry/auto-spawn.wat`
- `crates/wat-sqlite/wat-tests/std/telemetry/edn-newtypes.wat`

Documentation:
- `wat-rs/docs/arc/2026/04/095-service-paired-channels/DESIGN.md`
- `wat-rs/docs/arc/2026/04/095-service-paired-channels/INSCRIPTION.md` (this file)
