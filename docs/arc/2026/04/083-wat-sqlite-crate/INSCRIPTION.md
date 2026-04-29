# Arc 083 — `crates/wat-sqlite/` substrate crate — INSCRIPTION

**Status:** shipped 2026-04-28. Same-session sequence with arcs 084 + 085.

The substrate now ships sqlite-backed telemetry as a first-class
crate. The trading lab consumes it via two flat hooks (`Cargo.toml`
dep + 5-line wat wrapper); domain code declares `LogEntry` and gets
schema + INSERT + binder for free. The lab's domain-specific RunDb
Rust shim has been retired.

The arc came in three slices originally proposed plus two follow-on
arcs that surfaced during implementation:

- **Slice 1** — `:wat::sqlite::Db` substrate primitives (open +
  execute-ddl) + crate scaffold.
- **Slice 2** — `:wat::std::telemetry::Sqlite/spawn` with
  consumer-provides-hooks shape (schema-install + dispatcher +
  stats-translator).
- **[Arc 084](../084-sqlite-execute-params/INSCRIPTION.md)** —
  `:wat::sqlite::execute db sql params` + `:wat::sqlite::Param`
  enum. Promoted from arc 083 slice 1 deferral when the lab
  migration hit "ugly SQL string concat" friction.
- **[Arc 085](../085-enum-derived-sqlite-schemas/INSCRIPTION.md)** —
  `Sqlite/auto-spawn` derives schemas + INSERTs + binders from the
  consumer's enum decl. The user's "holy shit that's wild" moment;
  the lab migration collapses to a deletion sweep.
- **Slice 3** — lab migration off the typed Rust shim.

**Designs:** [`DESIGN.md`](./DESIGN.md) (slices 1+2) plus the two
follow-on arcs' designs.

---

## Slice 1 — `:wat::sqlite::Db`

`crates/wat-sqlite/src/lib.rs`:
- `WatSqliteDb` struct wrapping `rusqlite::Connection`,
  `#[wat_dispatch]`'d under `:rust::sqlite::Db` with thread-owned scope.
- `open(path) -> Self`, `execute_ddl(self, ddl)`.

`crates/wat-sqlite/wat/sqlite/Db.wat`:
- `:wat::sqlite::Db` typealias to the Rust shim.
- `:wat::sqlite::open` + `:wat::sqlite::execute-ddl` thin defines.

Tests at `wat-tests/sqlite/Db.wat` — open + execute-ddl
roundtrip.

## Slice 2 — `:wat::std::telemetry::Sqlite/spawn`

`crates/wat-sqlite/wat/std/telemetry/Sqlite.wat`:
- `Sqlite/run<E,G>` — top-level worker entry. Opens Db inside the
  spawned thread, runs the consumer's `schema-install` hook, builds
  the curried dispatcher closure capturing the worker-local Db,
  enters substrate's `Service/run`.
- `Sqlite/spawn<E,G>` — caller-side wiring. N bounded(1) Request<E>
  pairs → HandlePool → spawn the worker.

