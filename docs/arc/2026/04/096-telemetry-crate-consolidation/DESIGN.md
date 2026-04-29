# Arc 096 — telemetry crate consolidation — DESIGN

**Status:** in design 2026-04-29.

The substrate currently houses telemetry plumbing under
`wat/std/telemetry/*` and a separate wat-measure crate ships
WorkUnit / Event / scope HOF under `:wat::measure::*`. The split
predates the recognition that **measurement IS telemetry** — every
counter, every duration, every log line is a piece of telemetry
data. Splitting them across two namespaces and two crates was
artificial.

This arc folds them together. The substrate sheds telemetry; a
new `wat-telemetry/` crate owns the unified `:wat::telemetry::*`
namespace; a new `wat-telemetry-sqlite/` crate provides the
sqlite-backed sink. `wat-measure/` is deleted.

```
Before                                After
──────                                ─────
wat-rs                                wat-rs
├── wat/std/telemetry/                ├── wat/                 (no telemetry)
│   ├── Service.wat                   │   ├── std/             (core stdlib only)
│   ├── Console.wat                   │   └── holon/
│   └── ConsoleLogger.wat             │
├── wat/std/service/Console.wat       crates/
│                                     ├── wat-edn/
crates/                               ├── wat-lru/
├── wat-edn/                          ├── wat-holon-lru/
├── wat-lru/                          ├── wat-sqlite/          (Db primitives ONLY)
├── wat-holon-lru/                    ├── wat-telemetry/       (NEW — :wat::telemetry::*)
├── wat-sqlite/   (Db + Sqlite sink)  │   ├── wat/telemetry/
│   ├── wat/sqlite/                   │   │   ├── Service.wat
│   ├── wat/std/telemetry/Sqlite.wat  │   │   ├── Console.wat
│   └── src/auto.rs                   │   │   ├── ConsoleLogger.wat
└── wat-measure/  (WorkUnit/Event)    │   │   ├── WorkUnit.wat
                                      │   │   ├── Event.wat
                                      │   │   ├── types.wat
                                      │   │   └── uuid.wat
                                      │   └── src/{lib,workunit,shim}.rs
                                      └── wat-telemetry-sqlite/ (NEW)
                                          ├── wat/telemetry/sqlite/Sqlite.wat
                                          └── src/{lib,auto}.rs
                                          (wat-measure DELETED)
```

## What we know

### The honest unification

`:wat::measure::WorkUnit` opens a measurement scope, attaches
counters and durations and a tag set, ships them as
`:wat::measure::Event` rows through a
`:wat::std::telemetry::Service<:wat::measure::Event,_>`. Every
one of those nouns is telemetry. The measure/telemetry split
forced every consumer to import from two namespaces to do one
coherent thing.

After consolidation:

```scheme
(:wat::core::use! :wat::telemetry::WorkUnit)
(:wat::core::use! :wat::telemetry::Event)
(:wat::core::use! :wat::telemetry::Service)

(:wat::telemetry::WorkUnit/scope tags
  (:wat::core::lambda ((wu :wat::telemetry::WorkUnit) -> :T)
    body))
```

One namespace, one crate, one mental model.

### Why wat-sqlite stays unchanged

`wat-sqlite` provides general-purpose sqlite primitives — `:rust::sqlite::Db`
(opaque), `:wat::sqlite::open`, `:wat::sqlite::execute-ddl`,
`:wat::sqlite::pragma`, `:wat::sqlite::begin`, `:wat::sqlite::commit`.
These have no telemetry dependency. A consumer that just wants
to talk to sqlite (no Service<E,G>, no enum-derived schema) uses
wat-sqlite directly.

The sqlite-backed TELEMETRY SINK — `Sqlite/spawn`, `Sqlite/auto-spawn`,
the `:rust::sqlite::auto-{prep,install-schemas,dispatch}` shims —
moves to a new `wat-telemetry-sqlite/` crate that depends on
BOTH wat-telemetry (for Service<E,G>) AND wat-sqlite (for Db).
Two single-concern crates instead of one mixed-concern crate.

### Why Console moves

Console is a telemetry primitive — it's how telemetry events reach
stdout/stderr. Its driver (the `Vec<DriverPair>` ack-channel
machinery) is the canonical reference for arc 089 slice 5's
mini-TCP pattern; that pattern is generic over any service, but
Console is the lab's primary STDOUT telemetry surface. Folds
under `:wat::telemetry::Console::*` cleanly.

