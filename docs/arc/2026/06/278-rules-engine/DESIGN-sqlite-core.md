# DESIGN — `:wat::sqlite'`: sqlite in core, the first `Store` satisfier

> **STATUS: DESIGN (2026-07-05), unbuilt.** The core, primed sqlite that satisfies the `Store` contract
> (`DESIGN-store-contract.md`). It is **provided, not copied** — the arc-083/096 `wat-sqlite` battery is *prior art*
> (its capability-honest rw/ro split is worth providing again), never a `cp` source. It ships to current standard:
> errors as values, `deftest'`, records-are-EDN, one-way construction — and it adds the row-returning query primitive
> the old crate never had.

## Why core, why primed

- **Core.** The telemetry facility diagnoses rete, and rete is core; the facility depends on the store; therefore the
  store — **sqlite included** — is core. It lives in the substrate (`wat/`), loaded by default, adjacent to the rete
  tooling — not an opt-in battery.
- **Primed (`:wat::sqlite'` / `:rust::sqlite'`).** The `wat-sqlite` battery is **loaded** (`wat-cli/.../wat.rs:14`) and
  already owns a full `:wat::sqlite::*` namespace (`open`/`begin`/`commit`/`execute`/`Db`/`ReadHandle`) + `:rust::sqlite::*`.
  Leaving that bridge untouched (its own demise, on its own schedule) means the core version cannot reuse those names —
  so it primes: `:wat::sqlite'` (wat surface) + `:rust::sqlite'` (Rust bindings), **staged to replace** the battery,
  coexisting with it. Exactly the `:wat::telemetry'` pattern.
- **Provide capability, not replicate.** `cp` is the worst operation. `:wat::sqlite'` is designed from what the `Store`
  contract needs, taking wat-sqlite's *good ideas* (below) and leaving its *vintage* (panics, `deftest`, the missing
  query primitive) behind.

### Kept from the prior art (provided fresh, not copied)
- **The capability-honest rw/ro split** — a **read-write** handle vs a **read-only** handle opened `SQLITE_OPEN_READ_ONLY`;
  the type *is* the proof a reader cannot write (`ReadStore` in the contract). Genuinely good; provided again.
- **Typed `Param` enum** (no `:Any`, no variadic) — each bound value carries its SQLite affinity. On-doctrine; kept.
- **Thread-owned handles** (`ThreadOwnedCell`, zero-mutex) — the connection is a `Struct`, opened in the worker thread,
  never crossing a thread. The sink holds it in `:ephemeral`.

### Left behind (the vintage)
- **Panics on every fallible op** → replaced by values (§ Errors).
- **`deftest`** → `deftest'`.
- **`execute` returns `nil`** → returns rows-affected; **no row-returning query at all** → the `query` primitive is added.

## Two layers

1. **The raw `:wat::sqlite'` interop** — general sqlite primitives (open, ddl, write, **row-returning query**, tx,
   pragma). Backend-neutral SQL plumbing; usable by anything, not telemetry-specific.
2. **The `Store` satisfier** — `:wat::sqlite'::Db` supplies the `wat.query/Store` method impls (`ensure-schema`/`put`/
   `scan`/`scan-index`) *over* the raw interop, using native indexes + keyset pagination. This is what the sink holds.

## Layer 1 — the raw interop (`:wat::sqlite'`)

