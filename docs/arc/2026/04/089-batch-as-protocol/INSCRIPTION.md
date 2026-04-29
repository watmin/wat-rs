# Arc 089 — Batch-as-Protocol — INSCRIPTION

**Status:** shipped 2026-04-29 (slices 1–4). Slice 5 (Console ack channel)
held for separate consideration — it's correctness work on a different
axis and touches every `Console/out` call site; deferred until a consumer
or test surfaces a real failure mode that requires it.

The substrate's destination services (`:wat::std::telemetry::Service<E,G>`,
`:wat::std::telemetry::Sqlite/spawn` + `auto-spawn`) were rebuilt around
the archive's discipline at
`archived/pre-wat-native/src/programs/stdlib/database.rs`: batch is the
protocol, drivers drain all clients before processing, and the per-batch
boundary is exposed to sinks so they can wrap it (transactions, single
fwrite, etc.). Forced WAL pragma in `:rust::sqlite::Db::open` was
removed — substrate ships zero pragma defaults, consumers pick.

Measured win on the lab's pulse program (1000 BTC candles, 1100 telemetry
rows in 11 batches):

| | Before WAL | WAL only | Arc 089 (slices 1–4) |
|---|---:|---:|---:|
| Total runtime | ~17.0s | 5.43s | **0.046s** |
| flush_ns total | (dominant) | 5.13s | **0.022s** |
| Per-row cost | ~17 ms | ~5 ms | ~46 µs |

**117× total, 233× on flush.** The shape matches the archive's pattern:
each work unit (one drained batch) becomes one fsync, not N. The
substrate now respects the work-unit boundary the consumer already had.

---

## What shipped

### Slice 1 — Substrate `Db` primitives, no policy

`crates/wat-sqlite/src/lib.rs`:

- Removed forced `journal_mode = WAL` from `WatSqliteDb::open`. Substrate
  has no opinion on journal_mode, synchronous, cache_size, foreign_keys,
  mmap_size, page_size, or any other pragma.
- Added three methods to `:rust::sqlite::Db`:

```rust
pub fn pragma(&mut self, name: String, value: String);  // PRAGMA <name> = <value>;
pub fn begin(&mut self);                                 // BEGIN;
pub fn commit(&mut self);                                // COMMIT;
```

`pragma` is a thin proxy to `conn.pragma_update(None, name, value)`.
rusqlite's `&str` ToSql renders correctly for SQLite's pragma syntax —
bare or quoted both work for `journal_mode = WAL` etc.

`crates/wat-sqlite/wat/sqlite/Db.wat` exposes the wat-side wrappers
`:wat::sqlite::pragma`, `begin`, `commit`.

Read form (`pragma_query`) deferred. No consumer needs it yet; the
DESIGN flagged it as a known-unknown.

### Slice 2 — `Service/loop` drains all clients

`wat/std/telemetry/Service.wat` rebuilt around the archive's
`database()` shape (`database.rs:127-211`):

```
loop:
  block on select until ANY rx ready
  on :None  → remove the disconnected rx, recurse
  on Some(first-req):
    seed Pending<E> with first-req
    drain-rest — try-recv every other rx (each bounded(1), so at most
                 one queued); accumulate entries + ack-txs
    dispatch combined batch through the per-batch dispatcher (slice 3)
    ack EVERY contributing client (preserves "in-memory TCP" — producer's
        batch-log unblocks only after the work is done)
    bump-stats with combined batch size
    tick-window
    recurse with (stats', cadence')
```

Decomposed into small named functions (`extend`, `maybe-merge`,
`drain-rest`, `ack-all`, `bump-stats`, `loop-step`, `loop`) per the
project memory's "one let* per function" discipline.

Cross-client drain is observable when N>1 producers share one Service.
Today's lab has one producer per Service; the value-add latents until
the trader's multi-asset future. Existing 1-client tests all preserved
their semantics.

### Slice 3 — Per-batch dispatch contract

`Service<E,G>` dispatcher contract changed from `:fn(E)->()` to
`:fn(Vec<E>)->()`. Sinks see the batch boundary; per-entry vs per-batch
dispatch is the sink's choice.

- `Service/loop-step` calls `(dispatcher entries)` once per drained batch.
- `Service/tick-window` calls `(dispatcher (translator stats))` once
  per cadence fire instead of foldl-per-entry.
