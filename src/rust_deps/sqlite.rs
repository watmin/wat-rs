//! `:rust::sqlite` — arc 278 stone S1: a FRESH, thread-owned binding to
//! rusqlite, ships as core's FIRST default `:rust::` shim (registered from
//! `with_wat_rs_defaults`, `src/rust_deps/mod.rs`).
//!
//! Study-only precedent: `crates/wat-sqlite/src/lib.rs` (`:rust::sqlite::Db`)
//! shows the opaque + thread-owned shape this mirrors, but it `panic!`s on
//! every rusqlite error. This shim SUPERSEDES that contract: **every fallible
//! dispatch fn returns a wat `Result` — never a panic, never `.unwrap()`,
//! never `raise!`.**
//!
//! # Errors-as-values — the exact mechanism
//!
//! Every fallible method's Rust return type is `Result<T, RawFault>` where
//! [`RawFault`] = `(i64, String, String)` = `(code, diagnostic, message)`.
//! `Result<T, E>` and tuples already have blanket `ToWat`/`FromWat` impls
//! (`src/rust_deps/marshal.rs`), and `#[wat_dispatch]`'s codegen
//! (`crates/wat-macros/src/codegen.rs::rust_type_to_type_expr_tokens`)
//! recognizes `Result<T,E>` and tuple-of-known-primitives natively — so this
//! shim needs ZERO macro changes for any verb, including the two
//! constructors (`open`/`open_readonly`, below), which return plain
//! `Result<Self, RawFault>` / `Result<WatSqliteReadConnection, RawFault>`.
//!
//! `open`/`open_readonly` USED to be the one exception (a hand-built
//! `Value::Result` returning plain `wat::runtime::Value`) because the
//! macro's return-marshal codegen only special-cased a `Self`-wrapping
//! return when the WHOLE return type was literally `Self` —
//! `Result<Self, RawFault>` fell through to the generic
//! `<#ty as ToWat>::to_wat(result)` path, which re-quoted `Self` verbatim
//! into a plain `fn` INSIDE the generated `mod __wat_dispatch_*` (not an
//! impl block), where `Self` doesn't resolve. Fixed at the root (arc 278,
//! the macro-result-self stone): `emit_return_marshal` now has a dedicated
//! `Result<Self, E>` arm (`result_ok_is_self`) that opaque-wraps the `Ok`
//! payload the same way a bare `Self` return does, and `ToWat`s the `Err`
//! payload — never naming `Self` as a type in the generated free fn. So
//! this constructor pair is now a plain dispatch method like any other.
//!
//! `code` is the sqlite **primary** result code (`extended_code & 0xff`,
//! computed here in Rust via `ErrorCode`/`ffi::Error` — the same masking
//! `libsqlite3-sys::Error::new` itself performs) so `wat/sqlite.wat`'s
//! `classify` fn only needs plain equality checks (`= code 5`, `= code 19`,
//! …) — wat has no `i64` modulo intrinsic today, so pre-masking in Rust
//! avoids needing one. Masking to primary code still correctly buckets every
//! `SQLITE_CONSTRAINT` *extended* sub-code (they're `19 | (n << 8)`) onto the
//! same primary `19`, matching the design's "SQLITE_CONSTRAINT + its
//! extended codes -> Constraint" rule.
//!
//! `op` (the ratified `:wat::sqlite::Fault.op` keyword field) is
//! deliberately NOT threaded through this raw 3-tuple: wat's `Tuple` only has
//! `first`/`second`/`third` accessors (index 0-2, no fourth — see
//! `wat/core.wat`'s `format` macro accumulator note), and every
//! `:rust::sqlite::*` dispatch fn already corresponds to exactly one
//! `wat/sqlite.wat` surface verb, which therefore already knows which op
//! faulted — it supplies the `:keyword` literal itself when it lifts this
//! raw fault into `:wat::sqlite::Fault` (see `wat/sqlite.wat::classify`).
//! Net effect on the wire: an open on a bad path yields `Err((code, diag,
//! msg))`, never a panic — proven first, per the build order, before the
//! rest of the verbs were added.
//!
//! # Opaque + thread-owned
//!
//! `rusqlite::Connection` is `Send` but not `Sync`. `WatSqliteConnection`
//! (RW) and `WatSqliteReadConnection` (RO — the capability-honest half; it
//! exposes `select` only, not `execute`/`execute-ddl`/`pragma`/`begin`/
//! `commit`) each wrap one `Connection` and hand-implement [`ToWat`] for
//! non-macro callers, but `open`/`open_readonly` themselves now go through
//! `#[wat_dispatch(scope = "thread_owned")]`'s automatic `Result<Self, E>`
//! marshaling — the macro wraps the `Ok` payload in a [`ThreadOwnedCell`]
//! before opaquing, identically to a bare `Self` return. Downstream `&self`
//! methods (`execute`, `select`, …) go through the same macro-generated
//! `ThreadOwnedCell::with_ref` dispatch, so both halves agree on the wrapper
//! shape.

