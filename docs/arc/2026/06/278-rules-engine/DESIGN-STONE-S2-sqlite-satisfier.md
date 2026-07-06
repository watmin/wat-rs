# DESIGN — S2: the sqlite `Store` satisfier, differential-tested vs the MemStore oracle

> The stone where **R21 `EXPLORATA CAEDE NON VINCIMVR` turns `PROBATVM`** — the sqlite driver held bit-for-bit to
> the S-mem MemStore oracle (`3304cbd5`). Builds on S1 (`7f69b78d`, the raw `:wat::sqlite'` interop, in core).

## Why / what

`:wat::sqlite'` gives raw SQL verbs (S1); `:wat::query::Store` is the backend-agnostic contract (S0). S2 bridges
them: a satisfier that implements `ensure-schema`/`put`/`scan`/`scan-index` as SQL over S1, so a consumer holding a
`Store` gets sqlite persistence without naming it — the *same* consumer code that drives MemStore. Proven by a
**differential**: the same op sequence through both backends must return the same `Page`s.

## The satisfier — a `defstruct` wrapping the Connection (the MemStore pattern)

A `Store` satisfier must be a `:holder :Struct` (impure — it holds a live resource; `DESIGN-store-contract.md:56`).
The `Connection` is a `:rust::` opaque (no Aggregate holder), so — exactly as MemStore is a `defstruct` wrapping its
peer — the sqlite satisfier is a `defstruct` wrapping the Connection:

```clojure
(:wat::core::defstruct :wat::sqlite'::SqliteStore [conn <- :wat::sqlite'::Connection])
(:wat::core::extend-type :wat::sqlite'::SqliteStore :wat::query::Store
  (ensure-schema [self table indexes] …)   ;; CREATE TABLE main + CREATE INDEX idx_<name> per GSI (execute-ddl)
  (put [self rows] …)                        ;; BEGIN; INSERT per row (main + projected GSI cols); COMMIT (begin/execute/commit)
  (scan [self q] …)                          ;; keyset SELECT on (pk,sk); build Page (select)
  (scan-index [self q] …))                   ;; keyset SELECT on idx_<name>'s (ipk,isk); build IndexPage (select)
(:wat::core::derive :wat::sqlite'::SqliteStore :wat::query::ReadStore)   ;; the read-only edge, like MemStore
```

The impl bodies are now **type-checked** (the extend-type-honesty strike `fa8bbcb9`) — they must be honest, returning
`Result<T, :wat::query::Error>` per the surface, lifting `:wat::sqlite'::Error` faults into `:wat::query::Error`
(both are recovery-axis enums — map `Transient→Transient`, `Constraint→Constraint`, `Fatal→Fatal`, re-wrapping the
`Fault`). 293.W: the `SqliteStore` is impure (holds the Connection) → `:ephemeral`-only, never durable/wire — the
containment the honesty floor gives for free.

## The SQL — DDB-faithful: GSIs are SECONDARY COMPLETE TABLES (the builder's slugdb model, ratified 2026-07-05)