```clojure
;; the handles (thread-owned Structs holding a live rusqlite Connection) — names PROVISIONAL (intueri)
(wat.core/typealias :wat::sqlite'::Db          :rust::sqlite'::Db)          ;; read-write
(wat.core/typealias :wat::sqlite'::ReadHandle  :rust::sqlite'::ReadHandle)  ;; read-only (SQLITE_OPEN_READ_ONLY)

(wat.core/defenum :wat::sqlite'::Param :wat::enum::Pure       ;; a bound value with its SQLite affinity
  I64 [n <- :wat::core::i64]  F64 [x <- :wat::core::f64]  Str [s <- :wat::core::String]  Bool [b <- :wat::core::bool]
  Null [])                                                     ;; + Null (NULL-able projected columns)

;; open — fallible (bad path / permission / not-a-db) → a VALUE (Result), never a panic.
(:wat::core::defn :wat::sqlite'::open           [path <- :String] -> (:Result :wat::sqlite'::Db          :wat::sqlite'::Error) …)
(:wat::core::defn :wat::sqlite'::open-readonly  [path <- :String] -> (:Result :wat::sqlite'::ReadHandle  :wat::sqlite'::Error) …)

;; pragma / transaction control — set WAL + synchronous at open; begin/commit wrap a batch.
(:wat::core::defn :wat::sqlite'::pragma  [db <- :Db  name <- :String  value <- :String] -> (:Result :nil :Error) …)
(:wat::core::defn :wat::sqlite'::begin   [db <- :Db] -> (:Result :nil :Error) …)
(:wat::core::defn :wat::sqlite'::commit  [db <- :Db] -> (:Result :nil :Error) …)

;; execute — a write (INSERT/DELETE/UPDATE). Returns ROWS-AFFECTED (i64), not nil. Fallible → Result.
(:wat::core::defn :wat::sqlite'::execute
  [db <- :Db  sql <- :String  params <- :Vector<wat::sqlite'::Param>] -> (:Result :wat::core::i64 :Error) …)

;; execute-ddl — CREATE TABLE/INDEX (idempotent, IF NOT EXISTS). A malformed DDL is a PROGRAMMER error
;; (the schema is fixed code) → may panic-cascade; a runtime IO failure → Result. (see § Errors)
(:wat::core::defn :wat::sqlite'::execute-ddl [db <- :Db  ddl <- :String] -> (:Result :nil :Error) …)

;; query — THE NEW PRIMITIVE (wat-sqlite never had it): a parameterized SELECT that RETURNS ROWS.
;; a row is a Vector of Cell (the column values, in SELECT order); the result is a Vector of rows. Fallible → Result.
(:wat::core::defenum :wat::sqlite'::Cell :wat::enum::Pure
  I64 [n <- :i64]  F64 [x <- :f64]  Str [s <- :String]  Bool [b <- :bool]  Null [])
(:wat::core::defn :wat::sqlite'::query
  [rh <- :ReadHandle  sql <- :String  params <- :Vector<wat::sqlite'::Param>]
  -> (:Result :Vector<wat::core::Vector<wat::sqlite'::Cell>> :Error) …)
```

`Db` (rw) can also read; `ReadHandle` (ro) can only `query`. The `Store` satisfier uses `Db` for the write path and
`query` for the read path.

## Layer 2 — the `Store` satisfier (`:wat::sqlite'::Db` satisfies `wat.query/Store`)

The concrete SQL that satisfies the contract. **One `main` table; GSIs are native sqlite indexes on projected columns
— NOT separate tables** (that was a DynamoDB necessity; sqlite has native B-trees, so we use them — a private detail the
contract hides).

```sql
-- ensure-schema(table, [index…]) — idempotent. WAL + synchronous set at open, not here.
CREATE TABLE IF NOT EXISTS main (
  pk    TEXT NOT NULL,        -- partition key (namespace)
  sk    TEXT NOT NULL,        -- sort key (iso8601-nanos)
  data  TEXT NOT NULL,        -- the record's tagged EDN — OPAQUE (never parsed by sqlite)
  <ipk_col> TEXT,             -- one projected column per GSI's ipk (NULL-able)
  PRIMARY KEY (pk, sk)        -- the base access path is already a B-tree
);
CREATE INDEX IF NOT EXISTS idx_<name> ON main (<ipk_col>, sk);   -- the GSI: (ipk, isk=sk)

-- put(rows) — one transaction; a prepared INSERT reused per row. sk is unique nanos → pure INSERT.
BEGIN;
  INSERT INTO main (pk, sk, data, <ipk_col>…) VALUES (?, ?, ?, ?…);   -- projected ipk supplied by the consumer
  …                                                                    -- (INSERT OR REPLACE available if upsert wanted)
COMMIT;

-- scan(pk, sk-lo, sk-hi, limit, cursor) — KEYSET pagination; rides PK(pk,sk); O(log n) per page.
SELECT sk, data FROM main
 WHERE pk = ?1 AND sk >= ?2 AND sk <= ?3 AND (?cursor IS NULL OR sk > ?cursor)
 ORDER BY sk ASC LIMIT ?limit;
--  next-cursor = the last sk returned iff `limit` rows came back, else None.

-- scan-index(name, ipk, isk-lo, isk-hi, limit, cursor) — same, riding idx_<name>.
SELECT sk, data FROM main
 WHERE <ipk_col> = ?1 AND sk >= ?2 AND sk <= ?3 AND (?cursor IS NULL OR sk > ?cursor)
 ORDER BY sk ASC LIMIT ?limit;
```

