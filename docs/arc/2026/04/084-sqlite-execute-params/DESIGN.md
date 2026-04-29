# Arc 084 — `:wat::sqlite::execute` with parameter binding

**Status:** PROPOSED 2026-04-28. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 083 slice 1 — `:wat::sqlite::Db` + `open` + `execute-ddl`. The
  parameterized `execute(sql, params)` was specified in arc 083's
  DESIGN slice 1 plan and deferred mid-implementation because `:Any`
  is banned (per memory `feedback_no_new_types`) and the macro's
  `Vec<wat-enum>` shape was unknown.

**Surfaced by:** arc 083 slice 3 (lab migration). The migration plan
called for the lab to switch its dispatcher off `:rust::trading::RunDb`
(typed-method shim wrapping rusqlite) onto `:wat::sqlite::Db`. With
only `execute-ddl` available, every per-row insert in the lab
(paper_resolutions: 10 cols of mixed i64/f64/String; telemetry: 7
cols) would need SQL string concatenation in wat — squashing every
typed value through `to-string`, escaping ourselves, defeating
prepared-statement caching, and reinventing the typing the substrate
already provides at the rusqlite layer.

The user's framing (2026-04-28):

> "we choose - always - simple and honest - you just said something
> isn't honest - so its wrong"

SQL string concat for typed inserts is dishonest. The substrate's
sqlite primitive needs typed parameter binding. Promote the deferred
`execute` from arc 083 to its own arc and ship it.

---

## What this arc is, and is not

**Is:**
- A `:wat::sqlite::Param` enum with four variants — `I64 :i64`,
  `F64 :f64`, `Str :String`, `Bool :bool`. The four scalar shapes
  rusqlite's `ToSql` covers without nuance.
- A `:wat::sqlite::execute db sql params` primitive — runs a
  parameterized statement. `?1`/`?2`/... in `sql` bind positionally
  to `params[0]`/`params[1]`/...
- The Rust shim `WatSqliteDb::execute` that maps each `Value::Enum`
  back to the rusqlite `ToSql` shape and binds.
- A wat-tests entry that asserts a typed round-trip — INSERT with
  a Param vec, then read back via SQLite CLI verification (or, if
  a SELECT primitive surfaces in this arc, via wat).

**Is not:**
- A SELECT primitive. Read-side surface stays out-of-band (sqlite3
  CLI) until a consumer pulls. Same posture as arc 083 slice 1.
- A binder for blobs / dates / nulls. Defer to a follow-up when a
  consumer needs them. The four-variant Param covers paper_resolutions
  + telemetry today; that's the forcing function.
- A compile-time check that `params.len()` matches the placeholder
  count in `sql`. The substrate trusts the caller; rusqlite reports
  mismatches as runtime errors which the shim panics on with a
  diagnostic. A future arc can add a wat-side check by walking the
  SQL string for `?N` placeholders if a caller surfaces a need.

---

## Surface

```scheme
;; The Param enum. PascalCase variants per arc 048's enum convention
;; ("we embody our host language"). Each variant carries one scalar
;; payload — same set rusqlite's ToSql trait covers natively.
(:wat::core::enum :wat::sqlite::Param
  (I64  (n :i64))
  (F64  (x :f64))
  (Str  (s :String))
  (Bool (b :bool)))

;; Execute a parameterized statement. Each `?N` placeholder in `sql`
;; binds to `params[N-1]` (1-indexed per rusqlite/SQLite convention).
;; Panics with a diagnostic on rusqlite errors (placeholder mismatch,
;; constraint violation, syntax errors) — same posture as
;; `execute-ddl`.
(:wat::sqlite::execute
  (db :wat::sqlite::Db)
  (sql :String)
  (params :Vec<wat::sqlite::Param>)
  -> :())
```

### Usage

```scheme
(:wat::sqlite::execute db
  "INSERT INTO events (id, ts, label) VALUES (?1, ?2, ?3)"
  (:wat::core::vec :wat::sqlite::Param
    (:wat::sqlite::Param::I64 7)
    (:wat::sqlite::Param::I64 1730000000000)
    (:wat::sqlite::Param::Str "alpha")))
```

The verbose-but-honest shape from arc 083 DESIGN's Q1: each value
explicitly tagged with its SQLite affinity. rusqlite's `params![]`
macro hides this on the Rust side; wat's discipline is to surface it.

---

## Implementation

### Macro contract — `Vec<wat::runtime::Value>` is sufficient

The `#[wat_dispatch]` macro accepts `wat::runtime::Value` as a
fresh-var-typed parameter (`crates/wat-macros/src/codegen.rs:524`)
and recurses on `T` for `Vec<T>` (`:547`). So a Rust shim with
`pub fn execute(&mut self, sql: String, params: Vec<Value>) -> ()`
generates the type checker's scheme with `:Vec<α>` (α a fresh var).
The wat-side define declares `:Vec<wat::sqlite::Param>` at the call
site; the unifier binds α = `wat::sqlite::Param` per the standard
generic-monomorphization path (precedent: `wat-lru`'s `put(k :K, v :V)`
takes `Value, Value` in Rust).

