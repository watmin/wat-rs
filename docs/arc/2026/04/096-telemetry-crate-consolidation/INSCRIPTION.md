# Arc 096 — telemetry crate consolidation — INSCRIPTION

**Status:** shipped 2026-04-29. Four slices, one session, zero
test regressions.

The pre-arc shape spread telemetry across two namespaces and three
crates: `:wat::std::telemetry::*` lived in the substrate (Service,
Console, ConsoleLogger), `:wat::measure::*` in a separate
wat-measure crate (WorkUnit, Event, scope, Tags, uuid), and
`wat-sqlite` mixed general-purpose Db primitives with the
sqlite-backed telemetry sink. The user surfaced the diagnosis:
**measurement IS telemetry**, the splits were artificial, and
mixing the Db primitives with the sink-specific code in one crate
hid the dependency direction.

After this arc:

- **Substrate** sheds telemetry. Just kernel + holon + io + core.
- **`wat-telemetry/`** owns `:wat::telemetry::*` end-to-end.
  Service<E,G>, Console, ConsoleLogger, WorkUnit, Event, Tags,
  uuid::v4 — one crate, one namespace.
- **`wat-telemetry-sqlite/`** depends on both wat-telemetry and
  wat-sqlite. Provides ONE specific sink: `Sqlite/spawn` +
  `Sqlite/auto-spawn` + the three Rust shims for arc-085
  enum-derived schema.
- **`wat-sqlite/`** shrinks to just `:wat::sqlite::Db` —
  general-purpose sqlite primitives, no telemetry awareness.
- **`wat-measure/`** is gone. Folded into wat-telemetry.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped

### Slice 1 — scaffold wat-telemetry

`crates/wat-telemetry/` per the publishable-wat-crate template.
Service.wat, Console.wat (the dispatcher factory), and
ConsoleLogger.wat moved out of `wat-rs/wat/std/telemetry/` into
the new crate's `wat/telemetry/`. The substrate's
`:wat::std::service::Console` (the paired-channel mini-TCP
DRIVER from arc 089 slice 5) STAYS in the substrate as a generic
service-pattern reference — it's not telemetry-specific, and the
new wat-telemetry's Console.wat WRAPS it.

Namespace rewrite (only the moved files; substrate's
`:wat::std::service::Console` paths untouched):
  `:wat::std::telemetry::*  →  :wat::telemetry::*`

`wat-rs/src/stdlib.rs` dropped the three moved entries. Substrate's
Console-driver entry retained with explanatory comment.

Consumer sweep — the same namespace rewrite + new dep on
wat-telemetry — touched wat-sqlite (Sqlite.wat + auto.rs +
Cargo.toml + tests/test.rs), wat-measure (types.wat),
examples/console-demo (main.rs + Cargo.toml + main.wat).

### Slice 2 — scaffold wat-telemetry-sqlite

`crates/wat-telemetry-sqlite/` depends on wat-telemetry +
wat-sqlite + wat-edn + holon + rusqlite. Owns
`:wat::telemetry::Sqlite/*`. Files moved:

- `wat-sqlite/wat/std/telemetry/Sqlite.wat` →
  `wat-telemetry-sqlite/wat/telemetry/Sqlite.wat`
- `wat-sqlite/src/auto.rs` → `wat-telemetry-sqlite/src/auto.rs`
- `wat-sqlite/wat-tests/std/telemetry/{Sqlite,auto-spawn,
  edn-newtypes}.wat` → `wat-telemetry-sqlite/wat-tests/telemetry/`

`WatSqliteDb.conn` widened from `pub(crate)` to `pub` so
auto.rs (now in a sibling crate) can call `prepare_cached` +
`execute` on the underlying Connection. Comment explains the
cross-crate justification.

The moved Sqlite.wat had a `(:wat::load-file! "../../sqlite/Db.wat")`
form for the in-crate path; retired since wat-sqlite's Db.wat
reaches the consumer through `deps: [wat_sqlite, ...]` in their
wat::main! / test!, which composes Db.wat's types into the same
parse pass.

`wat-sqlite/Cargo.toml` shrank — dropped `wat-edn`, `wat-telemetry`,
`holon` deps (those served auto.rs only). `wat-sqlite/src/lib.rs`
dropped `mod auto` + `auto::register` from `register()`.
Description updated: "general-purpose sqlite primitives".

### Slice 3 — fold wat-measure into wat-telemetry

Files moved into `wat-telemetry/`:

- `wat-measure/wat/measure/{types,uuid,WorkUnit,Event}.wat`
  → `wat-telemetry/wat/telemetry/`
- `wat-measure/src/{workunit,shim}.rs`
  → `wat-telemetry/src/`
- `wat-measure/wat-tests/measure/{uuid,WorkUnit}.wat`
  → `wat-telemetry/wat-tests/telemetry/`

Namespace rewrites:
  `:wat::measure::*  →  :wat::telemetry::*`
  `:rust::measure::* →  :rust::telemetry::*`
  `:wat-measure::*   →  :wat-telemetry::*` (test prefixes)

`wat-telemetry/src/lib.rs` updated to declare both modules
(`pub mod shim;`, `pub mod workunit;`), list all 7 source files
in declaration order (Service, types, Event, uuid, WorkUnit,
Console, ConsoleLogger), and forward `register()` to the
shim + workunit registrars.

`wat-telemetry/Cargo.toml` gained `wat-macros` and
`wat-edn = { ..., features = ["mint"] }` (uuid::v4's wat-edn
backend).

`crates/wat-measure/` deleted entirely. Workspace Cargo.toml
dropped the member from both `members` and `default-members`.