Hooks (the consumer's seam) — TWO FLAT, not one nested:
- `schema-install :fn(Db)->()` — installs schemas via
  `(:wat::sqlite::execute-ddl db ddl)`.
- `dispatcher :fn(Db,E)->()` — per-entry router; substrate curries
  Db before handing `:fn(E)->()` to Service/loop.

The single-nested `init-fn :fn(Db)->fn(E)->()` shape was considered
and rejected; verbose-flat is the honest shape (see DESIGN's
post-rejection note + the user's "we choose - always - simple and
honest" framing).

Tests at `wat-tests/std/telemetry/Sqlite.wat` — spawn → drop → join
lifecycle + spawn → batch-log three entries → drop → join with
inserts via wat-sqlite primitives.

## Slice 3 — Lab migration

`holon-lab-trading/Cargo.toml`:
- Added `wat-sqlite = { path = "../wat-rs/crates/wat-sqlite" }`.
- Removed direct `rusqlite = "0.31"` dependency. The lab no longer
  needs rusqlite — substrate handles every sqlite call.

`holon-lab-trading/src/main.rs`:
- `wat::main! { deps: [shims, wat_sqlite] }` (added wat_sqlite).

`holon-lab-trading/src/shims.rs`:
- `WatRunDb` struct deleted. Schema constant deleted. `#[wat_dispatch]`
  impl block deleted (open + execute_ddl + log_paper_resolved +
  log_telemetry — all gone).
- `wat_sources()` reduced to two surfaces (CandleStream + LogEntry +
  emit-metric helper). `RunDb.wat`, `RunDbService.wat`, and
  `schema.wat` source entries deleted.
- `register()` no longer calls `__wat_dispatch_WatRunDb::register`.

Wat surface deletes:
- `wat/io/RunDb.wat`
- `wat/io/RunDbService.wat`
- `wat/io/log/schema.wat` (substrate derives schemas)
- `wat/io/telemetry/dispatch.wat` (substrate derives dispatcher)
- `wat/io/telemetry/maker.wat` (no longer needed; auto-spawn
  doesn't take a maker)
- `wat/io/telemetry/translate-stats.wat` (auto-spawn uses null
  cadence; no stats-translator)

Wat surface replaces:
- `wat/io/telemetry/Sqlite.wat` reduced to a 5-line typealias +
  thin spawn wrapper that delegates to substrate's
  `:wat::std::telemetry::Sqlite/auto-spawn` with `:trading::log::LogEntry`
  as the entry type.
- `wat/io/log/LogEntry.wat` updated header — the enum is now the
  source of truth for the on-disk schema (substrate derives from it).
- `wat/main.wat` updated load list — drops the deleted files.
- `wat/cache/reporter.wat` switched typed handles from
  `:trading::rundb::Service::*` to substrate equivalents.
- `wat/services/treasury.wat` same swap (~6 sites).

Tests:
- `wat-tests/io/log/telemetry.wat` rewritten — uses
  `:trading::telemetry::Sqlite/spawn` + substrate batch-log.
- `wat-tests/io/RunDb.wat`, `wat-tests/io/RunDbService.wat`,
  `wat-tests/io/telemetry/maker.wat`,
  `wat-tests/io/telemetry/translate-stats.wat` — all deleted.
- `wat-tests/io/telemetry/Sqlite.wat` header updated; body unchanged
  (calls `:trading::telemetry::Sqlite/spawn` which now delegates).
- `wat-tests-integ/proof/002-thinker-baseline/...wat` —
  outcome-logging restructured: builds Vec<LogEntry::PaperResolved>
  during the run, batch-logs at end of each thinker. Inner-scope
  packs results into a Counters struct so outer can join the
  driver before reading them.
- `wat-tests-integ/proof/003-thinker-significance/...wat` —
  type-name swap (already used Service shape; just renamed types).
- `wat-tests-integ/proof/004-cache-telemetry/...wat` — same
  type-name swap. Two-driver lockstep (cache + telemetry) intact.
- `wat-tests-integ/experiment/008-treasury-program/explore-handles.wat`
  — comment-only update.
- `tests/test.rs` and the three `tests/proof/proof_*.rs` files —
  added `wat_sqlite` to deps lists.

The type alias `:trading::telemetry::Spawn` works as a chain
through substrate's `:wat::std::telemetry::Service::Spawn<E>`. Test
files using deftest sandboxes need `(:wat::load-file!
"wat/io/telemetry/Sqlite.wat")` in the make-deftest prelude so the
typealias is in scope; this is now standard for any test that
references the alias.

---

## Verification

- `cargo build --workspace` clean (workspace + all wat-rs crates).
- `cargo build --manifest-path holon-lab-trading/Cargo.toml --tests
  --features proof-002,proof-003,proof-004` clean (all proof
  binaries compile).
- `cargo test --manifest-path holon-lab-trading/Cargo.toml --test
  test` green (lab's fast wat-suite — 346+ deftests pass).
- `cargo test --workspace` (wat-rs side): 728 substrate Rust tests +
  every wat-suite green; auto-spawn smoke test verifies the
  end-to-end roundtrip with rows landing in correct SQLite
  affinities.
- The slow proofs (002 / 003 / 004) compile + freeze; their wat
  source loads cleanly under the lab's deftest harness. Running
  them on real data is gated behind `--features proof-*` and
  takes minutes; deferred to a separate session.

---

## Cost of the journey

- One arc spawned a sub-arc (084) when the deferral inside slice 1
  surfaced as friction in slice 3. Memory `feedback_absence_is_signal`
  reinforced.
- A second sub-arc (085) emerged when explaining the consumer UX
  surfaced "Level 3" — the substrate could derive the schema +
  INSERT + binder from the enum decl rather than asking the
  consumer to write them. The user's "holy shit that's wild"
  recognition triggered. The right substrate work let the lab
  migration collapse from "translate every dispatcher arm" to
  "delete the old machinery + 5 new lines."
- The session shape — three arcs in one stretch — exemplifies the
  iterative-complexity principle (memory `feedback_iterative_complexity`):
  the right next slice surfaces from walking the previous slice's
  honest signal, not from up-front planning.

## Consumer impact

The substrate now provides:
- `:wat::sqlite::Db` (open / execute-ddl / execute with typed Param)
- `:wat::sqlite::Param` enum (I64 / F64 / Str / Bool)
- `:wat::std::telemetry::Sqlite/spawn` (explicit; takes hooks)
- `:wat::std::telemetry::Sqlite/auto-spawn` (derives from enum decl)

Every future cross-domain consumer (MTG, truth-engine, anything
that wants typed structured persistence) declares an enum, calls
auto-spawn, and ships. Zero SQL written. Zero Rust shim of their
own. The substrate has become an opinionated platform for
"declared data → on-disk persistence."

PERSEVERARE.