- Removed `Service/foldl-dispatch` helper — no longer needed.

`wat/std/telemetry/Console.wat`: `Console/dispatcher` returns a
`:fn(Vec<E>)->()` that internally foldls per-entry through
`Console/out` (Console is per-line by nature; the foldl moved INSIDE
the dispatcher, not outside).

`crates/wat-sqlite/wat/std/telemetry/Sqlite.wat`: introduced
`Sqlite::auto-dispatch-batch` — wraps the substrate's
`auto-dispatch` shim per-entry inside a `Db/begin` … `Db/commit` pair.
This is the archive's `flush()` discipline at
`database.rs:224-231`. `Sqlite/auto-spawn` composes this as the
dispatcher lambda.

### Slice 4 — `pre-install` hook on `Sqlite/spawn` + `Sqlite/auto-spawn`

```scheme
(:wat::std::telemetry::Sqlite/spawn
  path count cadence
  pre-install        ;; NEW — :fn(Db)->() runs after open, before schema-install
  schema-install
  dispatcher
  stats-translator)
```

`pre-install` is the seam for consumer pragma policy. Substrate ships
zero defaults — consumers add `(Db/pragma db "journal_mode" "WAL")`,
`(Db/pragma db "synchronous" "NORMAL")`, `(Db/pragma db "foreign_keys"
"ON")`, etc. Substrate ships `Sqlite/null-pre-install` for the
explicit "I'm fine with sqlite's defaults" opt-out, mirroring
`Service/null-metrics-cadence`.

Lab's `:trading::telemetry::Sqlite/spawn` now hardcodes its own
pre-install (WAL + synchronous=NORMAL) — the lab's policy decision,
not the substrate's:

```scheme
(:wat::core::define
  (:trading::telemetry::Sqlite::pre-install
    (db :wat::sqlite::Db) -> :())
  (:wat::core::let*
    (((_w :()) (:wat::sqlite::pragma db "journal_mode" "WAL"))
     ((_s :()) (:wat::sqlite::pragma db "synchronous" "NORMAL")))
    ()))
```

A different lab with stricter durability needs would pass
`synchronous=FULL` or omit WAL entirely. The substrate refuses to make
that choice for them.

---

## What's NOT in this arc

### Slice 5 — Console gains ack channel

