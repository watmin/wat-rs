# Arc 085 — enum-derived sqlite schemas (`Sqlite/auto-spawn`)

**Status:** PROPOSED 2026-04-28. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 048 — user-defined enum values + the `EnumDef` / `EnumVariant`
  decl registry that this arc reflects on.
- Arc 083 — `:wat::sqlite::Db` substrate primitives (open / execute-ddl)
  + `:wat::std::telemetry::Sqlite/spawn` with the explicit (consumer-
  provides-hooks) shape this arc complements.
- Arc 084 — `:wat::sqlite::execute` + `:wat::sqlite::Param` enum.
  The auto-spawn dispatcher writes through these primitives.

**Surfaced by:** the user's recognition (2026-04-28) — when shown the
explicit `Sqlite/spawn` consumer UX (write SQL, wrap each value in a
Param variant per dispatcher arm):

> "Level 3.... that form is so incredibly small... how does this
> work?... if level 3 works - holy shit that's wild...."

The lab's existing `LogEntry` enum decl (`PaperResolved` with 10
typed fields, `Telemetry` with 7) IS the schema. Every column name
in `paper_resolutions` is a renaming of a field name; every column
type is a mapping of a field type; every value bound at INSERT is
the matching field's payload. The enum decl is the source of truth;
the SQL dispatcher today is mechanical translation that can't drift
from it (because if it did, the test breaks). Mechanical translation
that can't drift IS what substrate-level derivation is for.

---

## What this arc is, and is not

**Is:**
- A capability-carrier addition: `SymbolTable.types: Option<Arc<TypeEnv>>`
  set at freeze time. Per memory `feedback_capability_carrier`,
  the right shape — TypeEnv lives in `FrozenWorld`, so making it
  reachable from shims threads it through SymbolTable alongside
  the existing `encoding_ctx` / `source_loader` / sigma-fns.
- A new factory `:wat::std::telemetry::Sqlite/auto-spawn` —
  sibling to arc 083's explicit `Sqlite/spawn`. Returns the same
  `Service::Spawn<E>` tuple shape so callers wire it into
  `:user::main` identically.
- The Rust shim machinery in wat-sqlite that walks an `EnumDef`,
  derives schemas + INSERTs, and ships them as cached state inside
  the worker thread.
- One naming convention pair: variant name PascalCase →
  table name snake_case; field name kebab-case → column name
  snake_case. Single source of truth for both is the enum decl.

**Is not:**
- A schema migration system. The first run creates tables. Adding a
  new variant in a future run runs `CREATE TABLE IF NOT EXISTS` for
  the new variant; existing tables stay. Adding/renaming columns is
  out of scope; consumer drops tables manually.
- A general persistence framework. SQLite specifically. No
  abstraction for "any backend." If MTG / truth-engine wants
  something other than SQLite, that's a sibling factory.
- A read API. Reads stay out-of-band via sqlite3 CLI; same posture
  as arc 083.
- A query DSL. The substrate doesn't synthesize SELECTs. The
  consumer reads via SQL strings against the auto-derived schema.
- A `:Vec<T>` / `:HashMap<K,V>` field encoder. Variants must have
  scalar fields (`:i64`, `:f64`, `:String`, `:bool`, optionally
  `:Option<T>` of one of those). Anything else panics at
  `auto-spawn` startup with a diagnostic.

---

## Surface

```scheme
;; Sibling to arc 083's Sqlite/spawn. Same Service::Spawn<E>
;; return shape; same `:user::main`-side wiring; different sink
;; provisioning: substrate walks E's enum decl at startup,
;; synthesizes schemas + INSERTs, wires the dispatcher closure
;; without consumer code.
;;
;; The consumer passes the enum NAME as a keyword value (no
;; angle-bracket generics at the call site — substrate looks up
;; the type by path through SymbolTable.types).

(:wat::std::telemetry::Sqlite/auto-spawn<E,G>
  (enum-name :wat::core::keyword)
  (path :String)
  (count :i64)
  (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
  -> :wat::std::telemetry::Service::Spawn<E>)
```

### Lab UX after this arc

```scheme
;; The lab's LogEntry decl already exists in wat/io/log/LogEntry.wat;
;; it stays where it is. The thin wrapper:

(:wat::core::typealias :trading::telemetry::Spawn
  :wat::std::telemetry::Service::Spawn<trading::log::LogEntry>)

(:wat::core::define
  (:trading::telemetry::Sqlite/spawn<G>
    (path :String)
    (count :i64)
    (cadence :wat::std::telemetry::Service::MetricsCadence<G>)
    -> :trading::telemetry::Spawn)
  (:wat::std::telemetry::Sqlite/auto-spawn
    :trading::log::LogEntry path count cadence))
```

That's the whole lab telemetry surface. Five lines. **Zero SQL.
Zero Db visible. Zero Param wrapping. Zero dispatcher.**
`wat/io/telemetry/{maker,dispatch,translate-stats}.wat` and
`wat/io/RunDb.wat` and `wat/io/RunDbService.wat` all delete in
slice 3 of arc 083.

