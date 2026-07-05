# BRIEF — S1: the `:wat::sqlite'` RAW interop (fresh Rust — the heaviest stone)

> **Executor: one sonnet SHADOWDANCER.** The orchestrator scouted the lair, confirmed the gap (RED probe:
> `:rust::sqlite'::Connection::open` → `UnresolvedReference` today), and ruled the placement (**sqlite is CORE**).
> Work ONLY in `/home/watmin/work/holon/wat-rs/` (`pwd` first; anchor git with `git -C …`; `.claude/worktrees/` is
> illegal). `cargo wat <file>` to dogfood; `cargo nextest run --release` (NEVER `cargo test`); `cargo build` to check
> compile. **Commit NOTHING** — the orchestrator weighs + commits.

## The work, in one paragraph

Give wat a **fresh**, honest binding to sqlite (rusqlite), in CORE: a `:rust::sqlite'` Rust shim (opaque
thread-owned `Connection`, the seven verbs, **errors as VALUES — never panic**), a baked `:wat::sqlite'` wat surface
(the `Error`/`Fault` recovery-axis enum + the thin wrap that lifts raw fault-values into it), and a `deftest'` gate.
This is the RAW layer *below* the backend-agnostic `Store` contract; **S2** (next stone) satisfies
`:wat::query::Store` with it, differential-tested against the S-mem MemStore oracle. The design is
`DESIGN-STONE-S1-sqlite-interop.md` — **read it first, in full.**

## The ruling (grounded — do not re-litigate)

sqlite is **CORE**. rusqlite joins bigint as a hard core dep; `:wat::query` is backed by memory OR sqlite, both
first-class core. `with_wat_rs_defaults()` (`src/rust_deps/mod.rs:169`) currently ships ZERO shims (lru moved to the
`wat-lru` crate); **S1 makes `:rust::sqlite'` the FIRST core default shim** — register it there so the baked surface,
`cargo wat`, and the test suite all resolve it.

## Read the rooms, in order (why each)

1. **`DESIGN-STONE-S1-sqlite-interop.md`** — the full design: the one contract decision (errors-as-values), the
   named surface (the seven verbs + handles, already intueri-cast — do NOT re-name), the build shape, out-of-scope.
2. **`crates/wat-sqlite/src/lib.rs`** — the MECHANISM reference: how a rusqlite `Connection` is wrapped as an opaque
   (`path = ":rust::sqlite::Db"` attribute) and thread-owned. **STUDY it; never `cp`.** It `panic!`s on error — you
   SUPERSEDE that with errors-as-values. Its verbs/surface are the OLD design; yours are the intueri-cast names.
3. **`src/rust_deps/marshal.rs`** — `make_rust_opaque<T: Send+Sync>(type_path, payload) -> Value`,
   `downcast_ref_opaque`, `rust_opaque_arc`, `FromWat`/`ToWat`. This is how you make the `Connection` opaque and
   marshal `Param`/`Cell` (i64/f64/String/nil) in and out.
4. **`src/rust_deps/custodia.rs`** — `ThreadOwnedCell<T: Send>` (`new`/`with_ref`/`with_mut`). A rusqlite `Connection`
   is `Send` but NOT `Sync`; wrap it `ThreadOwnedCell<Connection>` (the cell provides the guarded Send+Sync + the
   thread-id discipline). This is the zero-mutex pattern + the reason the handle is impure (293.W: `:ephemeral`-only).
5. **`src/rust_deps/mod.rs`** — `RustDispatch` (the dispatch fn type + its return), `RustSymbol`, `register_symbol`,
   `register_type`, `with_wat_rs_defaults` (where you register), and `crates/wat-macros/src/lib.rs` for the
   dispatch-generating attribute macro (mirror how wat-sqlite declares its methods).
6. **`wat/query.wat`** (the `Fault`+`Error` shape to MIRROR) — `:wat::query::Error` is a `defenum :wat::enum::Pure`
   with `Transient`/`Constraint`/`Fatal`, each carrying a `Fault {op, code, diagnostic, message}`. Your
   `:wat::sqlite'::Error`/`Fault` mirror it exactly (note `diagnostic`, NOT `sql` — arc-278 intueri-ratified).
7. **`tests/rete/probe_arc278_smem_roundtrip.{wat,rs}`** — the `deftest'` + `.rs`-harness gate shape to mirror.

## The one contract decision — errors are VALUES (never panic)

The `:rust::sqlite'` dispatch fns must **never `panic!`, never `.unwrap()`, never `raise!`**. On failure each yields
a wat **value** carrying `(op-keyword, code-i64, diagnostic-String, message-String)` — a raw fault the wat surface
lifts into `:wat::sqlite'::Error`. Classify rusqlite's primary/extended codes onto the recovery axis in **ONE place**
(the surface or a shim helper), so S2 never re-classifies:
- `SQLITE_BUSY` / `SQLITE_LOCKED` → **`Transient`** (retry)
- `SQLITE_CONSTRAINT` (+ its extended codes) → **`Constraint`** (surface-as-bug)
- everything else (`CANTOPEN`/`CORRUPT`/`MISUSE`/syntax/…) → **`Fatal`** (abort)

