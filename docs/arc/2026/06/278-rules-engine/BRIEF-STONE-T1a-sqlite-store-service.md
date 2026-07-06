# BRIEF — T1a: promote `SqliteStore` into the `sqlite-store'` SERVICE (so a sink can be GIVEN it)

> **Executor: one sonnet SHADOWDANCER.** The orchestrator scouted the lair, cast intueri on the naming (verdict A,
> weighed), and PROVED the novel composition with a disconfirming probe (green — a defservice holds a thread-owned
> Connection in `:ephemeral`, does fallible SQL, survives across RPCs). Work ONLY in
> `/home/watmin/work/holon/wat-rs/` (`pwd` first; `.claude/worktrees/` illegal). `cargo wat <f>` to dogfood;
> `cargo nextest run --release` (NEVER `cargo test`); `cargo build` to check. **Commit NOTHING.**

## The work (one paragraph)

Promote S2's struct-over-a-live-`Connection` `SqliteStore` into a **`sqlite-store'` defservice** — an actor that owns
the `Connection` on its own thread — plus a **`SqliteStore` satisfier that wraps the service PEER**, mirroring
`mem-store'`/`MemStore` almost line-for-line. This makes the sqlite store **wireable**: a telemetry sink (T1b) can be
*given* a store peer, where a live `Connection` could never cross the wire. The **SQL and all pure helpers already
exist** in `wat/query/sqlite-store.wat` (S2) — this is a **relocation + reshape**, not a rewrite. Also MOVE the whole
sqlite satisfier from `:wat::sqlite'` into `:wat::query` (intueri verdict A — a Store satisfier lives with the
CONTRACT, like `MemStore`, not with the raw driver). The raw driver (`:wat::sqlite'::Connection`/`open`/`execute`/
`select`/`Param`/`Cell`/`Error`/`Fault`) STAYS in `:wat::sqlite'`.

## The proven composition shape (copy this — it type-checks + runs; the orchestrator's probe)

```wat
;; a defservice holding a thread-owned :wat::sqlite'::Connection in :ephemeral, opened in :init,
;; ops doing FALLIBLE SQL and returning the Result IN the response record. PROVEN (create=Ok insert=Ok).
(:wat::service::defservice :probe::sqlite-svc
  :durable   [path <- :wat::core::String]
  :ephemeral [conn <- :wat::sqlite'::Connection]
  :init (:wat::core::fn [record <- :probe::sqlite-svc::Record] -> :probe::sqlite-svc::State
          (:probe::sqlite-svc::State record
            (:wat::core::Result/expect (:wat::sqlite'::open (:probe::sqlite-svc::Record/path record)) "open")))
  :ops
  [(:Create [s <- :State] -> [result <- :wat::core::Result<wat::core::nil,wat::sqlite'::Error>]
     (:wat::service::Outcome::Reply s
       (:probe::sqlite-svc::CreateResponse
         (:wat::sqlite'::execute-ddl (:probe::sqlite-svc::State/conn s) "CREATE TABLE t (id INTEGER)"))))
   ;; NOTE the checker taught: execute -> Result<i64,Error> (rows-affected); execute-ddl -> Result<nil,Error>.
   (:Insert [s <- :State] -> [result <- :wat::core::Result<wat::core::i64,wat::sqlite'::Error>]
     (:wat::service::Outcome::Reply s
       (:probe::sqlite-svc::InsertResponse
         (:wat::sqlite'::execute (:probe::sqlite-svc::State/conn s) "INSERT INTO t (id) VALUES (?)"
           (:wat::core::Vector :wat::sqlite'::Param (:wat::sqlite'::Param::I64 1))))))])
;; construction (inline, mem.wat's scope law): start -> Handle/addr -> connect' -> peer -> RPC
;; (:probe::sqlite-svc/start :locus (:wat::spawn::thread) :record (:probe::sqlite-svc::Record ":memory:"))
;; (:wat::kernel::connect' (:probe::sqlite-svc::Handle/addr h)) ; (:probe::sqlite-svc/create c (:probe::sqlite-svc/create-request))
```
(full file: `scratchpad/probe-sqlite-svc-compose.wat` — copy its skeleton for the service shape.)

## Read the rooms, in order
1. **`wat/query/mem.wat`** — THE MIRROR. The `mem-store'` defservice (ops → `Outcome::Reply s (Svc::OpResponse …)`),
   the `MemStore` defstruct wrapping the peer, the 4 `extend-type` methods doing the client RPC
   (`(mem-store'/put (MemStore/peer self) (mem-store'/put-request rows))` → read `PutResponse/…`), the `derive` to
   `ReadStore`, AND the **scope-NOTE** (start+connect'+construct INLINE — no convenience constructor; the S2
   `SqliteStore/open` helper is RETIRED for the same reason).
2. **`wat/query/sqlite-store.wat`** — S2, THE SOURCE. All the SQL + pure helpers (`lift-fault`/`lift-error`,
   `cell->string`, `row-from-cells`, `index-row-from-cells`, `build-page`, `build-index-page`, `ensure-index-tables`,
   `clear-index-projections`, `insert-index-projections`, `put-one-row`, `put-rows`, and the 4 Store-method bodies).
   These RELOCATE — the method bodies become the service op bodies; the pure helpers move namespace.
3. **`wat-tests/service-multiparam-init.wat`** — the `:init` law (`(Record, …operating-inputs) -> State`; resource in
   `:ephemeral`, opened in `:init`).