The trader's existing call sites stay — produce a
`:trading::log::LogEntry::PaperResolved ...` value, send it through
the spawn's req-tx. The substrate handles the rest.

---

## Mechanism

### Reflection — `SymbolTable.types`

Today's `SymbolTable` carries every "ambient capability the runtime
might need" (encoding context, source loader, macro registry, sigma
fns). The TypeEnv that holds enum decls already lives in
`FrozenWorld`; this arc puts an `Arc<TypeEnv>` reference onto
SymbolTable so shims that need to inspect types can:

```rust
pub struct SymbolTable {
    // ... existing fields ...
    pub types: Option<Arc<TypeEnv>>,
}
```

`freeze.rs` populates it after type-checking commits, alongside the
existing capability installs (`set_encoding_ctx`, `set_source_loader`).

### Schema derivation

For each `EnumVariant::Tagged { name, fields }` in the target enum:

```
table_name = pascal_to_snake(variant.name)        // PaperResolved → paper_resolved
columns    = fields.map(|(name, type)| {
    (kebab_to_snake(name),                        // run-name → run_name
     type_to_affinity(type))                      // :String → "TEXT NOT NULL"
})
ddl = format!("CREATE TABLE IF NOT EXISTS {table_name} ({cols});")
```

Type → SQLite affinity mapping:

| wat type | SQLite affinity |
|---|---|
| `:String` | `TEXT NOT NULL` |
| `:i64` | `INTEGER NOT NULL` |
| `:f64` | `REAL NOT NULL` |
| `:bool` | `INTEGER NOT NULL` |
| `:Option<T>` | `T`-affinity NULL |

Anything else panics at `auto-spawn` startup with a diagnostic
naming the variant + field + unsupported type. Future arcs add
support when a consumer surfaces a need.

Unit variants emit zero schema (no fields). Sending a unit-variant
entry at runtime executes nothing; the row is just "this happened
once" and there's nowhere to insert. Defer until a consumer needs
it (likely shape: a one-column `(timestamp_ns)` event table when
support lands).

### INSERT derivation

Per Tagged variant, cache one INSERT statement at startup:

```
INSERT INTO {table_name} ({col1}, {col2}, ...)
  VALUES (?1, ?2, ...);
```

### Dispatch

Per entry (Value::Enum) in the worker thread:
1. Look up variant_name in cached map → get (insert_sql, field_types).
2. Walk `entry.fields` parallel to `field_types`. Each
   `(value, type)` pair becomes one Param:
   - `(Value::String(s), :String)` → `Param::Str(s)`
   - `(Value::i64(n), :i64)` → `Param::I64(n)`
   - `(Value::f64(x), :f64)` → `Param::F64(x)`
   - `(Value::bool(b), :bool)` → `Param::Bool(b)`
   - `(Value::Option(None), :Option<_>)` → `Param::Null` (when slice
     2 adds Null) — for slice 1, panic if Option encountered.
3. Call substrate's `:wat::sqlite::execute`.

### Naming conventions

PascalCase → snake_case for table names: `PaperResolved` →
`paper_resolved`. Multi-word camel: `MetricSnapshot` →
`metric_snapshot`. Acronyms join: `HTTPRequest` → `httprequest`
(the simple algorithm; can refine later if a consumer cares).

Kebab-case → snake_case for columns: `run-name` → `run_name`.
Snake-case fields stay as-is. Plain words stay as-is.

These are conventions, not magic. Override hook deferred — if a
consumer's existing schema uses non-conventional names (lab's
`paper_resolutions`, plural), they either rename the table on
disk to match the derivation, or arc 086 adds an annotation
mechanism. Default: conventions; consumer can rename their data.

(The lab's existing `paper_resolutions` table will become
`paper_resolved` post-migration. The lab's tests / queries adjust
to the new name. One mechanical sweep.)

---

## Slice plan

### Slice 1 — `SymbolTable.types` capability carrier

`src/runtime.rs`:
- Add `pub types: Option<Arc<TypeEnv>>` field to `SymbolTable`.
- Add `set_types(Arc<TypeEnv>)` method.

`src/freeze.rs`:
- After type-check commits, call `symbols.set_types(Arc::new(types.clone()))`.
- Adjust signatures to keep `types` available before move into
  FrozenWorld (or clone before the FrozenWorld build).

Verify: `cargo test --workspace` clean. No new tests (capability
carrier is exercised in slice 2).

### Slice 2 — `auto-spawn` factory + tests

`crates/wat-sqlite/src/lib.rs`:
- New module / struct: `AutoSchemaWorker` with the cached
  `HashMap<variant_name, (insert_sql, field_types)>`.
- New free function or shim that:
  1. Reads `sym.types` for the enum decl.
  2. Walks variants, derives schemas + INSERTs (the mapping above).
  3. Spawns the worker thread that opens the Db, runs schemas,
     enters Service/loop with the auto-derived dispatcher.
- Returns `Service::Spawn<E>` (HandlePool + ProgramHandle tuple).

