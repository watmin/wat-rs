# BRIEF — S2: the sqlite `Store` satisfier, differential-tested vs the MemStore oracle

> **Executor: one sonnet SHADOWDANCER.** The orchestrator scouted the lair, ratified the model (the builder's
> DDB-faithful secondary-complete-tables), and PROVED the composition (a disconfirming probe: the secondary-tables
> model round-trips through S1's verbs — `main=2 idx=1 first-data={:v 1}`). Work ONLY in
> `/home/watmin/work/holon/wat-rs/` (`pwd` first; anchor git; `.claude/worktrees/` illegal). `cargo wat <f>` to
> dogfood; `cargo nextest run --release` (NEVER `cargo test`); `cargo build` to check. **Commit NOTHING.**

## The work

Bridge S1 (raw `:wat::sqlite'`) to S0 (`:wat::query::Store`): a **`SqliteStore` satisfier** implementing
`ensure-schema`/`put`/`scan`/`scan-index` as SQL over S1, the **DDB-faithful way** — a GSI is a **separate complete
table** `index_<name>(ipk,isk,pk,sk,data)`. Then prove it with a **differential** against the S-mem MemStore oracle:
the *same ops* through both backends return the *same Pages*. This is the stone where R21 turns `PROBATVM`. The full
grounded design is `DESIGN-STONE-S2-sqlite-satisfier.md` — **read it first.**

## The model (ratified — the builder's slugdb, DDB-faithful; do NOT revert to single-table)

A GSI is a table with 4 keys instead of 2, both named directly. `main(pk,sk,data,PK(pk,sk))`; per named GSI a
**separate complete table** `index_<name>(ipk,isk,pk,sk,data,PK(ipk,isk,pk,sk))` with the full item projected in.
`scan` and `scan-index` are the SAME keyset primitive on `(table, partition-col, sort-col)`. `put` is
**clear-then-insert** (upsert-safe: DELETE the base + all its index projections by `(pk,sk)`, then INSERT). The SQL is
in the design § "The SQL"; follow it verbatim.

## The PROVEN composition (copy this shape — it type-checks + runs)

```wat
;; open :memory: -> execute-ddl (secondary tables) -> execute (INSERT, Params) -> select (keyset) -> unpack Cells
(:wat::sqlite'::open ":memory:")                                   ;; -> Result<Connection,Error>
(:wat::sqlite'::execute-ddl conn "CREATE TABLE main (pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(pk,sk))")
(:wat::sqlite'::execute conn "INSERT INTO main (pk,sk,data) VALUES (?,?,?)"
  (:wat::core::Vector :wat::sqlite'::Param (:wat::sqlite'::Param::Str "u#1") (:wat::sqlite'::Param::Str "a") (:wat::sqlite'::Param::Str "{:v 1}")))
(:wat::sqlite'::select conn "SELECT pk,sk,data FROM main WHERE pk=? AND sk>=? AND sk<=? ORDER BY sk ASC LIMIT ?"
  (:wat::core::Vector :wat::sqlite'::Param (:wat::sqlite'::Param::Str "u#1") (:wat::sqlite'::Param::Str "a") (:wat::sqlite'::Param::Str "z") (:wat::sqlite'::Param::I64 2)))
;; select -> Result<Vector<Vector<Cell>>,Error>; each row is a Vector<Cell> in SELECT order; unpack via match:
(:wat::core::match cell -> :wat::core::String
  ((:wat::sqlite'::Cell::Str s) s) ((:wat::sqlite'::Cell::I64 _) "…") ((:wat::sqlite'::Cell::F64 _) "…") ((:wat::sqlite'::Cell::Nil) "…"))
```

## Read the rooms, in order

1. **`DESIGN-STONE-S2-sqlite-satisfier.md`** — the model, the SQL (verbatim), the differential, out-of-scope.
2. **`wat/query/mem.wat`** — MemStore: the satisfier PATTERN to mirror (a `defstruct` wrapping the resource,
   `extend-type`'d to `:wat::query::Store`, `derive`'d to `ReadStore`), AND the ORACLE semantics your Pages must match
   (its scan/scan-index filter+sort+take+cursor logic — replicate it in SQL).
3. **`wat/sqlite.wat`** — the S1 verbs you build on (`open`/`execute-ddl`/`execute`/`select`, `Param`/`Cell`, the
   `:wat::sqlite'::Error` enum). All fallible verbs return `Result<T, :wat::sqlite'::Error>`.
4. **`wat/query.wat`** — the `Store`/`ReadStore` surfaces + records (`StoredRow`/`Row`/`IndexRow`/`ScanRequest`/
   `IndexScanRequest`/`Page`/`IndexPage`/`TableSchema`/`IndexSchema`/`IndexKey`) + the `:wat::query::Error` enum.
5. **`tests/rete/probe_arc278_smem_roundtrip.{wat,rs}`** — the gate + differential shape to mirror.

## Build order

1. **`IndexSchema` + `name`** (the contract change): add `name <- :wat::core::String` as the FIRST field of
   `:wat::query::IndexSchema` in `wat/query.wat`. Update every positional construction: `mem.wat` (if it references
   IndexSchema) + the S-mem gate `tests/rete/probe_arc278_smem_roundtrip.wat`'s `(:wat::query::IndexSchema …)` (add
   the name, e.g. `"by-v"`). Rebuild; S-mem gate green.