use std::sync::Arc;

use rusqlite::types::{ToSql, ValueRef};
use rusqlite::{Connection, OpenFlags};

use wat_macros::wat_dispatch;

use crate::rust_deps::{make_rust_opaque, RustDepsBuilder, ThreadOwnedCell, ToWat};
use crate::runtime::{EnumValue, Value};

/// The raw fault payload marshaled across the `:rust::sqlite` boundary on
/// failure: `(code, diagnostic, message)`. See the module doc for why `op`
/// isn't a fourth element here.
pub type RawFault = (i64, String, String);

/// `:rust::sqlite::Connection` — the read/write handle. `Send`, not `Sync`;
/// thread-owned via `ThreadOwnedCell` (see [`ToWat`] impl below).
pub struct WatSqliteConnection {
    conn: Connection,
}

impl ToWat for WatSqliteConnection {
    fn to_wat(self) -> Value {
        make_rust_opaque(":rust::sqlite::Connection", ThreadOwnedCell::new(self))
    }
}

/// `:rust::sqlite::ReadConnection` — the read-only handle (`open-readonly`'s
/// return). A distinct opaque type from `Connection` (not a newtype wrapper
/// at the wat level) — the capability split is enforced BY TYPE: no
/// `execute`/`execute-ddl`/`pragma`/`begin`/`commit` dispatch is registered
/// under this type path, so the checker rejects any attempt to write
/// through a `ReadConnection`.
pub struct WatSqliteReadConnection {
    conn: Connection,
}

impl ToWat for WatSqliteReadConnection {
    fn to_wat(self) -> Value {
        make_rust_opaque(":rust::sqlite::ReadConnection", ThreadOwnedCell::new(self))
    }
}

// ─── shared helpers (used by both Connection and ReadConnection) ──────────

/// Turn a `rusqlite::Error` into a [`RawFault`]. `SqliteFailure` carries the
/// sqlite-native `ffi::Error` (primary `code` + `extended_code`); every other
/// `rusqlite::Error` variant (bad param count, type conversion failure,
/// etc.) has no sqlite-native code, so it gets `code = 0` (falls through
/// `wat/sqlite.wat::classify`'s equality checks to `Fatal` — the safe
/// default for "no sqlite diagnostic to retry/constraint-classify on").
fn fault_from_rusqlite(e: &rusqlite::Error) -> RawFault {
    match e {
        rusqlite::Error::SqliteFailure(ffi_err, _) => (
            (ffi_err.extended_code & 0xff) as i64,
            format!("{e:?}"),
            e.to_string(),
        ),
        other => (0, format!("{other:?}"), other.to_string()),
    }
}

/// Decode one `:wat::sqlite::Param` value (arriving as a generic
/// `Value::Enum` — the type checker already enforced the element type is
/// `:wat::sqlite::Param` before this dispatch fn runs) into a
/// rusqlite-bindable `Box<dyn ToSql>`. A malformed Param (wrong type_path /
/// wrong variant / payload mismatch) is a type-checker-contract violation,
/// not a sqlite error — still returned as a `RawFault` value (code 0, never
/// a panic) rather than trusting the shape and panicking, per this shim's
/// blanket no-panic discipline.
fn param_to_tosql(idx: usize, v: &Value) -> Result<Box<dyn ToSql>, RawFault> {
    let ev = match v {
        Value::Enum(ev) => ev,
        other => {
            return Err((
                0,
                format!("param[{idx}]"),
                format!(
                    "expected :wat::sqlite::Param, got {}",
                    other.type_name()
                ),
            ))
        }
    };
    if ev.type_path != ":wat::sqlite::Param" {
        return Err((
            0,
            format!("param[{idx}]"),
            format!("expected :wat::sqlite::Param, got {}", ev.type_path),
        ));
    }
    match (ev.variant_name.as_str(), ev.fields.first()) {
        ("I64", Some(Value::i64(n))) => Ok(Box::new(*n)),
        ("F64", Some(Value::f64(x))) => Ok(Box::new(*x)),
        ("Str", Some(Value::String(s))) => Ok(Box::new((**s).clone())),
        ("Nil", _) => Ok(Box::new(rusqlite::types::Null)),
        (variant, payload) => Err((
            0,
            format!("param[{idx}]"),
            format!("malformed Param::{variant} (payload {payload:?})"),
        )),
    }
}

