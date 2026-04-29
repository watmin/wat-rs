# Arc 089 — Batch-as-Protocol — DESIGN

**Status:** in design 2026-04-29.

The substrate's destination services (`:wat::std::telemetry::Service<E,G>`,
`:wat::std::telemetry::Sqlite/spawn` + `auto-spawn`, `:wat::lru::CacheService`)
were built around a per-entry dispatcher contract. The archive's pre-wat-native
implementation proved a different shape: **the batch IS the protocol.** Every
cross-thread call carries a `Vec<T>` of work, the driver drains all clients
into one combined batch, processes it as a single dense unit, then acks. The
shape generalizes — database flushes, cache lookups, treasury queries all use
it. Switching to it is not just a perf fix; it's the substrate respecting the
work-unit boundary the consumer already has.

This arc rebuilds the substrate's destination plumbing around batch-as-protocol
and removes a policy decision (forced WAL pragma) that snuck in during the
arc-085 push.

## What we know

### The archive's discipline (verified — `archived/pre-wat-native/`)

Four rules, consistent across `database.rs`, `cache.rs`, `treasury_program.rs`:

1. **No fire-and-forget across thread boundaries — "in-memory TCP."**
   Every cross-thread request/reply pair uses bounded(1) request queue +
   bounded(1) ack queue. Producer sends the request, then blocks on the
   ack until the driver has *completed the work* (flushed the batch,
   committed the transaction, written to the IOWriter). Bounded(1)
   provides backpressure on accept; the ack provides backpressure on
   completion. Together they prevent buildup and guarantee the producer
   never outruns the destination's actual durability boundary.

   This is the strict version of crossbeam's bounded send. With bounded
   send alone, the producer unblocks when the message lands in the queue
   — but the work hasn't happened yet. With an ack channel, the producer
   unblocks when the work is *done*. That's the difference between
   "buffered" and "durable."

   The distinction matters for **request/reply destinations** — Service,
   Console, Cache, Treasury. For **pure-dataflow streams** (map / filter
   combinators in `wat/std/stream.wat`) bounded send is sufficient — the
   data flow IS the protocol, no separate ack needed.

2. **Vec-carry every request.** Even single-key test helpers wrap in
   `vec![key]` (cache.rs:278). Batching is the protocol; singletons are
   batches of length 1.

3. **One round-trip per work unit.** The broker accumulates ~28 LogEntries per
   candle (BrokerSnapshot + PhaseSnapshot + ~25 metrics rows) into one
   `pending: Vec<LogEntry>`, then calls `flush_metrics(&db_tx, &mut pending)`
   ONCE per candle (broker_program.rs:354-413). Treasury's paper-state
   lookup: ONE `batch_get_paper_states(Vec<u64>)` for all active papers.

4. **Driver drains ALL clients before processing.** `select` wakes; the loop
   tries every client's queue; combines into one local Vec; processes once;
   acks everyone who contributed (database.rs:159-180, cache.rs:178-196).

### The transaction win (database.rs:224-231)

```rust
fn flush<T>(conn: &Connection, batch: &[T], insert: &impl Fn(&Connection, &T)) {
    conn.execute_batch("BEGIN").expect("...");
    for entry in batch { insert(conn, entry); }
    conn.execute_batch("COMMIT").expect("...");
}
```

One transaction per batch. With WAL, this collapses N row-fsyncs to 1
batch-fsync. Measured today: 1000-candle pulse goes from 17s (auto-commit per
row) → 5.4s (WAL alone, still per-row commits). Adding the transaction wrap
should drop another order of magnitude.

### The substrate gap (measured)

