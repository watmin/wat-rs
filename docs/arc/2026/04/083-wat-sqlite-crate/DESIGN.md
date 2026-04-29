# Arc 083 — `crates/wat-sqlite/` substrate crate

**Status:** PROPOSED 2026-04-29. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 029 — `:trading::rundb::Service` shipped lab-side as a CSP wrapper over rusqlite. Lab's `WatRunDb` shim provided the typed row-write methods (`log_paper_resolved`, `log_telemetry`).
- Arc 080 — substrate `:wat::std::telemetry::Service<E,G>` shipped as the generic queue-fronted shell.
- Arc 081 — substrate `:wat::std::telemetry::Console` shipped as a render-and-print dispatcher factory companion.
- Lab 059-002 sub-slice B — lab `:trading::telemetry::Sqlite/spawn` shipped as a thin worker that opens RunDb in its thread + calls substrate's `Service/loop`. Works end-to-end.

**Surfaced by:** The user's recognition (2026-04-29):

> "i think we should have a wat-rs/crate/wat-sqlite/ who does all the stuff RunDbService does - just with a better name... it can have a companion to the console reporter as well - yea?.."

The lab's `:trading::telemetry::Sqlite/spawn` is generic machinery. Nothing trader-specific in the WORKER's loop (open Db, install schemas via caller's fn, build dispatcher closure, run substrate `Service/loop`). The schema install + dispatcher are caller-provided already. Lifting the worker scaffold to substrate gives MTG / truth-engine / any future consumer a sqlite-backed telemetry destination for free — same way arc 081 gave them a console destination for free.

---

## What this arc is, and is not

**Is:**
- A new crate at `crates/wat-sqlite/` (sibling to `wat-edn`, `wat-lru`, `wat-holon-lru`).
- Substrate-level sqlite primitives at `:wat::sqlite::*` — `Db` type (thread-owned), `open`, `execute-ddl`, `execute` (parameterized statement).
- Substrate-level telemetry destination at `:wat::std::telemetry::Sqlite/spawn<E,G>` — the lifted version of lab's current `Sqlite/spawn`. Companion to `:wat::std::telemetry::Console/dispatcher`.
- The lab keeps a thin wrapper at `:trading::telemetry::Sqlite/spawn` that pre-supplies the trader-specific init-fn (calls `:trading::log::all-schemas` + `:trading::telemetry::dispatch`).

**Is not:**
- A retroactive rename of `:trading::rundb::*`. The lab's wat-side `:trading::rundb::log-paper-resolved` / `log-telemetry` stay as wat-level helpers that wrap `:wat::sqlite::execute` with the lab's domain-specific SQL. Could retire later if a consumer surfaces a need.
- A schema engine. Substrate ships `execute-ddl`; consumers run their own `CREATE TABLE` strings.
- A query API. Substrate provides write-side primitives only. Read-side stays consumer-side (and currently happens out-of-band via sqlite3 CLI, per the existing rundb pattern).
- A connection pool. One worker, one Db, thread-owned. Multi-writer is not a goal.

---

## Surface

### Low-level Db (substrate)

```scheme
;; Opaque Rust shim — thread-owned via ThreadOwnedCell. Cannot
;; cross thread boundaries; the worker that opens it must be the
;; one that uses it.
(:wat::core::typealias :wat::sqlite::Db
  :rust::wat::sqlite::Db)

;; Open or create a sqlite file. Panics on permissions / disk
;; errors per substrate's panic-vs-Option discipline (memory:
;; feedback_shim_panic_vs_option — construction panics; lookup
;; returns Option). Returns an opaque Db value.
(:wat::sqlite::open
  (path :String)
  -> :wat::sqlite::Db)

;; Execute a parameterless statement (CREATE TABLE, etc).
;; Panics on syntax errors; succeeds even when the statement is
;; idempotent (CREATE TABLE IF NOT EXISTS).
(:wat::sqlite::execute-ddl
  (db :wat::sqlite::Db)
  (ddl :String)
  -> :())

;; Execute a parameterized statement. params is a Vec<Value>;
;; supported value types are i64, f64, String, bool — same set
;; rusqlite's ToSql trait covers in slice 1.
(:wat::sqlite::execute
  (db :wat::sqlite::Db)
  (sql :String)
  (params :Vec<wat::sqlite::Param>)
  -> :())

;; A wrapper enum so heterogeneous param types fit in a Vec.
;; Caller picks the variant per param.
(:wat::core::enum :wat::sqlite::Param
  (I64 :i64)
  (F64 :f64)
  (Str :String)
  (Bool :bool))
```

### Telemetry destination (substrate, companion to Console)

```scheme
;; Like Console/dispatcher (arc 081): a factory that takes
;; everything the worker needs, returns a Spawn ready for
;; :user::main to wire.
(:wat::std::telemetry::Sqlite/spawn<E,G>
  (path :String)
  (count :i64)
  ;; init-fn runs once inside the worker thread. Receives the
  ;; freshly opened Db; returns the per-entry dispatcher closure.
  ;; The init body is where the consumer installs schemas.
  (init-fn :fn(wat::sqlite::Db)->fn(E)->())
  (stats-translator :fn(:wat::std::telemetry::Service::Stats)->Vec<E>)
  (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
  -> :wat::std::telemetry::Service::Spawn<E>)
```

The init-fn is the seam between substrate (knows about Db lifecycle) and consumer (knows about schemas + entry dispatch). One hook covers both setup AND per-entry dispatching.

### Lab thin wrapper (lab-side, post-arc-083)

```scheme
;; In wat/io/telemetry/Sqlite.wat — replaces the lab's current
;; loop-entry + Sqlite/spawn pair. Pre-supplies the trader's init.
(:wat::core::define
  (:trading::telemetry::Sqlite/spawn<G>
    (path :String)
    (count :i64)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    -> :wat::std::telemetry::Service::Spawn<trading::log::LogEntry>)
  (:wat::std::telemetry::Sqlite/spawn path count
    :trading::telemetry::sqlite-init
    :trading::telemetry::translate-stats-via-default-maker
    cadence))

(:wat::core::define
  (:trading::telemetry::sqlite-init
    (db :wat::sqlite::Db)
    -> :fn(trading::log::LogEntry)->())
  (:wat::core::let*
    (((_install :())
      (:wat::core::foldl (:trading::log::all-schemas) ()
        (:wat::core::lambda ((acc :()) (ddl :String) -> :())
          (:wat::sqlite::execute-ddl db ddl)))))
    (:wat::core::lambda ((entry :trading::log::LogEntry) -> :())
      (:trading::telemetry::dispatch db entry))))
```

---

## Slice plan

### Slice 0 — Crate scaffold

`crates/wat-sqlite/`:
- `Cargo.toml` declaring rusqlite dep.
- `src/lib.rs` empty registrar (no shims yet).
- `tests/test.rs` empty test stub.
- Placeholder `wat/sqlite/Db.wat` and `wat/std/telemetry/Sqlite.wat`.

Workspace root `Cargo.toml`:
- Add `crates/wat-sqlite` to `members` + `default-members`.

Verify `cargo build --workspace` clean. No tests to run (no surfaces yet).

### Slice 1 — Db type + open + execute-ddl + execute

`src/lib.rs` (or `src/shim.rs`): WatSqliteDb struct + `#[wat_dispatch]` macros for `open` / `execute-ddl` / `execute`. Param enum.

`wat/sqlite/Db.wat`: typealiases + thin wat surface.

Tests at `wat-tests/sqlite/Db.wat`:
- Open a temp file → execute-ddl CREATE TABLE → execute INSERT with params → no crash.
- Verify db file exists on disk after worker drops.

### Slice 2 — `:wat::std::telemetry::Sqlite/spawn`

`wat/std/telemetry/Sqlite.wat`: the spawn fn that orchestrates worker entry + Service/loop.

Tests at `wat-tests/std/telemetry/Sqlite.wat`:
- spawn + drop + join (lifecycle, no traffic)
- spawn + send mixed-batch + drop + join (with init-fn that creates a single events table + dispatcher that INSERTs each entry)
- spawn + counter-cadence + verify heartbeat rows land via stats-translator

### Slice 3 — Lab migration

Lab repo:
- Update `wat/io/telemetry/Sqlite.wat` — delegate to substrate's Sqlite/spawn.
- Lab keeps `:trading::rundb::log-paper-resolved` / `log-telemetry` (the typed Rust shims) for now. The `:trading::telemetry::dispatch` calls these.
- All lab call sites of `:trading::rundb::Service` migrate to use `:trading::telemetry::Sqlite/spawn` per 059-002 sub-slices C–F.
- Old `:trading::rundb::Service` retrofit deletes per 059-002 sub-slice G.

Tests: existing wat-suite + proof_002/003/004 stay green.

### Slice 4 — INSCRIPTION + USER-GUIDE docs

`wat-rs/docs/arc/2026/04/083-wat-sqlite-crate/INSCRIPTION.md` documents what shipped.

`wat-rs/docs/USER-GUIDE.md` gains a "Sqlite-backed telemetry" section that walks the trader's example pattern (init-fn shape; schema install; dispatcher).

---

## Open questions

### Q1 — `:wat::sqlite::Param` enum vs polymorphic execute

The execute primitive needs to bind heterogeneous params. Options:
- (a) Param enum (proposed): `(:wat::sqlite::Param::Str s)`, `(I64 n)`, etc. Caller wraps each value; verbose but explicit.
- (b) Variadic execute: `(execute db "SELECT ?" v1 v2 v3)`. Cleaner; requires substrate's variadic-arg machinery (not currently available for Rust shims).

Default: (a). Verbose-but-honest. Rust's rusqlite uses the same shape (`params![]` macro) under the hood.

### Q2 — Where does sqlite read live?

Substrate ships only writes in slice 1. Reads (SELECT, prepared queries) ship as a follow-up arc when a consumer surfaces a need (proof tests do all reads via sqlite3 CLI today).

### Q3 — Should lab's `:trading::rundb::log-*` Rust shims retire?

These could be reimplemented as wat using `:wat::sqlite::execute` + SQL strings. Saves the lab's Rust shim code. Costs: prepared-statement reuse inside Rust (rusqlite caches; the wat path also caches per Statement). Probably negligible.

Default: defer the retirement. The shims work; rewriting them in wat is a separate slice.

### Q4 — Dispatch the substrate's tests through what entry type?

Substrate ships ZERO entry variants. The slice-2 tests need a trivial entry type — `:i64` again (matches arc 080's substrate-Service tests). The init-fn creates a tiny `events` table; the dispatcher INSERTs each i64 as a row.

---

## Test strategy

- Slice 0: cargo build clean. No tests.
- Slice 1: pure Rust shim tests + wat smoke tests for Db/open/execute-ddl/execute.
- Slice 2: substrate Sqlite/spawn end-to-end tests with a tiny test entry type. Verify rows land.
- Slice 3: existing lab tests stay green.
- Slice 4: docs only.

---

## Dependencies

**Upstream (must ship before this arc starts):**
- Arc 080 (substrate `Service<E,G>`) — REQUIRED. The Sqlite spawn delegates to its loop.

**Downstream (this arc unblocks):**
- 059-002 sub-slices C–H — lab call sites migrate to use the substrate Sqlite alongside the substrate Service.
- Cross-domain consumers (MTG, truth-engine) — drop in `:wat::std::telemetry::Sqlite/spawn` with their own init-fn + entry types.

**Parallel-safe with:** Arc 079 (wat-edn) and 081 (Console) — both already shipped.

PERSEVERARE.
