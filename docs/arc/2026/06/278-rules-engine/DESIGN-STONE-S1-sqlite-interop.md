# DESIGN — S1: the `:wat::sqlite'` RAW interop (the heaviest stone — fresh Rust)

> Part of the sqlite → telemetry → rete build (`DESIGN-sqlite-core.md`). S1 is the RAW layer **below** the
> backend-agnostic `Store` abstraction; **S2** builds the `:wat::sqlite'::Connection`→`:wat::query::Store`
> satisfier on top of it, differential-tested against the **S-mem.gate MemStore oracle** (now green, `3304cbd5`).

## Why

The `Store` contract is backend-agnostic; sqlite is the first *real* driver. S1 gives wat a thin, honest,
**fresh** binding to rusqlite — connections, params, cells, and the seven raw verbs — with errors as VALUES (never
panics), so S2 can implement `ensure-schema`/`put`/`scan`/`scan-index` as SQL over it. The `wat-sqlite` /
`wat-telemetry-sqlite` crates are HINTS for the *mechanism* only — **build fresh in core `src/`, never `cp`**
(their foundation predates the current substrate; the interop names are already intueri-cast, below).

## The mechanism (grounded, file:line)

- **`:rust::` shims register via `RustDepsBuilder`** (`src/rust_deps/mod.rs`): each shim registers, per
  method-path keyword, a `RustDispatch` fn + a `TypeScheme` (for the checker) + Value↔Rust marshaling
  (`marshal.rs` — `make_rust_opaque`/`FromWat`/`ToWat`). The **`lru` shim is the register-pattern exemplar**
  (`with_wat_rs_defaults` ships it in core); the sqlite shim joins core the same way.
- **Opaque, thread-owned handles.** A rusqlite `Connection` is NOT `Send`/`Sync`. It rides as a `make_rust_opaque`
  handle guarded by **`ThreadOwnedCell`** (`src/rust_deps/custodia.rs`, the zero-mutex thread-id-guard pattern —
  the same one `lru::LruCacheCell` uses). This is the CONTRACT-level reason the `Connection` is impure and
  **293.W-contained**: it can only live in a struct/`:ephemeral`, never durable/wire — exactly where S2's Store
  satisfier holds it.
- **`crates/wat-sqlite/src/lib.rs`** (`:wat::sqlite::Db`, arc-083/096) — STUDY it for the thread-owned-Connection +
  opaque-handle shape; do NOT copy it. Its surface (`Db`, its verbs) is the OLD design we supersede.
- rusqlite `0.31` (`features=["bundled"]`) is a dep in the two crates but **NOT in the root** — S1 adds it to the
  root `Cargo.toml`.

## The one contract decision — errors are VALUES at the `:rust::` boundary; the `:wat::sqlite'::Error` enum is the surface

The `:rust::sqlite'` dispatch fns **never panic and never `raise!`**: each returns a wat **value** that is either the
success payload or a **fault value** carrying `(op, code, sqlstate/extended-code, message)`. The baked
`:wat::sqlite'` surface lifts that into the ratified **recovery-axis enum** — mirroring `:wat::query::Error`:

```clojure
(:wat::core::defrecord :wat::sqlite'::Fault [op <- :keyword  code <- :i64  diagnostic <- :String  message <- :String])
(:wat::core::defenum :wat::sqlite'::Error :wat::enum::Pure
  :Transient  [fault <- :wat::sqlite'::Fault]     ;; SQLITE_BUSY/LOCKED — retry
  :Constraint [fault <- :wat::sqlite'::Fault]     ;; SQLITE_CONSTRAINT — surface
  :Fatal      [fault <- :wat::sqlite'::Fault])    ;; CORRUPT/CANTOPEN/… — abort
;; classification: map rusqlite's primary/extended codes onto the recovery axis in ONE place (the shim or the
;; surface), so S2 never re-classifies. Note `diagnostic` (not `sql`) — the intueri-ratified field name (7abd0f07).
```

Rejected: `raise!`-on-error (a store fault is a *value the caller branches on*, not an exception — 299 Effectful
boundary); reusing the OLD `:wat::sqlite::Db` verbs (pre-current-substrate); a Send-able Connection (impossible +
293.W-wrong).