| Site | Today | Archive | Gap |
|---|---|---|---|
| `Service<E,G>::loop` | One-batch-from-one-client → process → ack → recurse | Drain ALL clients → process combined → ack all | No cross-client drain |
| `Service<E,G>` dispatcher | `:fn(E)->()` per-entry | `Vec<T>` to `flush(conn, batch, insert)` | Sinks can't see batch boundary |
| `Sqlite/auto-dispatch` | Per-entry INSERT, auto-commit | BEGIN/COMMIT around per-batch insert loop | No transaction wrap |
| `:rust::sqlite::Db::open` | Forces `journal_mode = WAL` | Just opens connection | Substrate is making a policy choice it shouldn't |
| `:wat::std::service::Console/out` and `/err` | Fire-and-forget — `send` only, no ack | (archive's ConsoleHandle was different — message goes straight to stdio, but ack-shape applies the same principle) | No ack channel; producer unblocks when message is queued, not when driver has written |

Pure-dataflow streams (`wat/std/stream.wat` map/filter etc.) are NOT in the
gap. They use bounded send for backpressure; they're not request/reply
destinations.

### What pragma is (proxy framing)

We are a thin proxy to rusqlite. The substrate has no opinion on
journal_mode, synchronous, cache_size, foreign_keys, mmap_size, page_size, or
any other pragma. Consumers pick. The substrate's job is to forward the call
to `Connection::pragma_update`.

## What we don't know

- **Pragma read form (`pragma-query`).** rusqlite has `pragma_query` and
  `pragma` (write+read) too. No consumer needs to READ a pragma value yet.
  When one does (likely: someone wants to verify journal_mode was set, or
  read `synchronous` to display in startup banner), we add `Db::pragma-query`
  in a follow-up. Today's slice ships write-only. **Declared unknown:** the
  shape of pragma-query when it's needed (probably `db name -> String`,
  panicking on no-result).

- **Frame hook on Service<E,G> vs per-batch dispatcher.** Two designs at the
  Service<E,G> level produce the same outcome:

  (a) Keep dispatcher per-entry; add a `frame :fn(()->())->()` hook that
      wraps the per-batch foldl. Sqlite passes a frame doing
      `(Db/begin db) (thunk) (Db/commit db)`.

  (b) Change dispatcher to `:fn(Vec<E>)->()` outright. Sqlite's dispatcher
      does the BEGIN/foldl/COMMIT itself. Console's dispatcher foldls
      per-entry.

  (b) is more honest — sinks naturally receive batches and can decide what
  to do with the boundary (transaction wrap, single fwrite, foldl, etc.).
  (a) preserves the per-entry contract for sinks that don't care about
  batches but adds a separate hook channel.

  **Decision:** going with (b). It collapses two hooks (dispatcher + frame)
  into one (dispatch-batch) and matches the archive's `flush(conn, batch,
  insert)` signature exactly. Console's added foldl is one line.

- **Whether cross-client drain matters at our current scale.** The lab has
  one telemetry client (the pulse program). Cross-client drain helps when N
  programs share one Service. We're putting it in because the archive proved
  the shape; the cost is small and we'd otherwise have to retrofit later.

## Slices

Each slice ships independently and stands alone. Run order matters: 1 first
(removes forced WAL — fixes a policy regression), then 2 (drain-all loop),
then 3 (per-batch dispatch contract), then 4 (pre-install hook gives consumers
a clean place to call `pragma`). After 4, the lab can opt back into WAL +
batched transactions on its own terms.

### Slice 1 — Substrate `Db` primitives, no policy

**What:** Add three methods to `:rust::sqlite::Db`. Remove the forced WAL
pragma from `Db::open`.

```rust
#[wat_dispatch(path = ":rust::sqlite::Db", scope = "thread_owned")]
impl WatSqliteDb {
    pub fn open(path: String) -> Self { /* no pragmas */ }
    pub fn execute_ddl(&mut self, ddl: String) { /* unchanged */ }
    pub fn execute(&mut self, sql: String, params: Vec<Value>) { /* unchanged */ }

    /// `:rust::sqlite::Db::pragma db name value` —
    /// `conn.pragma_update(None, name, value)`. Substrate forwards;
    /// substrate has no opinion on which pragmas a consumer sets or
    /// what values they choose. Examples (consumer-side):
    ///
    ///     (Db/pragma db "journal_mode" "WAL")
    ///     (Db/pragma db "synchronous" "NORMAL")
    ///     (Db/pragma db "cache_size" "10000")
    ///     (Db/pragma db "foreign_keys" "ON")
    pub fn pragma(&mut self, name: String, value: String) {
        self.conn.pragma_update(None, name.as_str(), value)
            .unwrap_or_else(|e| panic!(":rust::sqlite::Db::pragma: {name}={value}: {e}"));
    }