- **Open pragmas** — `PRAGMA journal_mode=WAL` (concurrent read while writing; crash-safe) + `PRAGMA synchronous=NORMAL`
  (throughput without durability loss on WAL). Set once at `open`.
- **`data` stays `TEXT`** (tagged EDN), opaque — sqlite never reads it; the rete filter decodes it in the consumer.
- **Keyset, not offset** — the cursor is the last `sk`; the next page is `sk > cursor`. No `OFFSET` re-scan.

## Errors — the panics, four-questioned (panics come out on the way to core)

Every wat-sqlite panic is judged; the posture is **recoverable → a value; invariant → panic-cascade**. Errors are
**typed EDN records** (arc-296 errors-as-record); fallible ops return `(:wat::core::Result T :wat::sqlite'::Error)`.

| condition (was: panic) | Obvious? | Simple? | Honest? | Good-UX? | verdict |
|---|---|---|---|---|---|
| `open` bad path / permission / not-a-db | y | y | **panic is a LIE** — a recoverable IO condition | caller must decide | **value** (`Err`) |
| `execute` constraint violation / disk full | y | y | **panic hides a data-dependent failure** | caller must decide | **value** (`Err`) |
| `query` runtime failure (locked, IO) | y | y | recoverable | caller retries/surfaces | **value** (`Err`) |
| `execute-ddl` malformed DDL | y | y | the schema is FIXED CODE — a bug, not runtime data | fail loud, fail now | **panic-cascade** (invariant) |
| a bound the caller already guaranteed (e.g. param arity) | y | y | the caller promised it | — | **panic-cascade** (invariant) |

So `:wat::sqlite'::Error` is a small errors-as-record enum (`OpenFailed` / `WriteFailed` / `QueryFailed`, each carrying
the sqlite message + context as EDN). The `Store` contract's fallible ops thread it: `ensure-schema`/`put`/`scan`/
`scan-index` return `(:Result … :wat::sqlite'::Error)` — surfaced to the telemetry service as a value it handles, never a
process-killing panic on a bad disk.

## Home + build

- **Home:** the core substrate (`wat/`), adjacent to the rete tooling. The Rust bindings (`:rust::sqlite'::*`) are
  promoted into the main crate's `src/` **at standard** (not copied from the battery); the wat surface (`:wat::sqlite'`)
  is a baked core `.wat`. The `wat-sqlite` + `wat-telemetry-sqlite` battery crates stay where they are, untouched, as
  bridges.
- **Tests:** `deftest'` throughout (the vintage tell that flagged the old crate).
- **The strike order** (later): (a) the raw interop `:wat::sqlite'` (open rw/ro + pragma + tx + execute + **query**) with
  a `deftest'` gate; (b) the `Store` satisfier (`ensure-schema`/`put`/`scan`/`scan-index` SQL) with a `deftest'` gate
  proving a round-trip (put a batch → scan a page → keyset-paginate → scan a GSI); then the telemetry sink holds a
  `Store` and never names sqlite.

## Open (for the strike)

- **Names** (intueri): `:wat::sqlite'` vs the sink's expectation; `Db`/`ReadHandle`, `Cell`, `Error` + its variants,
  `query`/`execute`/`scan`/`scan-index`.
- **The `Error` shape** — settled jointly with the contract's error channel (`DESIGN-store-contract.md § Semantics`).
- **Promotion mechanics** — how `:rust::sqlite'` bindings register in core beside the still-loaded `wat-sqlite` battery
  (the primed namespace guarantees no collision; verify the `:rust::` interop registry accepts the primed segment).