## The named surface (intueri-cast, from PROBANDO STRVIMVS — do NOT re-cast)

`:wat::sqlite'` verbs + handles, all ratified:

| handle/verb | shape | note |
|---|---|---|
| `Connection` / `ReadConnection` | opaque, thread-owned | RW vs RO — the capability split (a reader cannot write, by type) |
| `Param` / `Cell` | marshaled value in / value out | bind params; read result cells (i64/f64/String/blob/nil) |
| `open` / `open-readonly` | `String path -> Result<Connection,Error>` | `open-readonly` yields a `ReadConnection` |
| `pragma` | `Connection, String -> Result<nil,Error>` | e.g. `journal_mode=WAL`, `foreign_keys=ON` |
| `begin` / `commit` | `Connection -> Result<nil,Error>` | explicit txn (S2's `put` is one atomic batch) |
| `execute` | `Connection, String, Vector<Param> -> Result<i64,Error>` | DML; returns rows-affected |
| `execute-ddl` | `Connection, String -> Result<nil,Error>` | CREATE TABLE/INDEX (S2's `ensure-schema`) |
| `select` | `Connection/ReadConnection, String, Vector<Param> -> Result<Vector<Vector<Cell>>,Error>` | the raw read; **`query` is reserved** for the higher `:wat::query` engine — a promise, not a collision |

## The build shape (the strike order within S1)

1. **dep** — add `rusqlite = { version = "0.31", features = ["bundled"] }` to the root `Cargo.toml`.
2. **shim** — `src/rust_deps/sqlite.rs` (fresh): the `:rust::sqlite'` opaque handles (Connection/ReadConnection via
   `ThreadOwnedCell<Connection>` — `Connection` is `Send` not `Sync`, the cell provides the guarded Send+Sync;
   `make_rust_opaque`), Param/Cell marshaling (`marshal.rs` FromWat/ToWat), the seven verbs as dispatch fns
   returning fault-VALUES; register it inside **`with_wat_rs_defaults()`** (`src/rust_deps/mod.rs:169`).
   **GROUNDED (the ruling): sqlite is CORE.** `with_wat_rs_defaults` currently ships ZERO shims (lru was moved to
   the `wat-lru` crate for leanness); S1 makes `:rust::sqlite'` the **FIRST core default shim** — so the baked
   `wat/sqlite.wat` surface, `cargo wat`, and the whole test suite all resolve it. rusqlite joins bigint as a hard
   core dep (`query` is backed by memory OR sqlite, both first-class core; other backends come later as external
   crates). The SHIM MECHANISM exemplar is `crates/wat-sqlite/src/lib.rs` (the `path = ":rust::sqlite::Db"` opaque +
   thread-owned Connection pattern) — **study it, never cp**, and supersede its `panic!`-on-error with errors-as-values.
3. **surface** — `wat/sqlite.wat` (baked, loads after `wat/core.wat`): `:wat::sqlite'::Fault` + `Error` defenum +
   the thin `:wat::sqlite'` wat surface that wraps the `:rust::sqlite'` verbs and lifts fault-values → `Error`.
4. **gate** — a co-located `deftest'` (mirroring S-mem.gate): `open` in-memory (`":memory:"`) → `execute-ddl` a
   table → `execute` an insert (params) → `select` it back → assert the round-trip + one forced fault (e.g. a
   constraint violation → `Error::Constraint`).

## Out of scope (rejected — named, not deferred)

The `Store` satisfier (S2). Connection pooling / multi-thread sharing (thread-owned by design; a per-thread open is
S2's concern). Async/WAL-tuning beyond a `pragma` passthrough. Prepared-statement caching (a later perf stone if a
consumer measures it). Blob streaming (S2 stores opaque EDN `String` data — `Cell` covers `String`/`i64`/`f64`/nil;
blob can wait for a real consumer).

## Blast radius

Root `Cargo.toml` (+1 dep) · `src/rust_deps/sqlite.rs` (new) + its one-line registration in `rust_deps/mod.rs`'s
`with_wat_rs_defaults` · `wat/sqlite.wat` (new baked source) + its `src/stdlib.rs` bake entry · the co-located gate.
No change to the `Store` contract (`wat/query.wat`), the MemStore, or `rusqlite`'s crate consumers.