    /// `:rust::sqlite::Db::begin db` — `BEGIN;`
    pub fn begin(&mut self) {
        self.conn.execute_batch("BEGIN")
            .unwrap_or_else(|e| panic!(":rust::sqlite::Db::begin: {e}"));
    }

    /// `:rust::sqlite::Db::commit db` — `COMMIT;`
    pub fn commit(&mut self) {
        self.conn.execute_batch("COMMIT")
            .unwrap_or_else(|e| panic!(":rust::sqlite::Db::commit: {e}"));
    }
}
```

`pragma_update`'s third param is `&dyn ToSql`; rusqlite renders `&str`
correctly for SQLite's pragma syntax (bare or quoted both work).

**Files:**
- `wat-rs/crates/wat-sqlite/src/lib.rs` — add three methods, remove WAL.
- `wat-rs/crates/wat-sqlite/wat/sqlite/Db.wat` — wat-side surface (the
  macro auto-generates binding; this file documents).

**Tests:** `wat-rs/crates/wat-sqlite/wat-tests/sqlite/Db.wat` — open, set
WAL pragma, execute_ddl, begin/insert/commit/insert (verify no auto-commit
inside transaction by inspecting row count after rollback, etc.).

**Done when:** the test passes and the existing `:rust::sqlite::Db::execute`
test still works. No INSCRIPTION yet — the slice is part of arc 089.

### Slice 2 — `Service/loop` drains all clients

**What:** Replace the "one-batch-from-one-client → process → ack → recurse"
loop with the archive's drain-all pattern.

Today (Service.wat:158-216):

```scheme
;; Get one Request from one client.
(:wat::core::match (:wat::kernel::select rxs) -> :()
  ((Some req)
    ;; foldl entries through dispatcher
    ;; ack just this one client
    ;; tick-window
    ;; recurse))
```

After:

```scheme
;; Block on select until SOMETHING arrives.
;; Drain ALL clients' pending Requests into Vec<Request<E>>.
;; Combine batches into one Vec<E>.
;; dispatch-batch combined
;; ack EACH client whose batch we drained.
;; tick-window
;; recurse
```

The `dispatch` change comes in slice 3; for slice 2 we keep the per-entry
contract and just rebuild the loop shape.

**Open detail:** `try_recv`-equivalent at the wat layer. Today
`:wat::kernel::select rxs` blocks until one is ready. We need a
non-blocking poll. Two options:
- Add `:wat::kernel::try-recv rx -> Option<T>` primitive.
- Use `select` with a timeout of zero.

Cleanest: add `try-recv`. Crossbeam already exposes it; substrate just needs
the wat-level surface. **Sub-task** for slice 2: ship `try-recv` if it's not
already there. (Verify before assuming missing.)

**Files:**
- `wat-rs/wat/std/telemetry/Service.wat` — rewrite `Service/loop`.
- `wat-rs/src/runtime.rs` + check.rs — `try-recv` if missing.
- `wat-rs/wat-tests/std/telemetry/Service.wat` — multi-client batch test.

### Slice 3 — Per-batch dispatch contract

**What:** Change `Service<E,G>` dispatcher from `:fn(E)->()` to
`:fn(Vec<E>)->()`.

Today:

```scheme
(dispatcher :fn(E)->())
;; Service/loop:
(:wat::core::foldl entries () (:wat::core::lambda ((acc :()) (e :E) -> :())
  (dispatcher e)))
