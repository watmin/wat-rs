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

## The SQL (grounded — `DESIGN-sqlite-core.md:103–138`, verbatim contract)

- **ensure-schema:** `CREATE TABLE IF NOT EXISTS main (pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL,
  <per-GSI ipk/isk cols NULL-able>, PRIMARY KEY (pk,sk))` + `CREATE INDEX IF NOT EXISTS idx_<name> ON main
  (<ipk_col>,<isk_col>)` per GSI. Pragmas (`journal_mode=WAL`, `synchronous=NORMAL`) at `open`, not here.
- **put:** `BEGIN; INSERT INTO main (pk,sk,data,<gsi cols…>) VALUES (?,?,?,…) [reused per row]; COMMIT`. Keys/data
  are opaque EDN strings the consumer built; sqlite never parses them.
- **scan:** `SELECT pk,sk,data FROM main WHERE pk=?1 AND sk>=?2 AND sk<=?3 AND (?cursor IS NULL OR sk>?cursor)
  ORDER BY sk ASC LIMIT ?limit`. `next-cursor` = last `sk` iff `limit` rows returned, else `None`.
- **scan-index:** the same on `idx_<name>`'s `(<ipk_col>,<isk_col>)`; `next-cursor` = last `isk`.
- Keyset (not offset); `data` stays opaque `TEXT`.

## THE ONE CONTRACT DECISION — `IndexSchema` needs a `name` (sqlite is its first consumer, `ALIVS ARGVIT`)

`StoredRow.index-keys` is a **name-keyed** `HashMap<String, IndexKey>` and `scan-index` takes an `index` **name** —
but `IndexSchema{pk,sk,ipk,isk}` carries **no name**. MemStore never hit this (it ignores `IndexSchema` — the
name-keyed HashMap is self-describing). sqlite is the first real consumer, and it *can't* build `idx_<name>` on the
right per-GSI columns, nor project `put`'s named index-keys into the right columns, without linking name → schema.

**Recommended fix (the honest completion): add `name <- :wat::core::String` to `:wat::query::IndexSchema`.** Then:
`ensure-schema` creates, per `IndexSchema`, columns named by the schema (`ipk`/`isk` become the column names,
qualified per-GSI, e.g. `<name>_ipk`/`<name>_isk`) + `idx_<name>`; `put` projects each row's `index-keys[name]` →
that GSI's columns; `scan-index(index=name)` selects on them. This is a small, ratified-S0-touching change to
`wat/query.wat` — but it's the *correct* model (a GSI schema that can't name its GSI is incomplete), and it's needed
before the telemetry consumer (multi-GSI). **Surface to the builder before the strike.**

- Alternative (rejected): scope S2 to a **single** GSI (the one unambiguous case, matching the S-mem gate) and defer
  `name` — a seam (sqlite less capable than MemStore's N-GSI HashMap; the differential couldn't test multi-GSI). Only
  take this if the builder wants to defer the contract change.

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
