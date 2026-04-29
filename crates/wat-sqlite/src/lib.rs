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

use rusqlite::Connection;
use wat_macros::wat_dispatch;

/// `:rust::sqlite::Db` — thread-owned SQLite handle.
///
/// Wraps `rusqlite::Connection`. Single-thread discipline: open in
/// the worker that will use it; pass the path across threads, not
/// the Db. Per the substrate's panic-vs-Option contract: open
/// panics on bad path / permission; per-call writes panic on
/// rusqlite errors (a future arc may switch to `Result<()>` once
/// a consumer wants graceful error handling).
pub struct WatSqliteDb {
    conn: Connection,
}

#[wat_dispatch(
    path = ":rust::sqlite::Db",
    scope = "thread_owned"
)]
impl WatSqliteDb {
    /// `:rust::sqlite::Db::open path` — open or create a sqlite
    /// file at `path`. Schema install is the consumer's job (call
    /// `execute_ddl` afterward). Panics on rusqlite errors.
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

}

// Note: a parameterized `execute(sql, params)` primitive ships in a
// follow-up slice. The wat type system bans `:Any` per 058-030; the
// param-binding surface needs a typed enum (`:wat::sqlite::Param`
// with I64/F64/Str/Bool variants) plus macro support for
// `Vec<wat-enum>`. Slice 1 ships open + execute-ddl, which is
// enough to install schemas. Consumers writing rows use SQL string
// concat for now (acceptable for internal-typed values; SQL
// injection isn't a concern when all values come from typed
// programmatic sources).

// ─── Crate registrar ────────────────────────────────────────────

/// wat source files this crate contributes.
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[wat::WatSource {
        path: "wat-sqlite/sqlite/Db.wat",
        source: include_str!("../wat/sqlite/Db.wat"),
    }];
    FILES
}

/// Registrar for wat-sqlite. Wires the `:rust::sqlite::Db`
/// shim through `#[wat_dispatch]`'s generated registration code.
pub fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    __wat_dispatch_WatSqliteDb::register(builder);
}