fn bind_params(params: &[Value]) -> Result<Vec<Box<dyn ToSql>>, RawFault> {
    params
        .iter()
        .enumerate()
        .map(|(i, v)| param_to_tosql(i, v))
        .collect()
}

/// One read column -> a `:wat::sqlite::Cell` value (a generic `Value::Enum`
/// constructed directly — mirrors how the type checker expects a
/// `:wat::sqlite::Cell` instance to look at runtime). Blob is out of scope
/// (DESIGN-STONE-S1-sqlite-interop.md § out of scope) — degrades to `Nil`
/// rather than dropping the row; a later stone adds a real `Blob` variant if
/// a consumer needs it.
fn cell_to_wat(v: ValueRef) -> Value {
    let (variant, fields): (&str, Vec<Value>) = match v {
        ValueRef::Null => ("Nil", vec![]),
        ValueRef::Integer(i) => ("I64", vec![Value::i64(i)]),
        ValueRef::Real(f) => ("F64", vec![Value::f64(f)]),
        ValueRef::Text(t) => (
            "Str",
            vec![Value::String(Arc::new(String::from_utf8_lossy(t).into_owned()))],
        ),
        ValueRef::Blob(_) => ("Nil", vec![]),
    };
    Value::Enum(Arc::new(EnumValue {
        type_path: ":wat::sqlite::Cell".to_string(),
        variant_name: variant.to_string(),
        names: crate::runtime::builtin_enum_variant_names(":wat::sqlite::Cell", variant),
        fields,
    }))
}

fn execute_impl(conn: &Connection, sql: &str, params: &[Value]) -> Result<i64, RawFault> {
    let bound = bind_params(params)?;
    let mut stmt = conn
        .prepare_cached(sql)
        .map_err(|e| fault_from_rusqlite(&e))?;
    let refs: Vec<&dyn ToSql> = bound.iter().map(|b| b.as_ref()).collect();
    let n = stmt
        .execute(refs.as_slice())
        .map_err(|e| fault_from_rusqlite(&e))?;
    Ok(n as i64)
}

fn select_impl(
    conn: &Connection,
    sql: &str,
    params: &[Value],
) -> Result<Vec<Vec<Value>>, RawFault> {
    let bound = bind_params(params)?;
    let mut stmt = conn
        .prepare_cached(sql)
        .map_err(|e| fault_from_rusqlite(&e))?;
    let refs: Vec<&dyn ToSql> = bound.iter().map(|b| b.as_ref()).collect();
    let col_count = stmt.column_count();
    let mut rows = stmt
        .query(refs.as_slice())
        .map_err(|e| fault_from_rusqlite(&e))?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(|e| fault_from_rusqlite(&e))? {
        let mut cells = Vec::with_capacity(col_count);
        for c in 0..col_count {
            let vr = row.get_ref(c).map_err(|e| fault_from_rusqlite(&e))?;
            cells.push(cell_to_wat(vr));
        }
        out.push(cells);
    }
    Ok(out)
}

// ─── Connection (RW) ────────────────────────────────────────────────────

#[wat_dispatch(path = ":rust::sqlite::Connection", scope = "thread_owned")]
impl WatSqliteConnection {
    /// `:rust::sqlite::Connection::open path` — open or create a sqlite
    /// file at `path`. No pragmas set (the consumer picks its own policy via
    /// `pragma`); no schema install (the consumer calls `execute-ddl`). A
    /// bad path (e.g. a nonexistent directory) yields `Err(RawFault)`, never
    /// a panic — the errors-as-values mechanism proven on this verb first.
    /// `Result<Self, (i64, String, String)>` now marshals automatically
    /// (arc 278 macro-result-self stone): the macro opaque-wraps the `Ok`
    /// payload exactly as a bare `Self` return would, and `ToWat`s the `Err`
    /// tuple.
    //
    // `#[wat_dispatch]`'s codegen inspects the SYNTACTIC return type (a `syn::Type`), not the
    // resolved type — a `RawFault` type-alias name isn't in its known-type list, so every method
    // below spells the literal tuple `(i64, String, String)` instead of the `RawFault` alias
    // (kept for the internal helper fns above, which the macro never sees).
    pub fn open(path: String) -> Result<Self, (i64, String, String)> {
        Connection::open(&path)
            .map(|conn| WatSqliteConnection { conn })
            .map_err(|e| fault_from_rusqlite(&e))
    }