**No macro work.** Confirmed by reading codegen.rs lines 524-558.

### Rust shim — runtime extraction of Param variants

The Rust body iterates `params: Vec<Value>`, matches each on
`Value::Enum(EnumValue { type_path, variant_name, fields })`,
asserts `type_path == ":wat::sqlite::Param"`, and dispatches on
`variant_name`:
- `"I64"` — `fields[0]` extracted as `Value::I64(n)`, bound as i64.
- `"F64"` — `fields[0]` as `Value::F64(x)`, bound as f64.
- `"Str"` — `fields[0]` as `Value::Str(s)`, bound as &str.
- `"Bool"` — `fields[0]` as `Value::Bool(b)`, bound as bool.

Mismatches panic with the wat-rs panic-vs-Option discipline (per
memory `feedback_shim_panic_vs_option`): construction-time
assertions panic with a diagnostic; the shim trusts the type
checker to have caught wat-side type errors before reaching runtime.

The bindings flow into `rusqlite`'s `params_from_iter` (or the
manual `Statement::raw_bind_parameter` loop) — whichever the
existing `execute_ddl` precedent uses for the simplest call shape.

---

## Slice plan

### Slice 1 — Param enum + execute primitive

`crates/wat-sqlite/src/lib.rs`:
- `pub enum Param { I64(i64), F64(f64), Str(String), Bool(bool) }`
- `#[wat_dispatch_enum]` (or whatever variant macro is needed — TBD
  during implementation; if there's no enum-shim macro, the wat-side
  enum decl is enough since the Rust side only does runtime
  extraction from `Value::Enum`).
- `pub fn execute(&mut self, sql: String, params: Vec<Value>) -> ()`
  with the runtime extraction loop above.

`crates/wat-sqlite/wat/sqlite/Db.wat`:
- `(:wat::core::enum :wat::sqlite::Param ...)` declaration (4
  variants).
- `(:wat::core::define (:wat::sqlite::execute db sql params -> :()) ...)`
  thin wrapper.

### Slice 2 — Tests

`crates/wat-sqlite/wat-tests/sqlite/Db.wat` — extend the existing
slice-1 file with one new deftest:
- `test-execute-params` — open → execute-ddl creates table →
  execute INSERT with one of each Param variant → verify rows by
  re-querying via `execute-ddl` with side-channel assertions (or
  out-of-band sqlite3 CLI per the slice-1 pattern).

### Slice 3 — INSCRIPTION

`docs/arc/2026/04/084-sqlite-execute-params/INSCRIPTION.md` —
shipped surface, what the slice-1 deferral cost (one arc and one
session of friction), and the lab-migration unblock signal.

---

## Open questions

### Q1 — `Param` as a wat-decl enum vs a Rust-shim enum

Two ways to expose the four-variant type to wat:
- **(a) Wat-decl enum** (proposed): `(:wat::core::enum :wat::sqlite::Param ...)`
  in the Db.wat surface. The Rust shim accepts `Vec<Value>` and
  extracts `Value::Enum` at runtime. Variants exist on the wat side;
  the Rust side reads them as data.
- **(b) Rust shim enum + auto-derived wat surface.** Add `Param` to
  Rust with `#[wat_dispatch_enum]` (if such a macro exists; would
  need to check). The macro emits the wat-side enum decl + per-variant
  constructors; the Rust shim takes `Vec<Param>` directly.

**Default: (a).** Matches the wat-lru precedent (Rust takes Value;
wat-side declares typed surface). One of the four variants would
need duplicate decl (Rust enum + wat enum) under (b); (a) keeps the
single source of truth on the wat side, where the Param decl lives
next to the `execute` callsite and stays grep-able from the same
file users read.

### Q2 — Should rusqlite's NULL surface as a fifth variant?

Today's lab callers don't write NULLs. rusqlite supports NULL via
`Option<T>` in `ToSql`; the wat counterpart would be a fifth Param
variant `Null` or a `Param::Null`. Defer until a consumer surfaces
a need for nullable columns.

### Q3 — Prepared-statement caching

rusqlite caches prepared statements by SQL text on each `Connection`.
The shim's `execute` should call `Connection::prepare_cached` rather
than `prepare` to get the cache benefit — same SQL text reused
across calls hits the cache. No wat-side surface change; just the
implementation choice in the Rust shim.

---

## Test strategy

- Slice 1: cargo build clean.
- Slice 2: one new deftest exercising all four Param variants
  end-to-end. Existing slice-1 tests stay green.
- Slice 3: docs only.

---

## Dependencies

**Upstream:** Arc 083 slice 1 (the `:wat::sqlite::Db` shim and the
`execute-ddl` precedent). Arc 048 (user-defined enum values — the
construction syntax `(:wat::sqlite::Param::I64 n)`).

**Downstream this arc unblocks:**
- Arc 083 slice 3 (lab migration off `:rust::trading::RunDb`) — the
  forcing function that surfaced this gap.
- Every future consumer of `:wat::std::telemetry::Sqlite/spawn`
  that wants typed inserts — MTG, truth-engine, anything else.

PERSEVERARE.