### What stays in the substrate

After the move:

- Language core: `:wat::core::*` (define, lambda, let*, match,
  enum, struct, typealias, HashMap, vec, etc.)
- Kernel: `:wat::kernel::*` (spawn, send, recv, select,
  HandlePool, etc.)
- IO: `:wat::io::*` (IOReader, IOWriter, file ops)
- Holon algebra: `:wat::holon::*` (HolonAST, Atom, Bind, Bundle,
  Permute, Thermometer, cosine, etc.)
- Time: `:wat::time::*` (Instant, now, epoch-nanos)
- Config: `:wat::config::*`
- Test: `:wat::test::*` (deftest, assert-eq, etc.)

The substrate is the language. Everything domain-aware — telemetry,
sinks, measurement — lives in crates.

### Rust-side moves

- `wat-rs/wat/std/telemetry/*` → `wat-telemetry/wat/telemetry/*`
- `wat-rs/wat/std/service/Console.wat` (driver side) →
  `wat-telemetry/wat/telemetry/Console-driver.wat` (or fold into
  Console.wat)
- `wat-sqlite/wat/std/telemetry/Sqlite.wat` →
  `wat-telemetry-sqlite/wat/telemetry/sqlite/Sqlite.wat`
- `wat-sqlite/src/auto.rs` (the three Rust shims) →
  `wat-telemetry-sqlite/src/auto.rs`
- `wat-measure/wat/measure/*` → `wat-telemetry/wat/telemetry/*`
  (Tag, Tags, WorkUnit, Event, types.wat, uuid.wat)
- `wat-measure/src/workunit.rs` → `wat-telemetry/src/workunit.rs`
- `wat-measure/src/shim.rs` → `wat-telemetry/src/shim.rs`

Cargo dependency edges:

```
wat        wat-edn    wat-lru    wat-sqlite
   ↑         ↑           ↑          ↑
   └─────────┴───┬───────┴──────────┘
                 │
          wat-telemetry  ←── wat-holon-lru (existing)
                 ↑
                 │  wat-sqlite ──┐
                 │               │
           wat-telemetry-sqlite ─┘
```

### The :wat::measure::uuid::v4 fate

Stays as a free function. Path renames to
`:wat::telemetry::uuid::v4` (or moves under WorkUnit's namespace
if more honest). The wat-edn `mint` feature dependency is
unchanged.

## What we don't know

- **Whether to keep `:wat::telemetry::uuid::*` as a sub-namespace
  or fold into `:wat::telemetry::WorkUnit/uuid-v4`.** Today it's
  a free function used at WorkUnit::new. If WorkUnit::new is the
  ONLY caller, fold. If callers want raw uuids for other
  purposes, keep separate. Default: keep separate (one verb per
  move, reusable).

- **Whether `Console` should split into `Console` (the driver
  crate-internal pattern) and `ConsoleLogger` (the consumer-facing
  surface).** They're two files today; might merge. Defer to
  the implementation.

- **Whether the lab's existing `:trading::telemetry::Sqlite/spawn`
  wrapper survives.** It thinly wraps wat-sqlite's Sqlite/spawn
  with the lab's pre-install (WAL + synchronous=NORMAL). After
  arc 096, the lab wraps wat-telemetry-sqlite's Sqlite/spawn
  instead. Same shape; namespace change. Lab handles its own
  migration.

## Slices

