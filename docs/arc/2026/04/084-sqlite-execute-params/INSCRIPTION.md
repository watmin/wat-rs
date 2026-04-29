# Arc 084 — `:wat::sqlite::execute` with parameter binding — INSCRIPTION

**Status:** shipped 2026-04-28. Same-session follow-on to arc 083.

The deferral arc 083 slice 1 carried — "execute(sql, params) ships
in a follow-up slice once the `:wat::sqlite::Param` enum + the
macro's `Vec<wat-enum>` shape settle" — was the friction we hit
trying to walk arc 083 slice 3 (lab migration). With only
`execute-ddl`, the lab dispatcher's only path to typed inserts was
SQL string concatenation in wat. The user named that dishonest, so
it was wrong; the deferred work got promoted to its own arc and
shipped immediately.

Three durables:

1. **`:wat::sqlite::Param` enum with four scalar variants** — `I64`,
   `F64`, `Str`, `Bool`. Each variant carries one payload of the
   matching scalar shape. The four variants match rusqlite's `ToSql`
   coverage without nuance. Future arcs can add `Null` / `Blob` /
   `Date` when a consumer surfaces a need.
2. **`:wat::sqlite::execute db sql params`** — runs a parameterized
   statement. `?N` placeholders bind to `params[N-1]` (1-indexed per
   rusqlite/SQLite). Uses `prepare_cached` so repeated calls with the
   same SQL text reuse rusqlite's prepared-statement cache.
3. **The macro already supported `Vec<Value>`.** Confirmed by
   reading `crates/wat-macros/src/codegen.rs:524` (`Value` →
   `ctx.fresh_var()`) and `:547` (`Vec<T>` → recurse on `T`). No
   macro work; the wat-side `:Vec<wat::sqlite::Param>` declaration
   is enforced by the type checker before reaching the shim, which
   accepts `Vec<Value>` and extracts `Value::Enum` at runtime per
   the wat-lru precedent.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped

### Slice 1 — Param enum + execute primitive

`crates/wat-sqlite/src/lib.rs`:
- New `pub fn execute(&mut self, sql: String, params: Vec<Value>)`
  on `WatSqliteDb`. Iterates params, calls
  `param_value_to_tosql(idx, sql, v)` per element, collects
  `Box<dyn ToSql>`, calls `prepare_cached(sql)`, runs
  `Statement::execute(&[&dyn ToSql])`. Panics with positional
  diagnostics on rusqlite errors (placeholder mismatch / constraint
  violation / syntax) per the panic-vs-Option discipline.
- New `fn param_value_to_tosql(idx, sql, v)` helper. Pattern-matches
  on `Value::Enum` with `type_path == ":wat::sqlite::Param"`,
  dispatches on `variant_name` ("I64" / "F64" / "Str" / "Bool"),
  returns the appropriate `Box<dyn ToSql>`. Type-checker
  contract violations panic with a diagnostic naming the position
  in the Vec and the SQL text.

`crates/wat-sqlite/wat/sqlite/Db.wat`:
- New `(:wat::core::enum :wat::sqlite::Param ...)` declaration with
  four tagged variants.
- New `(:wat::core::define (:wat::sqlite::execute ...) ...)` thin
  wrapper forwarding to `:rust::sqlite::Db::execute`.

The deferral comment block in `lib.rs` and `Db.wat` (the
"acceptable for internal-typed values; SQL injection isn't a
concern when all values come from typed programmatic sources"
posture) replaced with the live primitive.

### Slice 2 — Tests

`crates/wat-sqlite/wat-tests/sqlite/Db.wat`:
- New `test-execute-params` deftest exercising one of each Param
  variant. Inserts a row with `Str "alpha-run"`, `I64 42`, `F64
  0.125`, `Bool true`. Round-trip verified out-of-band:
  ```
  $ sqlite3 /tmp/wat-sqlite-test-003.db \
      'SELECT run_name, paper_id, residue, ok, typeof(run_name), typeof(paper_id), typeof(residue), typeof(ok) FROM rows'
  alpha-run|42|0.125|1|text|integer|real|integer
  ```
  Each value bound with the correct SQLite type affinity. Bool
  stores as integer (0/1) per rusqlite/SQLite convention — SQLite
  has no native bool type.

5 of 5 wat-sqlite tests green (3 Db + 2 Sqlite). Workspace stays
clean: 728 substrate Rust tests + every other wat-suite green.

### Slice 3 — INSCRIPTION (this file)

---

## What's still uncovered

- **Null binding.** No `Param::Null` variant. Today's lab forcing
  function (paper_resolutions: 10 NOT NULL columns; telemetry: 7
  NOT NULL columns) doesn't write nulls. A future arc adds the
  variant when a consumer needs nullable columns.
- **Read-side primitive.** No `query` / `prepare` / `select`
  surface; reads still happen out-of-band via sqlite3 CLI per the
  arc 083 slice 1 posture. A future arc adds it when a consumer
  needs in-test row counting.
- **Placeholder-count check at the wat layer.** `params.len()`
  vs `?N` count in `sql` is checked by rusqlite at bind time (and
  surfaces as a panic via the shim's diagnostic). A wat-side check
  would parse the SQL string for `?N`s; not worth it until a
  caller surfaces a need.

## Cost of the deferral

The arc 083 slice 1 deferral cost one full session of friction:
- arc 083 slice 2 shipped substrate `:wat::std::telemetry::Sqlite/spawn`
  — works correctly.
- arc 083 slice 3 (lab migration) hit the `Db` type-mismatch wall
  the moment we tried to walk it (substrate opens
  `:wat::sqlite::Db`; lab dispatcher requires `:trading::rundb::RunDb`).
- The migration has only one honest path: switch to substrate Db,
  which means typed inserts via `execute(sql, params)`, which the
  deferral didn't ship.

The lesson — already captured in memory `feedback_absence_is_signal`
— is reinforced: when the language LACKS something a downstream
slice needs, the language work IS the slice. Deferral pushed the
friction one slice forward; the cost was real.

## Consumer impact

Unblocks:
- **Arc 083 slice 3** (lab migration off `:rust::trading::RunDb`).
- Every future consumer of `:wat::std::telemetry::Sqlite/spawn`
  that wants typed inserts — MTG, truth-engine, future cross-domain
  experiments. They write inserts in the verbose-but-honest
  `:Vec<wat::sqlite::Param>` shape; the substrate handles the
  binding correctly with zero per-consumer reinvention.

PERSEVERARE.