> **Reversal (kept honest).** The earlier `DESIGN-sqlite-core.md` proposed a single `main` table with GSIs as native
> sqlite indexes on projected columns — a STORAGE optimization that broke the DDB fidelity that is this store's whole
> point. The builder's 5-yr-old `slugdb` (github.com/watmin/Ruby-slugdb-sqlite3) is the DDB-faithful model and it
> WINS: an index IS a complete table with the same `(partition, sort, item)` shape. The duplication is not waste — it
> is the DDB semantics (GSIs are projected copies), it keeps the schema uniform, it collapses `scan`/`scan-index`
> into ONE keyset primitive, a GSI read returns the full item with NO join, and — transactional — our indexes are
> ALWAYS in sync (strictly better than DDB's eventual consistency). `DESIGN-sqlite-core.md`'s single-table SQL is
> SUPERSEDED for S2.

- **ensure-schema:** the base + one complete table per named GSI:
  ```sql
  CREATE TABLE IF NOT EXISTS main   (pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(pk,sk));
  -- per GSI (named), a SEPARATE COMPLETE TABLE — the full item projected in + the base-key pointer:
  CREATE TABLE IF NOT EXISTS index_<name> (ipk TEXT NOT NULL, isk TEXT NOT NULL, pk TEXT NOT NULL, sk TEXT NOT NULL,
                                           data TEXT NOT NULL, PRIMARY KEY(ipk, isk, pk, sk));
  ```
  Pragmas (`journal_mode=WAL`, `synchronous=NORMAL`) at `open`. (An `indexes(name, schema)` metadata table like
  slugdb's is OPTIONAL — the wat-layer `SqliteStore` already knows its declared GSIs; add it only if a reflect/reopen
  path needs it — out of scope for S2.)
- **put:** one transaction; **clear-then-insert** per row — the upsert-safe shape (an upsert that CHANGES a row's
  index-keys must clear the OLD projections or leak stale GSI rows; this is *why* slugdb deletes-then-inserts —
  correctness, not simplification — and the index tables carry `(pk,sk)` so the clear needs no old-item read):
  ```sql
  BEGIN;
    DELETE FROM main        WHERE pk=? AND sk=?;                 -- clear the base row (upsert)
    DELETE FROM index_<name> WHERE pk=? AND sk=?;                -- clear ALL old projections of this base row (per GSI)
    INSERT INTO main (pk, sk, data) VALUES (?, ?, ?);
    INSERT INTO index_<name> (ipk, isk, pk, sk, data) VALUES (?, ?, ?, ?, ?);   -- per GSI in the row's index-keys
  COMMIT;
  ```
  `index-keys[name] = IndexKey{ipk,isk}` supplies the projected keys; `data` is the same opaque EDN, copied in. A row
  that projects no key for a given GSI simply gets no `index_<name>` row (a sparse GSI — DDB-faithful).
- **scan** and **scan-index** are the SAME keyset primitive — a range-scan on a table by its (partition, sort):
  ```sql
  -- scan(pk, sk-lo, sk-hi, limit, cursor) → on main, keyed (pk, sk):
  SELECT pk, sk, data FROM main
   WHERE pk=?1 AND sk>=?2 AND sk<=?3 AND (?cursor IS NULL OR sk>?cursor) ORDER BY sk ASC LIMIT ?limit;
  -- scan-index(name, ipk, isk-lo, isk-hi, limit, cursor) → on index_<name>, keyed (ipk, isk); returns pk,sk too:
  SELECT ipk, isk, pk, sk, data FROM index_<name>
   WHERE ipk=?1 AND isk>=?2 AND isk<=?3 AND (?cursor IS NULL OR isk>?cursor) ORDER BY isk ASC LIMIT ?limit;
  ```
  `next-cursor` = last `sk` (scan) / last `isk` (scan-index) iff `limit` rows returned, else `None`. Keyset, not
  offset; `data` stays opaque `TEXT`. **The two share one query-builder** (table + partition-col + sort-col + the
  projected-select-columns), parameterized — that's the uniformity slugdb buys.

## `IndexSchema` needs a `name` — CONFIRMED (the model makes it first-class, not a bolt-on)

`StoredRow.index-keys` is name-keyed and `scan-index` takes an index **name**; in the secondary-complete-tables model
the name **is the table name** (`index_<name>`). So `IndexSchema` gains `name <- :wat::core::String` — `ensure-schema`
creates `index_<name>` per `IndexSchema`, `put` routes `index-keys[name]` into `index_<name>`, `scan-index(name)`
selects from it. This is the correct model (slugdb's `indexes(name, schema)` makes the name first-class), needed
before multi-GSI. A small, ratified-S0-touching change to `wat/query.wat` + the S-mem gate's positional `IndexSchema`
construction. (`IndexSchema.pk`/`sk` — the base-key attr *names* — are redundant with `main`'s fixed `pk`/`sk` columns
in the opaque-string model; keep them for the metadata/reflection future or drop them — a minor call for the strike.)

## The differential — the whole point (`we do not lose`)

One wat fn drives BOTH backends through the **surface** type, so the *same code* exercises each:

```clojure
(:wat::core::defn :probe::run-ops [store <- :wat::query::Store] -> …   ;; ensure-schema -> put -> scan pages -> scan-index
  …returns the collected Pages…)
;; the gate (a deftest', mirroring S-mem.gate): build a MemStore AND a SqliteStore (:memory:), run run-ops on EACH,
;; assert the returned Pages are EQUAL (rows + next-cursors) — same ops -> same Pages, bit-for-bit.
```

The op sequence mirrors S-mem.gate (5 rows on one pk, 2 projecting a GSI; keyset-paginate 2/2/1; scan-index). MemStore
is the oracle; sqlite must match it. A divergence is a sqlite-satisfier bug (or a genuine contract ambiguity the
differential surfaces — e.g. sort-order of equal keys, cursor-boundary semantics).

## Out of scope (named, not deferred)

Multi-GSI *if* the builder defers `IndexSchema.name` (then single-GSI, tracked). Connection pooling. `OR REPLACE`
upsert semantics (pure INSERT unless a consumer needs upsert — name it then). Blob `Cell`. The telemetry key-schema
(consumer design). Prepared-statement caching (perf stone if measured).

## Build order

1. **(if ratified) `IndexSchema` + `name`** — the contract change in `wat/query.wat` (+ update the S-mem gate's
   `IndexSchema` construction + mem.wat if it references the field positionally).
2. **the satisfier** — `wat/query/sqlite_store.wat` (or fold into `wat/sqlite.wat`): the `SqliteStore` defstruct +
   extend-type impls (SQL string-building over S1's verbs; `:wat::sqlite'::Error` → `:wat::query::Error` lift).
3. **the differential gate** — `tests/rete/probe_arc278_sqlite_store_differential.{wat,rs}`: `run-ops` over both, Pages
   asserted equal.

## Blast radius

`wat/query.wat` (+`name` on IndexSchema, if ratified) · the new satisfier source + its `src/stdlib.rs` bake · the
differential gate · possibly the S-mem gate's IndexSchema construction. No change to S1, the `Store` methods'
signatures, or MemStore's logic.