```

After:

```scheme
(dispatch :fn(Vec<E>)->())
;; Service/loop:
(dispatch entries)
```

Sinks decide what to do with the batch:
- Console: `dispatch = (lambda (entries) -> () (foldl entries () (lambda (acc e) -> () (Console/out con-tx (format e)))))`
- Sqlite: `dispatch = (lambda (entries) -> () (Db/begin db) (foldl entries () dispatcher) (Db/commit db))`
- File: one combined `IOWriter/write` of the joined buffer.

The substrate's `:rust::sqlite::auto-dispatch` shim grows a batch sibling:

```rust
fn dispatch_auto_dispatch_batch(args, env, sym) -> Result<Value, RuntimeError> {
    // args: db, enum-name, entries: Vec<Value>
    // for each entry: variant lookup, bind, execute (NO begin/commit here —
    // batching is the wat-side caller's call via Db/begin + Db/commit)
}
```

Or: keep `auto-dispatch` per-entry, expose `Db/begin` + `Db/commit` to wat,
and have `Sqlite/auto-spawn`'s dispatch lambda do the wrap. **Decision:**
the second. Substrate primitives stay simple; wat composition does the wrap.
Auto-spawn's dispatch becomes:

```scheme
(:wat::core::lambda ((entries :Vec<E>) -> :())
  (:wat::core::let*
    (((_b :()) (:rust::sqlite::Db::begin db)))
    (:wat::core::let*
      (((_d :())
        (:wat::core::foldl entries ()
          (:wat::core::lambda ((acc :()) (e :E) -> :())
            (:rust::sqlite::auto-dispatch db enum-name e)))))
      (:rust::sqlite::Db::commit db))))
```

**Files:**
- `wat-rs/wat/std/telemetry/Service.wat` — type signature + foldl removal.
- `wat-rs/wat/std/telemetry/Console.wat` — Console dispatcher gains internal foldl.
- `wat-rs/crates/wat-sqlite/wat/std/telemetry/Sqlite.wat` — auto-spawn's
  dispatch becomes the batch-wrapping lambda above.
- `wat-rs/wat-tests/std/telemetry/{Service,Console,Sqlite}.wat` — tests
  updated to the new shape.

### Slice 4 — `pre-install` hook on `Sqlite/spawn` + `Sqlite/auto-spawn`

**What:** Add one optional hook to the substrate's sqlite spawn surface.

```scheme
(:wat::core::define
  (:wat::std::telemetry::Sqlite/spawn<E,G>
    (path :String)
    (count :i64)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    (pre-install :fn(wat::sqlite::Db)->())   ;; NEW — runs after open, before schema-install
    (schema-install :fn(wat::sqlite::Db)->())
    (dispatch :fn(wat::sqlite::Db,Vec<E>)->())
    (stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<E>)
    -> :wat::std::telemetry::Service::Spawn<E>)
  ...)
```

`Sqlite/auto-spawn` grows the same parameter and forwards. Default for
"don't care" is a no-op lambda — the substrate ships
`Sqlite/null-pre-install` for the explicit opt-out, mirroring
`null-metrics-cadence`.

**Files:**
- `wat-rs/crates/wat-sqlite/wat/std/telemetry/Sqlite.wat` — add `pre-install`
  to `Sqlite/run`, `Sqlite/spawn`, `Sqlite/auto-spawn`. Add
  `Sqlite/null-pre-install` helper.
- `wat-rs/crates/wat-sqlite/wat-tests/std/telemetry/Sqlite.wat` — test that
  pre-install runs in the worker thread, before schema-install, with the
  Db value the worker holds.

**Done when:** lab can write

```scheme
(:wat::std::telemetry::Sqlite/auto-spawn
  :trading::log::LogEntry
  path
  1
  (:wat::std::telemetry::Service/null-metrics-cadence)
  (:wat::core::lambda ((db :wat::sqlite::Db) -> :())
    (:wat::std::sqlite::Db::pragma db "journal_mode" "WAL")
    (:wat::std::sqlite::Db::pragma db "synchronous" "NORMAL")))
```

and the resulting db file has WAL journal mode confirmed by external sqlite
inspection.

### Slice 5 — Console gains ack channel ("in-memory TCP")

**What:** Replace `:wat::std::service::Console`'s fire-and-forget send
with the same send+ack pattern Service<E,G> uses. Producer's `Console/out`
or `Console/err` blocks until the driver has called `IOWriter/write-string`
on the underlying writer.

The current message protocol:

```scheme
(:wat::core::typealias :wat::std::service::Console::Message
  :(i64,String))   ;; (tag, msg)