The baked `:wat::sqlite'` surface's public methods return `:wat::core::Result<T, :wat::sqlite'::Error>` (mirroring the
`Store` surface). `open-readonly` returns a `ReadConnection` (the capability-honest half — a reader can't write, by type).

## Build order (prove the foundation before the full fan-out)

1. **dep** — add `rusqlite = { version = "0.31", features = ["bundled"] }` to the **root** `Cargo.toml`.
2. **shim, minimum-first** — `src/rust_deps/sqlite.rs`: the `Connection` opaque (`ThreadOwnedCell<Connection>`) + the
   FOUR round-trip verbs `open` · `execute-ddl` · `execute` · `select`, with the **errors-as-values marshaling proven
   on `open` first** (open a good path → a `Connection` opaque; open a bad path like `"/nonexistent/x.db"` → a Fault
   VALUE, no panic). `cargo build` + a throwaway `cargo wat` that opens `":memory:"` and selects `1` must work BEFORE
   you add the rest. THEN `open-readonly` · `pragma` · `begin` · `commit`. Register in `with_wat_rs_defaults`.
3. **surface** — `wat/sqlite.wat` (baked; add the `src/stdlib.rs` bake entry after `wat/core.wat`): the
   `(:wat::core::use! :rust::sqlite'::*)` decl, `:wat::sqlite'::Fault` + `:wat::sqlite'::Error` defenum, and the thin
   surface wrapping the raw verbs + the one-place code→recovery-axis classifier.
4. **gate** — `tests/rete/probe_arc278_sqlite_interop.{wat,rs}` (mirror S-mem.gate): `open ":memory:"` → `execute-ddl`
   a table → `execute` an insert (with `Param`s) → `select` it back (assert the `Cell`s) → force a `Constraint`
   fault (e.g. insert a PK twice → assert `Error::Constraint`). Un-`#[ignore]`'d, GREEN.

## STOP triggers (halt + report — do not improvise)

1. **STOP-PANIC:** no `panic!`/`.unwrap()`/`.expect()`/`raise!` on a sqlite error anywhere in the shim. If you cannot
   marshal an error as a VALUE cleanly (e.g. building the fault Value from Rust is awkward), STOP and report the exact
   blocker — do NOT fall back to panic (that is the OLD wat-sqlite contract we are superseding).
2. **STOP-NAMES:** the surface names are intueri-cast + ratified (Connection/ReadConnection · Param/Cell · open/
   open-readonly/pragma/begin/commit/execute/execute-ddl/select · Fault{op,code,diagnostic,message} · Error). Do NOT
   rename or add verbs. `query` is RESERVED (the higher `:wat::query` engine) — the raw read is `select`.
3. **STOP-THREAD:** the `Connection` is thread-owned (`ThreadOwnedCell`); do NOT make it `Send`-shareable or pool it
   across threads. If a test hits a cross-thread access panic, the fault is the test's scope, not the shim.
4. **STOP-NOCP:** do NOT modify or copy `crates/wat-sqlite` or `crates/wat-telemetry-sqlite` — study them, author
   fresh in core. Do NOT touch `wat/query.wat`/`wat/query/mem.wat`/the `Store` contract.
5. **STOP-CASCADE:** register the shim by adding to `with_wat_rs_defaults`; do NOT thread a new param through the
   composition entry points (harness/compose/test_runner already call `with_wat_rs_defaults`).

## The gate (EXPECTATIONS)

| what | command | expected |
|---|---|---|
| core compiles with rusqlite | `cargo build --release` | clean |
| a raw sqlite round-trip runs | `cargo wat <a scratch .wat opening :memory:, ddl, insert, select>` | prints the selected value |
| the interop gate passes | `cargo nextest run --release -E 'test(sqlite_interop)'` | passed |
| S-mem + S0 still green | `cargo nextest run --release -E 'test(smem_roundtrip) or test(query_contract)'` | passed |
| whole floor | `cargo nextest run --release` | report the Summary line VERBATIM; 0 failed modulo the known `no_inlined_wat` reminder |

## Your final report MUST contain

1. Every file created/changed (paths) + the shim's public-method list + the `:wat::sqlite'` surface/Error shape.
2. How you marshaled errors-as-values (the exact mechanism — this is the load-bearing novelty).
3. The verbatim `sqlite_interop` + `smem_roundtrip` results and the whole-floor Summary line.
4. Any STOP trigger hit, or "no STOP triggers hit".
Your final message IS the return value — raw facts, no ceremony.

## Prior comparable to copy for shape

S-mem.gate (`3304cbd5`, the gate + harness shape) · the extend-type-honesty strike (`fa8bbcb9`, a clean fresh-code
strike weighed to zero-new-failures) · `crates/wat-sqlite` (the opaque+thread-owned mechanism, study-only).