    /// `:rust::sqlite::Connection::execute_ddl conn ddl` — run a DDL string
    /// (CREATE TABLE/INDEX, …) via `execute_batch`. No parameter binding —
    /// for parameterized statements use `execute`.
    pub fn execute_ddl(&self, ddl: String) -> Result<(), (i64, String, String)> {
        self.conn
            .execute_batch(&ddl)
            .map_err(|e| fault_from_rusqlite(&e))
    }

    /// `:rust::sqlite::Connection::execute conn sql params` — run a
    /// parameterized DML statement (INSERT/UPDATE/DELETE). Returns rows
    /// affected. `params` arrives as `Vector<Param>`; each element is
    /// decoded via `param_to_tosql` (never panics on a malformed element —
    /// see its doc).
    pub fn execute(&self, sql: String, params: Vec<Value>) -> Result<i64, (i64, String, String)> {
        execute_impl(&self.conn, &sql, &params)
    }

    /// `:rust::sqlite::Connection::select conn sql params` — the raw read.
    /// Returns one `Vector<Cell>` per row.
    pub fn select(&self, sql: String, params: Vec<Value>) -> Result<Vec<Vec<Value>>, (i64, String, String)> {
        select_impl(&self.conn, &sql, &params)
    }

    /// `:rust::sqlite::Connection::pragma conn name value` — set a pragma
    /// via `conn.pragma_update(None, name, value)`. Substrate is a thin
    /// proxy; the consumer picks its own policy (journal_mode, synchronous,
    /// foreign_keys, …).
    pub fn pragma(&self, name: String, value: String) -> Result<(), (i64, String, String)> {
        self.conn
            .pragma_update(None, name.as_str(), value.as_str())
            .map_err(|e| fault_from_rusqlite(&e))
    }

    /// `:rust::sqlite::Connection::begin conn` — `BEGIN;`. Pairs with
    /// `commit` to wrap a batch in one transaction.
    pub fn begin(&self) -> Result<(), (i64, String, String)> {
        self.conn
            .execute_batch("BEGIN")
            .map_err(|e| fault_from_rusqlite(&e))
    }

    /// `:rust::sqlite::Connection::commit conn` — `COMMIT;`.
    pub fn commit(&self) -> Result<(), (i64, String, String)> {
        self.conn
            .execute_batch("COMMIT")
            .map_err(|e| fault_from_rusqlite(&e))
    }
}

// ─── ReadConnection (RO) ────────────────────────────────────────────────

#[wat_dispatch(path = ":rust::sqlite::ReadConnection", scope = "thread_owned")]
impl WatSqliteReadConnection {
    /// `:rust::sqlite::ReadConnection::open_readonly path` — open an
    /// EXISTING sqlite file read-only (`SQLITE_OPEN_READ_ONLY`); a missing
    /// file or permission failure yields `Err((i64, String, String))`, never
    /// a panic. `Result<Self, (i64, String, String)>` marshals automatically
    /// (arc 278 macro-result-self stone) — see `open`'s doc above.
    pub fn open_readonly(path: String) -> Result<Self, (i64, String, String)> {
        Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map(|conn| WatSqliteReadConnection { conn })
            .map_err(|e| fault_from_rusqlite(&e))
    }

    /// `:rust::sqlite::ReadConnection::select conn sql params` — the raw
    /// read, identical shape to `Connection::select`; a `ReadConnection`
    /// simply has no other verb registered under its type path (the
    /// capability-honest half).
    pub fn select(&self, sql: String, params: Vec<Value>) -> Result<Vec<Vec<Value>>, (i64, String, String)> {
        select_impl(&self.conn, &sql, &params)
    }
}

/// Registrar for `:rust::sqlite` — wires both `Connection` (RW) and
/// `ReadConnection` (RO) dispatch tables. Called from
/// `RustDepsBuilder::with_wat_rs_defaults` (`src/rust_deps/mod.rs`) — the
/// FIRST core default shim (lru was moved OUT to the `wat-lru` crate; sqlite
/// is CORE per the arc-278 ruling).
pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatSqliteConnection::register(builder);
    __wat_dispatch_WatSqliteReadConnection::register(builder);
}