2. **The `SqliteStore` satisfier** — a new baked source `wat/query/sqlite_store.wat` (baked after `wat/sqlite.wat`):
   `(:wat::core::defstruct :wat::sqlite'::SqliteStore [conn <- :wat::sqlite'::Connection])`, `extend-type` to
   `:wat::query::Store` (the 4 impls — SQL over S1's verbs, the secondary-tables model, clear-then-insert `put`, the
   keyset primitive, `Cell`→`Row`/`IndexRow` unpacking, `next-cursor`), `derive` to `:wat::query::ReadStore`. Lift
   `:wat::sqlite'::Error` → `:wat::query::Error` (both recovery-axis: `Transient→Transient`, `Constraint→Constraint`,
   `Fatal→Fatal`, re-wrapping the `Fault`'s fields). A `SqliteStore/open`-style constructor helper is fine (returns a
   `SqliteStore` over an opened `:memory:` or path connection).
3. **The differential gate** — `tests/rete/probe_arc278_sqlite_store_differential.{wat,rs}` (mirror the S-mem gate):
   a `(:wat::core::defn :probe::run-ops [store <- :wat::query::Store] -> …)` that runs the FULL op sequence
   (ensure-schema with one GSI → put 5 rows [2 projecting the GSI] → scan keyset 2/2/1 → scan-index) and returns the
   collected Pages; a `deftest'` that builds a MemStore AND a SqliteStore (`:memory:`), runs `run-ops` on EACH, and
   `assert-eq`s the Pages EQUAL (rows + next-cursors). Same ops → same Pages.

## STOP triggers (halt + report)
1. **STOP-HONEST:** the `extend-type` impl bodies are TYPE-CHECKED now (the honesty strike) — they must genuinely
   return `Result<T, :wat::query::Error>`. Do NOT loosen types to pass; if an impl can't be made honest, STOP.
2. **STOP-DIFFERENTIAL:** the gate must compare the two backends' Pages to EACH OTHER (MemStore == SqliteStore), not
   just assert each is independently non-empty. The differential IS the deliverable. If they DIVERGE, STOP and report
   the exact rows/cursors — it's a real SqliteStore bug (or a genuine contract ambiguity), not a test to fudge.
3. **STOP-MODEL:** secondary complete tables (`index_<name>`), NOT single-table-with-native-indexes. `put` is
   clear-then-insert (upsert-safe). Do not revert.
4. **STOP-NOCP:** do NOT modify S1 (`wat/sqlite.wat`, `src/rust_deps/sqlite.rs`), MemStore's logic, or the `Store`
   method signatures. `IndexSchema` + `name` is the only contract change.
5. **STOP-FORM:** a nullary variant pattern in `match` is parenthesized — `((:wat::sqlite'::Cell::Nil) body)`, not
   bare. `first` returns the element (not Option). Empty typed `PersistentVector` is bare `(:wat::core::PersistentVector)`.

## The gate (EXPECTATIONS)
| what | command | expected |
|---|---|---|
| the differential passes (MemStore == SqliteStore) | `cargo nextest run --release -E 'test(sqlite_store_differential)'` | passed |
| S-mem + S0 + S1 still green | `cargo nextest run --release -E 'test(smem_roundtrip) or test(query_contract) or test(sqlite_interop)'` | passed |
| whole floor | `cargo nextest run --release` | Summary line VERBATIM; 0 failed modulo the known `no_inlined_wat` reminder |

## Final report: files changed · the SqliteStore's 4 impls (the SQL + the Error lift) · the differential gate + how it compares Pages · the verbatim `sqlite_store_differential` + whole-floor Summary · any divergence found · STOP triggers hit or "none".

## Prior comparable: S-mem.gate (`3304cbd5`) + S1 (`7f69b78d`). The proven composition probe is
`scratchpad/s2-secondary-tables-probe.wat` (the exact S1-verb shape that type-checks + runs).
