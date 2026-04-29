//! `wat-sqlite` — sqlite-backed telemetry destination for wat.
//!
//! Arc 083. Two surfaces:
//!
//! - `:wat::sqlite::Db` — thread-owned Rust shim over
//!   `rusqlite::Connection`. Open / execute-ddl / execute (with
//!   parameter binding). Cannot cross thread boundaries (per the
//!   substrate's ThreadOwnedCell discipline); the worker that opens
//!   the Db is the one that uses it.
//!
//! - `:wat::std::telemetry::Sqlite/spawn` — companion to
//!   `:wat::std::telemetry::Console/dispatcher` under arc 080's
//!   service contract. The sqlite-flavored telemetry destination:
//!   spawns a worker that opens a Db, runs substrate's
//!   Service/loop, dispatches each entry through the consumer's
//!   init-fn-built closure. (Slice 2.)
//!
//! # Using from a Rust binary crate
//!
//! ```text
//! // Cargo.toml
//! [dependencies]
//! wat        = { path = "../wat-rs" }
//! wat-sqlite = { path = "../wat-rs/crates/wat-sqlite" }
//!
//! // main.rs
//! wat::main! {
//!     source: include_str!("program.wat"),
//!     deps: [wat_sqlite],
//! }
//! ```

use rusqlite::types::ToSql;
use rusqlite::Connection;
use wat::runtime::Value;
use wat_macros::wat_dispatch;

mod auto;

/// `:rust::sqlite::Db` — thread-owned SQLite handle.
///
/// Wraps `rusqlite::Connection`. Single-thread discipline: open in
/// the worker that will use it; pass the path across threads, not
/// the Db. Per the substrate's panic-vs-Option contract: open
/// panics on bad path / permission; per-call writes panic on
/// rusqlite errors (a future arc may switch to `Result<()>` once
/// a consumer wants graceful error handling).
pub struct WatSqliteDb {
    pub(crate) conn: Connection,
}

#[wat_dispatch(
    path = ":rust::sqlite::Db",
    scope = "thread_owned"
)]
impl WatSqliteDb {
    /// `:rust::sqlite::Db::open path` — open or create a sqlite
    /// file at `path`. No pragmas are set; the substrate refuses to
    /// pick a journal_mode / synchronous policy on the consumer's
    /// behalf. Use `:rust::sqlite::Db::pragma` after open to set
    /// whatever policy the consumer wants. Schema install is the
    /// consumer's job (call `execute_ddl` afterward). Panics on
    /// rusqlite errors.
    pub fn open(path: String) -> Self {
        let conn = Connection::open(&path).unwrap_or_else(|e| {
            panic!(":rust::sqlite::Db::open: cannot open {path}: {e}")
        });
        Self { conn }
    }

    /// `:rust::sqlite::Db::execute-ddl db ddl` — run a DDL
    /// string (CREATE TABLE, CREATE INDEX, etc.) via execute_batch.
    /// Idempotent when the DDL uses `IF NOT EXISTS`. No parameter
    /// binding — for parameterized statements use `execute`.
    pub fn execute_ddl(&mut self, ddl: String) {
        self.conn.execute_batch(&ddl).unwrap_or_else(|e| {
            panic!(":rust::sqlite::Db::execute-ddl: {e}")
        });
    }

    /// `:rust::sqlite::Db::execute db sql params` — run a parameterized
    /// statement. Each `?N` placeholder in `sql` binds positionally to
    /// `params[N-1]` (1-indexed per SQLite/rusqlite convention).
    ///
    /// `params` is wat-side typed as `:Vec<wat::sqlite::Param>`; the
    /// type checker enforces every element is a Param variant before
    /// reaching this shim, so the runtime extraction below trusts the
    /// shape and panics with a diagnostic on any deviation (treated
    /// as type-checker-bug / programmer-error per the panic-vs-Option
    /// discipline).
    ///
    /// Uses `prepare_cached` so repeated calls with the same SQL text
    /// reuse rusqlite's prepared-statement cache — important for the
    /// service-batch workloads this primitive was forced into shape
    /// for (340+ inserts per proof in the lab's existing pattern).
    pub fn execute(&mut self, sql: String, params: Vec<Value>) {
        // Map each Value::Enum payload into a rusqlite-bindable scalar.
        // The closure on each variant returns Box<dyn ToSql> so we can
        // collect them into a single Vec without intermediate copies of
        // the underlying String / i64 / f64 / bool.
        let bound: Vec<Box<dyn ToSql>> = params
            .into_iter()
            .enumerate()
            .map(|(idx, v)| param_value_to_tosql(idx, &sql, v))
            .collect();
        let mut stmt = self.conn.prepare_cached(&sql).unwrap_or_else(|e| {
            panic!(":rust::sqlite::Db::execute: prepare {sql:?}: {e}")
        });
        let refs: Vec<&dyn ToSql> = bound.iter().map(|b| b.as_ref()).collect();
        stmt.execute(refs.as_slice()).unwrap_or_else(|e| {
            panic!(":rust::sqlite::Db::execute: bind/exec {sql:?}: {e}")
        });
    }