```

becomes:

```scheme
(:wat::core::typealias :wat::std::service::Console::Message
  :(i64,String,wat::std::service::Console::AckTx))   ;; (tag, msg, ack-tx)
```

Each client gets a bounded(1) ack channel at setup time alongside its
request channel. The `Console/out` / `Console/err` helpers become:

```scheme
(:wat::core::define
  (:wat::std::service::Console/out
    (handle :wat::std::service::Console::Tx)
    (ack-tx :wat::std::service::Console::AckTx)
    (ack-rx :wat::std::service::Console::AckRx)
    (msg :String)
    -> :())
  (:wat::core::let*
    (((_send :Option<()>)
      (:wat::kernel::send handle (:wat::core::tuple 0 msg ack-tx)))
     ((_recv :Option<()>) (:wat::kernel::recv ack-rx)))
    ()))
```

Driver loop after writing each message sends `()` on the per-message
`ack-tx`. Same lifecycle handling as before — disconnected ack-tx is
swallowed silently (caller is gone, nothing to wake).

**Open question:** does `Console/spawn`'s API grow to return ack channels
alongside the request senders, or do callers create their own ack channel
per client? The Service<E,G> pattern has callers create their own ack
channel; that gives the caller flexibility (one ack per logical scope vs
one per client) and keeps `Console/spawn`'s return type stable. Going
with the same pattern: `Console/spawn` returns the same shape as today
(pool of req-tx + driver handle); callers create their ack channels at
their own scope.

**Files:**
- `wat-rs/wat/std/service/Console.wat` — protocol grows ack-tx into the
  Message tuple; driver sends ack after write; client helpers gain
  ack-tx + ack-rx parameters; doc comment loses the "fire-and-forget"
  language.
- `wat-rs/wat/std/telemetry/ConsoleLogger.wat` — ConsoleLogger captures
  its own ack pair; `Logger/log` threads it through.
- All callers of `Console/out` / `Console/err` (lab pulse, lab smoke,
  lab bare-walk, examples) — pass ack channels.
- `wat-rs/wat-tests/std/service/Console.wat` — verify producer blocks
  until driver has written (test using a slow IOWriter or a count
  inspection).

**Done when:** producer's `Console/out` returns ONLY after the driver's
`IOWriter/write-string` has completed. Verified by a test that wedges
the writer (e.g., a counter-based mock) and observes the producer
remains blocked until the wedge releases.

## What's NOT in this arc

- **Cache batch primitives.** `:wat::lru::CacheService::get`/`put` is
  per-key. The archive's `batch_get` / `batch_set` shape is right but
  belongs to its own arc — different crate (wat-lru), different consumers,
  no current lab caller. Skeleton at
  `docs/arc/2026/04/090-cache-batch-primitives/`.

- **Lab consumer rewrite.** Once slice 4 ships, the lab updates
  `:trading::telemetry::Sqlite/spawn` (currently a 5-line auto-spawn
  delegate) to pass its own pragmas via `pre-install`. That's a single-file
  follow-up, not an arc — handled as a slice on the lab's proposal 059.

- **Pragma read form.** Deferred. Add when a consumer needs it.

## Order of operations

```
slice 1 (primitives + remove forced WAL)
   ↓
slice 2 (drain-all loop)
   ↓
slice 3 (per-batch dispatch)
   ↓
slice 4 (pre-install hook)
   ↓
slice 5 (Console gains ack channel)
   ↓
[lab: update :trading::telemetry::Sqlite/spawn with pragmas + deps,
      update Console/out callers to thread ack channels]
   ↓
[run pulse, measure, confirm dense work unit lands as one transaction]
```

Slices 1–4 can ship without 5 (Sqlite perf doesn't depend on Console
ack); slice 5 is independent and can ship at its own pace. They're in
the same arc because they share the principle ("no fire-and-forget,
batch is the protocol") and revisiting the substrate's RPC plumbing
twice would mean two consumer-side sweeps.

After slice 4, INSCRIPTION captures the shipped shape and the measured perf
delta on the pulse program.
