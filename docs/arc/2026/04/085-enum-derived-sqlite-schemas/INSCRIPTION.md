# Arc 085 — enum-derived sqlite schemas (`Sqlite/auto-spawn`) — INSCRIPTION

**Status:** shipped 2026-04-28. Same-session follow-on to arcs 083 + 084.

The recognition was the user's:

> "Level 3.... that form is so incredibly small... how does this
> work?... if level 3 works - holy shit that's wild."

Level 3 works. The trader's existing `LogEntry` enum decl IS the
schema. The substrate now reads the decl at startup, derives the
CREATE TABLE per variant, the INSERT per variant, and the per-entry
binder — all from the single source of truth. Consumer-facing
substrate work to migrate a domain off ad-hoc SQL onto the auto
sink: 5 lines of wat (a typealias + a thin spawn wrapper).

Three durables shipped:

1. **`SymbolTable.types: Option<Arc<TypeEnv>>`** — capability
   carrier for shims that need to reflect on declared types.
   Slots in alongside the existing `encoding_ctx` /
   `source_loader` / `macro_registry` / sigma-fns. Per memory
   `feedback_capability_carrier` ("attach to SymbolTable next
   to encoding_ctx"). Populated at freeze time from the same
   `TypeEnv` that flows into `FrozenWorld`.

2. **`:wat::std::telemetry::Sqlite/auto-spawn<E,G>`** — the wat
   factory. Sibling to arc 083's explicit `Sqlite/spawn`; same
   `Service::Spawn<E>` return shape; takes the entry enum's name
   as a `:wat::core::keyword` value (no angle-bracket generics
   at the call site). Composes via three Rust shims registered
   manually under `:rust::sqlite::*` (no `#[wat_dispatch]` macro
   — the macro doesn't expose runtime context to user methods,
   and these shims need direct `sym.types` access).

3. **The auto-derived dispatch path.** Rust shim cache (`SCHEMAS`
   keyed by enum-keyword path) populated at startup, read in the
   worker. Variant-name → `(insert_sql, field_types)`; per entry,
   walks `Value::Enum.fields` parallel to `field_types`, builds
   the `Box<dyn ToSql>` vec, calls `Connection::prepare_cached`
   + `execute`. Same panic-vs-Option discipline as arcs 083/084.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped

### Slice 1 — `SymbolTable.types` capability carrier

`src/runtime.rs`:
- New `pub types: Option<Arc<TypeEnv>>` field on `SymbolTable`.
- New `set_types(Arc<TypeEnv>)` setter + `types() -> Option<&Arc<TypeEnv>>`
  borrow.
- Debug impl extended.

`src/freeze.rs`:
- `FrozenWorld::freeze` calls `symbols.set_types(Arc::new(types.clone()))`
  alongside the existing capability installs.

Workspace tests stay green; no behavioral change for any existing
shim (carrier is invisible until a shim asks for it).

### Slice 2 — auto-spawn factory + tests

`crates/wat-sqlite/src/auto.rs` (new module, ~340 lines):
- `AutoSchema` struct holding `HashMap<variant_name, AutoVariant>` +
  `ordered_ddls`.
- `AutoVariant` struct holding `insert_sql` + `field_types`.
- Process-wide `SCHEMAS: OnceLock<RwLock<HashMap<String, Arc<AutoSchema>>>>`
  cache keyed by enum keyword path.
- Three hand-registered shims:
  - `:rust::sqlite::auto-prep enum-name` — caller-side; reads
    `sym.types`, walks the EnumDef, derives + caches.
  - `:rust::sqlite::auto-install-schemas db enum-name` — worker-
    side; pulls cached DDLs, runs each via `WatSqliteDb::execute_ddl`.
  - `:rust::sqlite::auto-dispatch db enum-name entry` — worker-side;
    looks up variant, binds fields, calls `prepare_cached + execute`.
- Naming conventions (slice 1):
  - PascalCase variant → snake_case table (`Buy` → `buy`,
    `PaperResolved` → `paper_resolved`).
  - Kebab-case field → snake_case column.
  - Type → SQLite affinity: `:String` → TEXT NOT NULL, `:i64` →
    INTEGER NOT NULL, `:f64` → REAL NOT NULL, `:bool` → INTEGER
    NOT NULL.
  - Anything else panics at `auto-prep` with a diagnostic.

`crates/wat-sqlite/wat/std/telemetry/Sqlite.wat`:
- New `:wat::std::telemetry::Sqlite::auto-empty-translator<E>` —
  no-op stats translator (auto-spawn uses null cadence; translator
  is never invoked at runtime).
- New `:wat::std::telemetry::Sqlite/auto-spawn<E,G>` — composes
  the three Rust shims into a substrate `Sqlite/spawn` call.
  The closures (`schema-install` and `dispatcher`) capture the
  enum-name keyword and cross thread boundaries cleanly.

`crates/wat-sqlite/src/lib.rs`:
- `mod auto;` declared.
- `register()` calls `auto::register(builder)` after the macro-
  generated `WatSqliteDb` registration.
- `WatSqliteDb.conn` made `pub(crate)` so the `auto` module can
  reach the underlying rusqlite Connection for `prepare_cached`.

### Slice 2 — Tests

`crates/wat-sqlite/wat-tests/std/telemetry/auto-spawn.wat`:
- Tiny throwaway `:test::Event` enum with two Tagged variants
  exercising all four scalar types:
  - `Buy (price :f64) (qty :i64)`
  - `Sell (price :f64) (qty :i64) (reason :String) (forced :bool)`
- `test-event-roundtrip` deftest spawns the auto sink, sends one
  of each variant, drops, joins. End-to-end roundtrip verified
  out-of-band:
  ```
  $ sqlite3 /tmp/wat-sqlite-test-auto-001.db 'SELECT *, typeof(price), typeof(qty) FROM buy'
  100.5|7|real|integer
  $ sqlite3 /tmp/wat-sqlite-test-auto-001.db 'SELECT *, typeof(price), typeof(qty), typeof(reason), typeof(forced) FROM sell'
  102.25|3|stop-loss|1|real|integer|text|integer
  ```
  The substrate derived the schemas, derived the inserts, routed
  each entry by variant, bound each field with the correct SQLite
  affinity. Bool round-trips as integer per rusqlite/SQLite
  convention.

6 of 6 wat-sqlite tests green; workspace 728 substrate Rust tests +
every other wat-suite stays clean.

### Slice 3 — INSCRIPTION (this file)

---

## What's still uncovered

- **Unit variants emit nothing.** A `(MyEnum (FlagOnly))` variant
  has no fields → no table → dispatching it panics. Future arc
  adds an event-style table (single timestamp_ns column) when a
  consumer surfaces a need.
- **`:Option<T>` fields not yet supported.** Today's auto-prep
  panics if a Tagged variant declares an Option-typed field. The
  semantic is clear (NULLABLE column; `Param::Null` binding), it
  just needs the Param enum extended (arc 084 deferred Null) and
  the value mapper handling `Value::Option`.
- **Table-name overrides.** PascalCase → snake_case is automatic
  and unconditional. A consumer with an existing table named
  outside the convention (e.g., the lab's `paper_resolutions`,
  plural) accepts the auto-derived name (`paper_resolved`) or
  uses the explicit `Sqlite/spawn`. Future arc adds an annotation
  syntax on the enum decl.
- **Schema migrations.** First run creates tables (`IF NOT
  EXISTS`); subsequent runs see existing tables and write to
  them. Adding a column or renaming requires manual sqlite3 CLI
  intervention. No migration framework. Same posture as
  rusqlite's basic primitives.
- **Self-heartbeat.** Auto-spawn forces null cadence. Substrate
  Stats rows (`batches` / `entries` / `max-batch-size`) are not
  emitted into the auto-derived sink — there's no obvious place
  to put them without another conventionally-named table or a
  consumer-defined heartbeat variant. Consumers who want self-
  telemetry use the explicit `Sqlite/spawn` with a stats-translator.

## Consumer impact

Unblocks:
- **Arc 083 slice 3** (lab migration). The lab's existing
  `LogEntry` enum (`PaperResolved` + `Telemetry`) becomes the
  source of truth. The lab's `Sqlite.wat` collapses to a
  typealias + a 4-line spawn wrapper. The dispatcher,
  schema-install, maker, translate-stats, RunDb shim, and
  RunDbService all delete. The lab tests' table-name
  expectations update from `paper_resolutions` /
  `telemetry` to the auto-derived `paper_resolved` /
  `telemetry`.
- **Cross-domain consumers** (MTG, truth-engine, future
  experiments) get the same UX for free: declare an enum,
  call `Sqlite/auto-spawn`, ship.

The substrate now treats "the consumer's enum decl" as a
first-class compile target. This is what wat is for.

PERSEVERARE.