`crates/wat-sqlite/wat/std/telemetry/Sqlite.wat`:
- Add `:wat::std::telemetry::Sqlite/auto-spawn` thin wrapper.
- The explicit `Sqlite/spawn` from arc 083 stays — both factories
  coexist; `auto-spawn` is the consumer-friendly default,
  `spawn` is the override-everything escape hatch.

`crates/wat-sqlite/wat-tests/std/telemetry/`:
- New `auto-spawn.wat` deftest with a tiny throwaway test enum
  declaring two variants of mixed types. Send entries; verify rows
  land via the existing-test-pattern (sqlite3 CLI side-channel).

### Slice 3 — INSCRIPTION + arc 083 slice 3 collapse

`docs/arc/2026/04/085-.../INSCRIPTION.md` — what shipped, what's
deferred (Null/Option, schema migrations, table-name overrides).

The lab migration (arc 083 slice 3) becomes a deletion sweep:
- `wat/io/telemetry/Sqlite.wat` reduces to the 5-line auto-spawn wrapper.
- `wat/io/telemetry/dispatch.wat` deletes.
- `wat/io/telemetry/maker.wat` deletes.
- `wat/io/telemetry/translate-stats.wat` deletes (auto-spawn
  derives its own stats translator? — see Q3).
- `wat/io/RunDb.wat` deletes.
- `wat/io/RunDbService.wat` deletes.
- `src/shims.rs` — drop `WatRunDb` + its registrations.
- `Cargo.toml` — drop direct `rusqlite` dep, add `wat-sqlite`.
- Lab tests update for the new table name (`paper_resolutions` →
  `paper_resolved`).

---

## Open questions

### Q1 — Substrate-derived stats vs consumer-derived

`Sqlite/spawn` (arc 083) takes a consumer-provided `stats-translator`
that maps substrate `Stats` → `Vec<E>`. That's how the service's own
heartbeat lands as entries the dispatcher writes.

For `auto-spawn`, the consumer's E might not have a "Stats-like"
variant. Two answers:
- (a) `auto-spawn` requires E to declare a variant that matches
  substrate Stats's shape (`batches :i64`, `entries :i64`,
  `max-batch-size :i64`). Substrate auto-translates. Tight
  coupling.
- (b) `auto-spawn` takes the cadence as `null-metrics-cadence` only
  (no self-heartbeat) — service runs without emitting its own rows.
  Consumer who wants heartbeat uses the explicit `Sqlite/spawn`.
- (c) `auto-spawn` takes the cadence + an optional translator;
  default is no-translator + null-cadence; passing both opts in to
  heartbeat.

**Default: (b).** Auto-spawn is "give me a sink for entries"; if
you want substrate's own emissions in your sink, use the explicit
factory. The simpler default.

### Q2 — Override hook for table names

Default: derived. Future arc adds an annotation mechanism:

```scheme
(:wat::core::enum :trading::log::LogEntry
  (PaperResolved
    (run-name :String) ...)
  :@table "paper_resolutions")    ;; arc 086 syntax — not this arc
```

Until that lands, consumers either:
- accept the derived names (rename existing data on disk), or
- use the explicit `Sqlite/spawn` and write their own schemas.

### Q3 — How does `auto-spawn` reach `sym.types` from the
spawned worker thread?

The shim's body runs in the CALLER thread (sets up channels +
spawns), then the worker thread executes the entry function. The
worker function needs the cached SQL strings.

The setup-side shim has access to `sym.types`; it walks the enum
decl and produces a `Vec<(variant_name, insert_sql, field_types)>`
configuration vec. That vec is plain data (Send-safe) and gets
cloned into the spawn closure's environment. The worker uses the
pre-derived data; never reaches back for `sym.types`.

### Q4 — Is `EnumValue.fields` ordered consistently with the
declaration?

Per arc 048 (constructor synthesis), tagged variants synthesize a
function whose param list is the declaration order. Constructing
`(:E::Variant a b c)` calls `:wat::core::variant :E :Variant a b c`
which produces an `EnumValue` with `fields: vec![a, b, c]`. So
`entry.fields[i]` corresponds to `variant.fields[i]` in the decl.
**Confirmed by reading runtime.rs:1316-1330.**

---

## Test strategy

- Slice 1: workspace tests stay green (capability carrier is
  invisible to existing code).
- Slice 2: tiny test enum, two variants, mixed types, end-to-end
  spawn → batch-log → drop → join → verify rows via sqlite3 CLI.
  Existing arc 083 tests stay green (the explicit factory path
  is unchanged).
- Slice 3: arc 083 slice 3 lab migration; existing lab proofs stay
  green.

---

## Dependencies

**Upstream:** Arc 084 (parameterized execute — auto-dispatcher
writes through it). Arc 048 (enum decl registry the
`SymbolTable.types` exposes). Arc 083 (the `Sqlite/spawn` shape this
sibling factory mirrors).

**Downstream:** Arc 083 slice 3 (lab migration collapses to a
deletion sweep + the 5-line auto-spawn wrapper). Future cross-domain
consumers (MTG, truth-engine) get the same UX for free.

PERSEVERARE.