    /// `:rust::sqlite::Db::pragma db name value` — set a pragma via
    /// `conn.pragma_update(None, name, value)`. Substrate is a thin
    /// proxy to rusqlite; consumers pick their own policy. The `value`
    /// is a String (rusqlite's `&str` ToSql renders correctly for
    /// SQLite's pragma syntax — bare or quoted). Examples:
    ///
    /// ```text
    /// (:wat::sqlite::Db::pragma db "journal_mode" "WAL")
    /// (:wat::sqlite::Db::pragma db "synchronous" "NORMAL")
    /// (:wat::sqlite::Db::pragma db "cache_size" "10000")
    /// (:wat::sqlite::Db::pragma db "foreign_keys" "ON")
    /// ```
    ///
    /// Read form (`pragma_query`) deferred — add when a consumer needs it.
    pub fn pragma(&mut self, name: String, value: String) {
        self.conn
            .pragma_update(None, name.as_str(), value.as_str())
            .unwrap_or_else(|e| {
                panic!(":rust::sqlite::Db::pragma: {name}={value}: {e}")
            });
    }

    /// `:rust::sqlite::Db::begin db` — `BEGIN;`. Pairs with
    /// `commit` to wrap a batch of inserts in one transaction (one
    /// fsync for the whole batch instead of per-row auto-commit).
    /// The archive's `flush()` discipline at
    /// `archived/pre-wat-native/src/programs/stdlib/database.rs:224-231`.
    pub fn begin(&mut self) {
        self.conn.execute_batch("BEGIN").unwrap_or_else(|e| {
            panic!(":rust::sqlite::Db::begin: {e}")
        });
    }

    /// `:rust::sqlite::Db::commit db` — `COMMIT;`. Closes a
    /// transaction opened by `begin`. On error, the panic surfaces
    /// the rusqlite diagnostic; the caller is expected to wrap
    /// inserts in begin/commit pairs at the per-batch boundary.
    pub fn commit(&mut self) {
        self.conn.execute_batch("COMMIT").unwrap_or_else(|e| {
            panic!(":rust::sqlite::Db::commit: {e}")
        });
    }

}

/// Map one `:wat::sqlite::Param::*` Value into a rusqlite-bindable
/// boxed `ToSql`. Panics with a positional diagnostic if the value
/// isn't a Param-shape — that's a type-checker contract violation,
/// not a runtime input error.
fn param_value_to_tosql(idx: usize, sql: &str, v: Value) -> Box<dyn ToSql> {
    let ev = match &v {
        Value::Enum(ev) => ev.clone(),
        _ => panic!(
            ":rust::sqlite::Db::execute: param[{idx}] in {sql:?}: \
             expected :wat::sqlite::Param, got {}",
            v.type_name()
        ),
    };
    if ev.type_path != ":wat::sqlite::Param" {
        panic!(
            ":rust::sqlite::Db::execute: param[{idx}] in {sql:?}: \
             expected :wat::sqlite::Param, got {}",
            ev.type_path
        );
    }
    match (ev.variant_name.as_str(), ev.fields.first()) {
        ("I64", Some(Value::i64(n))) => Box::new(*n),
        ("F64", Some(Value::f64(x))) => Box::new(*x),
        ("Str", Some(Value::String(s))) => Box::new((**s).clone()),
        ("Bool", Some(Value::bool(b))) => Box::new(*b),
        (variant, payload) => panic!(
            ":rust::sqlite::Db::execute: param[{idx}] in {sql:?}: \
             malformed Param::{variant} (payload {payload:?})"
        ),
    }
}

// ─── Crate registrar ────────────────────────────────────────────

/// wat source files this crate contributes.
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[
        wat::WatSource {
            path: "wat-sqlite/sqlite/Db.wat",
            source: include_str!("../wat/sqlite/Db.wat"),
        },
        wat::WatSource {
            path: "wat-sqlite/std/telemetry/Sqlite.wat",
            source: include_str!("../wat/std/telemetry/Sqlite.wat"),
        },
    ];
    FILES
}

/// Registrar for wat-sqlite. Wires the `:rust::sqlite::Db` shim
/// (open / execute-ddl / execute) through `#[wat_dispatch]`'s
/// generated registration code, plus the arc-085 auto-spawn shims
/// (auto-prep / auto-install-schemas / auto-dispatch) registered
/// hand-written because they need direct `sym.types` access.
pub fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    __wat_dispatch_WatSqliteDb::register(builder);
    auto::register(builder);
}
