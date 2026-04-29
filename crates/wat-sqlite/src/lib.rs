//! `wat-sqlite` — sqlite-backed telemetry destination for wat.
//!
//! Arc 083. Two surfaces:
//!
//! - `:wat::sqlite::Db` — thread-owned Rust shim over
//!   `rusqlite::Connection`. Open / execute-ddl / execute-with-params.
//!   Cannot cross thread boundaries (per the substrate's
//!   ThreadOwnedCell discipline); the worker that opens the Db is
//!   the one that uses it.
//!
//! - `:wat::std::telemetry::Sqlite/spawn` — companion to
//!   `:wat::std::telemetry::Console/dispatcher` under arc 080's
//!   service contract. The sqlite-flavored telemetry destination:
//!   spawns a worker that opens a Db, runs substrate's
//!   Service/loop, dispatches each entry through the consumer's
//!   init-fn-built closure.
//!
//! # Slice 0 status
//!
//! Crate scaffold only. Rust shims + wat surfaces ship in slice 1
//! (Db primitives) and slice 2 (Sqlite/spawn).
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

/// wat source files this crate contributes. Slice 0 ships an empty
/// list; slices 1 + 2 add `wat/sqlite/Db.wat` and
/// `wat/std/telemetry/Sqlite.wat` respectively.
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[];
    FILES
}

/// Registrar for wat-sqlite. Slice 0 is a no-op; slice 1 wires the
/// `:rust::wat::sqlite::Db` shim through `#[wat_dispatch]`.
pub fn register(_builder: &mut wat::rust_deps::RustDepsBuilder) {
    // No-op for slice 0.
}