```
Slice 1 — scaffold wat-telemetry crate
  - crates/wat-telemetry/ per CONVENTIONS.md "publishable wat crate"
  - Move Service.wat, Console.wat, ConsoleLogger.wat from
    wat-rs/wat/std/telemetry/ + wat-rs/wat/std/service/Console.wat
  - Rename :wat::std::telemetry::* → :wat::telemetry::*
  - Rename :wat::std::service::Console → :wat::telemetry::Console
    (driver side merges into Console.wat or stays as
    Console-driver.wat)
  - wat-rs/src/stdlib.rs: remove the moved files from STDLIB_FILES
  - workspace Cargo.toml: add member; default-members
  - Substrate tests under wat-tests/std/telemetry/ + service/Console.wat
    move to wat-telemetry/wat-tests/

Slice 2 — fold wat-measure into wat-telemetry
  - Move types.wat, uuid.wat, WorkUnit.wat, Event.wat from
    wat-measure/wat/measure/ → wat-telemetry/wat/telemetry/
  - Rename :wat::measure::* → :wat::telemetry::* (Tag, Tags,
    WorkUnit, Event, uuid::v4)
  - Move workunit.rs + shim.rs from wat-measure/src/ to
    wat-telemetry/src/
  - wat-measure/wat-tests/measure/* → wat-telemetry/wat-tests/telemetry/*
  - Delete crates/wat-measure/
  - workspace Cargo.toml: remove member; default-members
  - Update wat-rs/Cargo.toml workspace members

Slice 3 — scaffold wat-telemetry-sqlite crate
  - crates/wat-telemetry-sqlite/ per the publishable-wat-crate template
  - Move wat-sqlite/wat/std/telemetry/Sqlite.wat →
    wat-telemetry-sqlite/wat/telemetry/sqlite/Sqlite.wat
  - Move wat-sqlite/src/auto.rs → wat-telemetry-sqlite/src/auto.rs
    (with all three Rust shims)
  - wat-telemetry-sqlite/Cargo.toml: deps on wat + wat-telemetry +
    wat-sqlite (chains the two underlying crates).
  - Rename :wat::std::telemetry::Sqlite/* →
    :wat::telemetry::sqlite::Sqlite/* (or wherever feels honest)
  - Move wat-sqlite/wat-tests/std/telemetry/{Sqlite,auto-spawn,
    edn-newtypes}.wat → wat-telemetry-sqlite/wat-tests/
  - wat-sqlite/Cargo.toml: now a leaner crate (just Db); strip
    src/auto.rs from src/lib.rs's register()
  - wat-sqlite/wat-tests/sqlite/Db.wat stays; it's the Db
    primitives test, no telemetry

Slice 4 — consumer sweep + tests + docs
  - examples/console-demo/ updates to new namespace
  - Substrate tests sweep (any remaining :wat::std::telemetry::*
    references in stdlib loaders or tests)
  - docs/CONVENTIONS.md — update the namespace privilege table +
    crate-folder-layouts examples
  - docs/USER-GUIDE.md — mass replace old namespace, update
    examples
  - docs/ZERO-MUTEX.md — Service<E,G> references new path
  - Per-crate README.md updates
  - cargo test --workspace must pass; pulse benchmark stays at
    ~45ms (lab tracks separately)
  - INSCRIPTION.md captures shape of moved files + namespace map
```

## What's NOT in this arc

- **Lab consumer migration.** External repo (`holon-lab-trading`).
  Lab updates `:trading::telemetry::*` to consume
  `:wat::telemetry::*` + the new wat-telemetry-sqlite path on
  its own next session. Arc 096 ships only the wat-rs side.

- **Functional changes to Service/Console/Sqlite/WorkUnit.** This
  arc is a pure namespace + crate-boundary move. Behavior
  identical. Tests pass without semantic changes.

- **A unified `:wat::kernel::ConnectionHandle<E>`.** Arc 095
  flagged that Console::Handle and Service::Handle have the
  same shape `(Tx, AckRx)`. A future housekeeping arc could
  pull them up to a kernel-level alias. Not in scope here.

## Surfaced by

User direction 2026-04-29, mid-arc-091-slice-4 / arc-095:

> "we need type aliases.... should wat/std/telemetry/Service.wat
> be in the measure crate?.... why isn't this in that namespace?
> calling this std feels very strange"

> "i think :wat::telemetry::* is the home telemetry things and
> :wat::measure uses them?... that feels honest?..."

> "or.. we fold :wat::measure::* into :wat::telemetry::* ...
> that's maybe the most honest... break telemetry into a its own
> crate and delete the measure crate once we're cut over?..."

> "we can then further break wat-telemetery-sqlite into its own
> dep... the wat-telemetry crate just provides a wrapper on
> console?.."

> "no... wat-sqlite is its own thing... wat-telemetry-sqlite deps
> on telemetry AND sqlite... otherwise i agree with your 4 points"

The honest read: measurement IS telemetry. The measure/telemetry
split was artificial. wat-sqlite is its own concern (sqlite
primitives); wat-telemetry-sqlite combines them as one specific
sink. Each crate has one concern.

User has phenomenal test confidence — full workspace `cargo test`
exercise + per-crate wat-test suites — so the cross-crate move
is well-instrumented.

## How sub-arcs / slices ship

Per the established pattern (arcs 089, 091, 095):
- Each slice gets implemented + tested + committed
- INSCRIPTION.md captures what shipped at arc close
- Tasks track slice progress
- Lab consumer migration deferred to lab's next session