### Slice 4 — docs sweep + INSCRIPTION

- `docs/CONVENTIONS.md`, `docs/USER-GUIDE.md`, `docs/README.md`
  swept for `:wat::std::telemetry::*` → `:wat::telemetry::*`.
- `src/runtime.rs` — two doc comments updated.
- INSCRIPTION (this file).

Lab consumer migration deferred — external repo.

---

## What's NOT in this arc

- **Lab consumer migration.** External repo
  (`holon-lab-trading`). The lab's
  `:trading::telemetry::Sqlite/spawn` wrapper updates to depend
  on wat-telemetry-sqlite + wat-telemetry on its own next
  session.
- **Functional changes.** Pure namespace + crate-boundary moves;
  every test passes with no semantic edits.
- **A unified `:wat::kernel::ConnectionHandle<E>`.** Console::Handle
  and Service::Handle have the same shape `(Tx, AckRx)`. Pulling
  them up to a kernel-level alias is a future housekeeping arc.

---

## Surfaced by

User direction 2026-04-29, mid-arc-091-slice-4 / arc-095:

> "calling this std feels very strange... should
> wat/std/telemetry/Service.wat be in the measure crate?..."

> "i think :wat::telemetry::* is the home telemetry things and
> :wat::measure uses them?... that feels honest?..."

> "or.. we fold :wat::measure::* into :wat::telemetry::*... that's
> maybe the most honest... break telemetry into a its own crate
> and delete the measure crate once we're cut over?..."

> "we can then further break wat-telemetry-sqlite into its own
> dep... the wat-telemetry crate just provides a wrapper on
> console?.."

> "no... wat-sqlite is its own thing... wat-telemetry-sqlite deps
> on telemetry AND sqlite... otherwise i agree with your 4
> points"

> "i want to work on the names once you wrap up - i do not care
> how much of a refactor it is - we have phenominal testing"

The "extremely messy" framing from arc 095 ("client passes the
client into the server") reappeared at the architectural level:
crates with mixed concerns leak the dependency direction. Each
crate now has one concern: wat-telemetry is the abstract shell;
wat-sqlite is the Db driver; wat-telemetry-sqlite is the one
specific sink that combines them.

---

## Test coverage

Workspace summary: `cargo test --workspace` zero failures across
all crates; 88 test groups green. The count dropped from 91
(pre-arc) because wat-measure isn't its own test target anymore —
its tests moved to wat-telemetry.

Specific lock points:

- `wat-telemetry/wat-tests/telemetry/{Service,Console,WorkUnit,
  uuid}.wat` — 16 deftests covering Service<E,G> lifecycle,
  Console dispatcher, WorkUnit data primitives, scope HOF, uuid
  uniqueness.
- `wat-telemetry-sqlite/wat-tests/telemetry/{Sqlite,auto-spawn,
  edn-newtypes}.wat` — 4 deftests covering spawn lifecycle,
  batch-log throughput, auto-derived schema, Tagged/NoTag
  newtype TEXT binding.
- `wat-sqlite/wat-tests/sqlite/Db.wat` — 5 deftests covering
  open/execute/pragma/begin/commit. No telemetry references.

---

## Files moved, files deleted

Moved (substrate → wat-telemetry):
- `wat-rs/wat/std/telemetry/Service.wat`
- `wat-rs/wat/std/telemetry/Console.wat`
- `wat-rs/wat/std/telemetry/ConsoleLogger.wat`
- `wat-rs/wat-tests/std/telemetry/Service.wat`
- `wat-rs/wat-tests/std/telemetry/Console.wat`

Moved (wat-sqlite → wat-telemetry-sqlite):
- `crates/wat-sqlite/wat/std/telemetry/Sqlite.wat`
- `crates/wat-sqlite/src/auto.rs`
- `crates/wat-sqlite/wat-tests/std/telemetry/Sqlite.wat`
- `crates/wat-sqlite/wat-tests/std/telemetry/auto-spawn.wat`
- `crates/wat-sqlite/wat-tests/std/telemetry/edn-newtypes.wat`

Moved (wat-measure → wat-telemetry):
- `crates/wat-measure/wat/measure/types.wat`
- `crates/wat-measure/wat/measure/uuid.wat`
- `crates/wat-measure/wat/measure/WorkUnit.wat`
- `crates/wat-measure/wat/measure/Event.wat`
- `crates/wat-measure/src/workunit.rs`
- `crates/wat-measure/src/shim.rs`
- `crates/wat-measure/wat-tests/measure/uuid.wat`
- `crates/wat-measure/wat-tests/measure/WorkUnit.wat`

Deleted:
- `crates/wat-measure/` (entire crate)

Created:
- `crates/wat-telemetry/`
- `crates/wat-telemetry-sqlite/`
- `docs/arc/2026/04/096-telemetry-crate-consolidation/{DESIGN,INSCRIPTION}.md`

Updated cross-cutting:
- `Cargo.toml` (workspace) — added wat-telemetry +
  wat-telemetry-sqlite; removed wat-measure.
- `src/stdlib.rs` — dropped 3 entries; substrate's Console
  driver retained.
- `src/runtime.rs` — 2 comment refs updated.
- `crates/wat-sqlite/{Cargo.toml,src/lib.rs,tests/test.rs}` —
  scope narrowed.
- `crates/wat-sqlite/src/lib.rs` — `WatSqliteDb.conn` widened
  to `pub`.
- `examples/console-demo/{Cargo.toml,src/main.rs,wat/main.wat}` —
  consumer migration.
- `docs/CONVENTIONS.md`, `docs/USER-GUIDE.md`, `docs/README.md` —
  namespace sweep.