`:wat::std::service::Console/out` and `Console/err` are still
fire-and-forget. Bounded(1) provides backpressure on accept; an ack
channel would provide backpressure on completion ("the data is durable
in IOWriter, not just queued"). The DESIGN.md sketches the shape.

Held because:
- The perf win the user named (5.4s → 0.046s) doesn't depend on
  Console acks.
- Slice 5 touches every `Console/out` and `Console/err` call site in
  the substrate, examples, and lab — wide churn for a correctness
  property no current test exercises.
- A Service-shape Console refactor (Console becomes a Service<Message,()>
  consumer) is the cleaner long-term shape but a bigger arc.

When a consumer surfaces a real failure mode (e.g., logs lost on crash,
ordering inversion across producers), slice 5 ships in its own arc.

### Cache batch primitives — Arc 090

`:wat::lru::CacheService` is still per-key (`get key`, `put key val`).
The archive's `batch_get(Vec<K>) -> Vec<(K,Option<V>)>` shape is right
but no current consumer needs it. Skeleton at
`docs/arc/2026/04/090-cache-batch-primitives/` waits for a real
consumer.

### Pragma read form

`:rust::sqlite::Db::pragma-query` deferred. No consumer needs to read
pragma values yet. When one does (likely: someone wants to verify
journal_mode landed, or read synchronous for a startup banner), the
shape is `db name -> String`, panicking on no-result.

---

## Surfaced by

User direction 2026-04-28:

> "do not box us into something shitty in our core lang - the user
> must make whatever choices and we'll forward them.... we're just a
> proxy to the rust sqlite lib - ya?"

> "in our archive - we spent a lot of time figuring out intra-thread
> rpc using request/reply structs for tx/rx - make sure we're using
> the patterns we found rewarding... a work unit must be as dense as
> we can make it..."

> "we also learned that fire and forget is undesirable.... a producer
> writing to a pipe is blocked until a ack is received... this is an
> 'in memory tcp' if you get my gist"

The first quote shaped slice 1 (no forced WAL, generic pragma proxy).
The second shaped slices 2 + 3 (drain-all, batch-as-protocol). The
third shaped slice 5 (deferred — Console ack channel) and reinforced
slices 2-3 (the substrate's Service already had send + ack on
`batch-log`; slices 2-3 made the BATCH the unit of that ack rather
than per-entry dispatches that hide the work-unit boundary).

The archive these came from:
- `archived/pre-wat-native/src/programs/stdlib/database.rs` — `flush()`
  at lines 224-231, drain-all at 159-180, BEGIN/COMMIT discipline.
- `archived/pre-wat-native/src/programs/stdlib/cache.rs` — typed
  Request/Response enums with Vec carry, drain-all at 178-196,
  writes-first-reads-second.
- `archived/pre-wat-native/src/programs/app/broker_program.rs:354-413`
  — per-candle `pending: Vec<LogEntry>` accumulator, ONE `flush_metrics`
  per work unit.
- `archived/pre-wat-native/src/programs/telemetry.rs` — `flush_metrics`
  helper signature.

---

## Test coverage

Substrate suite — 1413 tests passing across the workspace after slices 1–4.
New tests:

- `crates/wat-sqlite/wat-tests/sqlite/Db.wat` — `test-pragma-wal`,
  `test-begin-commit`. Verified out-of-band: `sqlite3 ... 'PRAGMA
  journal_mode'` returns `wal`; `SELECT SUM(n) FROM counters` returns
  6 (= 1+2+3 inside one transaction).
- `crates/wat-sqlite/wat-tests/std/telemetry/Sqlite.wat` —
  `pragma-wal` helper exercises slice-4's pre-install hook with a
  real non-trivial body. The produced db file's `journal_mode` is
  `wal` (verified via sqlite3 CLI).
- Existing Service/Console/auto-spawn tests updated to the per-batch
  dispatcher shape (slice 3) and the new `pre-install` parameter (slice
  4); all pass with no semantic regression.

Lab end-to-end:

- `holon-lab-trading/wat/programs/pulse.wat` — 1000 candles. Phase
  timing breakdown emitted as `pulse.timing` rows in the run db; SQL
  query confirms the 117× total / 233× flush speedup.

---

## Files changed

Substrate:
- `wat-rs/crates/wat-sqlite/src/lib.rs` — `pragma`, `begin`, `commit`;
  removed forced WAL.
- `wat-rs/crates/wat-sqlite/wat/sqlite/Db.wat` — wat surface for the
  three new primitives.
- `wat-rs/wat/std/telemetry/Service.wat` — drain-all loop;
  per-batch dispatcher; stats accumulation; `Pending<E>` typealias.
- `wat-rs/wat/std/telemetry/Console.wat` — `render-line` extracted;
  `Console/dispatcher` returns `:fn(Vec<E>)->()`.
- `wat-rs/crates/wat-sqlite/wat/std/telemetry/Sqlite.wat` —
  `pre-install` parameter on `Sqlite/run`, `Sqlite/spawn`,
  `Sqlite/auto-spawn`; `Sqlite/null-pre-install` opt-out helper;
  `auto-dispatch-batch` wrapping per-batch BEGIN/COMMIT.

Tests:
- `wat-rs/crates/wat-sqlite/wat-tests/sqlite/Db.wat`
- `wat-rs/wat-tests/std/telemetry/Service.wat`
- `wat-rs/wat-tests/std/telemetry/Console.wat`
- `wat-rs/crates/wat-sqlite/wat-tests/std/telemetry/Sqlite.wat`
- `wat-rs/crates/wat-sqlite/wat-tests/std/telemetry/auto-spawn.wat`

Lab:
- `holon-lab-trading/wat/io/telemetry/Sqlite.wat` —
  `:trading::telemetry::Sqlite::pre-install` (WAL +
  synchronous=NORMAL); `Sqlite/spawn` forwards it to auto-spawn.

Documentation:
- `wat-rs/docs/arc/2026/04/089-batch-as-protocol/DESIGN.md`
- `wat-rs/docs/arc/2026/04/089-batch-as-protocol/INSCRIPTION.md` (this file)
- `wat-rs/docs/arc/2026/04/090-cache-batch-primitives/DESIGN.md` (skeleton
  for the deferred wat-lru cache batch arc)