4. **`wat/query.wat`** — the `Store`/`ReadStore` contract + records (`StoredRow`/`Page`/`ScanRequest`/etc.).
5. **`src/stdlib.rs`** — `wat/query/sqlite-store.wat` is already baked (after mem.wat); keep it there.
6. **`tests/rete/probe_arc278_sqlite_store_differential.{wat,rs}`** — the S2 differential gate; its SqliteStore
   CONSTRUCTION changes (start the service + wrap the peer, inline); the differential ITSELF is preserved.

## Build order
1. **The `sqlite-store'` service** in `wat/query/sqlite-store.wat` (namespace `:wat::query`):
   - `(:wat::service::defservice :wat::query::sqlite-store' :durable [path <- :wat::core::String  index-names <- (:wat::core::Vector :wat::core::String)] :ephemeral [conn <- :wat::sqlite'::Connection] :init (fn [record] -> State  open conn from record.path + the WAL/NORMAL pragmas S2 sets) :ops [EnsureSchema Put Scan ScanIndex])`.
   - Ops carry the `Result` in their response (the SQL is fallible): `EnsureSchema`/`Put` → `Result<nil,query::Error>`;
     `Scan` → `Result<Page,query::Error>`; `ScanIndex` → `Result<IndexPage,query::Error>`. Each op's body is S2's
     corresponding Store-method body, calling S1 verbs on `(:wat::query::sqlite-store'::State/conn s)`, and `put` reads
     the GSI name list from `(Record/index-names (State/durable s))`. Reply with `s` UNCHANGED (the mutation is in
     sqlite via the conn — the service's State doesn't change).
   - The `sqlite'::Error → query::Error` lift stays as a helper, MOVED to `:wat::query` (it is the satisfier's
     mechanism). Do the lift where the op builds its `Result<_,query::Error>` response (so responses already carry
     `query::Error`, and the satisfier just forwards them).
2. **The `SqliteStore` satisfier** — `(:wat::core::defstruct :wat::query::SqliteStore [peer <- :wat::kernel::Peer'<wat::query::sqlite-store'::Op,wat::query::sqlite-store'::Reply>])`, `extend-type`d to `:wat::query::Store` (the 4 methods do the client RPC + return the response's `Result` — mirror MemStore's method shape exactly), `derive` to `:wat::query::ReadStore`.
3. **MOVE the pure helpers** (`cell->string`, `row-from-cells`, `index-row-from-cells`, `build-page`,
   `build-index-page`, `ensure-index-tables`, `clear-index-projections`, `insert-index-projections`, `put-one-row`,
   `put-rows`, `lift-fault`, `lift-error`) from `:wat::sqlite'::` to `:wat::query::`. Mechanical single-source rename;
   they reference `:wat::sqlite'::{Connection,Param,Cell,execute,select,…}` (the raw driver — those refs stay
   `:wat::sqlite'`). RETIRE `SqliteStore/open` (the scope-trap forbids a convenience constructor).
4. **Update the differential gate** `tests/rete/probe_arc278_sqlite_store_differential.wat`: construct the SqliteStore
   by starting `sqlite-store'` inline + wrapping the peer (mirror how MemStore is constructed there), then run the same
   ops. The differential assertion (mem-result == sqlite-result) is UNCHANGED — same ops → same Pages.

## STOP triggers (halt + report)
1. **STOP-COMPOSE:** if a defservice CANNOT hold the thread-owned Connection across RPCs, STOP — but it CAN (the probe
   proved it, green); if you hit "channel disconnected", the cause is the scope-trap (construction not inline), not the
   model — inline the start+connect'+RPCs in ONE scope.
2. **STOP-DIFFERENTIAL:** the gate must still compare mem == sqlite through the `Store` surface. If they DIVERGE after
   the promotion, STOP and report the exact rows/cursors — it's a real transcription bug, not a test to fudge.
3. **STOP-NAMESPACE:** the raw driver STAYS `:wat::sqlite'` (Connection/Param/Cell/open/execute/select/Error/Fault). Do
   NOT move those. ONLY the satisfier + its pure helpers + the lift move to `:wat::query`.
4. **STOP-NOCP:** do NOT change the S1 raw interop (`wat/sqlite.wat`, `src/rust_deps/sqlite.rs`), MemStore's logic, the
   `Store` method signatures, or `wat/query.wat`'s records.

## The gate (EXPECTATIONS)
| what | command | expected |
|---|---|---|
| the differential passes (mem == sqlite, now service-backed) | `cargo nextest run --release -E 'test(sqlite_store_differential)'` | passed |
| S-mem + S0 + S1 still green | `cargo nextest run --release -E 'test(smem_roundtrip) or test(query_contract) or test(sqlite_interop)'` | passed |
| whole floor | `cargo nextest run --release` | Summary line VERBATIM; `0 failed` modulo the known `no_inlined_wat` reminder |

## Final report: files changed · the `sqlite-store'` service (durable/ephemeral/init/ops) · the `SqliteStore`
satisfier (the 4 RPC methods + derive) · what moved namespace · the differential gate's new construction · the verbatim
`sqlite_store_differential` + whole-floor Summary · any divergence · STOP triggers hit or "none" · anything that surprised you.

## Prior comparable: S2 itself (`4e1ea3c9`, the SQL) + S-mem (`3304cbd5`, the service+satisfier mirror). The proven
composition probe is `scratchpad/probe-sqlite-svc-compose.wat`.
