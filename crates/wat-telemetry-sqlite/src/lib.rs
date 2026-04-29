//! `wat-telemetry-sqlite` — sqlite-backed telemetry sink for wat.
//!
//! Arc 096. Composes the two underlying crates:
//!
//! - [`wat_telemetry`] provides the generic Service<E,G> shell —
//!   the queue-fronted destination service with paired-channel
//!   batch+ack discipline (arc 089 + arc 095). The `:wat::telemetry::*`
//!   namespace.
//!
//! - [`wat_sqlite`] provides `:wat::sqlite::Db` — the thread-owned
//!   sqlite handle with open/execute-ddl/execute/pragma/begin/commit.
//!   Pure plumbing; no telemetry awareness.
//!
//! This crate combines them into ONE specific sink:
//!
//! - `:wat::telemetry::Sqlite/spawn` — explicit two-hook (pre-install +
//!   schema-install + dispatcher) factory. The consumer's hook
//!   declares pragmas and DDL, then per-batch dispatches each
//!   entry through their own `:fn(Db,Vec<E>)->()` closure.
//!
//! - `:wat::telemetry::Sqlite/auto-spawn` — enum-derived schema.
//!   Consumer declares an enum (per arc 085); the substrate walks
//!   variants, builds CREATE TABLEs + cached INSERTs, dispatches
//!   per-entry through the auto-derived path. Five lines of wat
//!   to migrate a domain off ad-hoc SQL.
//!
//! - The three Rust shims at `:rust::sqlite::auto-{prep,
//!   install-schemas, dispatch}` — moved here from wat-sqlite
//!   per arc 096 (they're telemetry-specific; wat-sqlite stays
//!   general-purpose).
//!
//! # Two-part contract
//!
//! - [`wat_sources`] — the baked `wat/telemetry/Sqlite.wat`.
//! - [`register`] — wires the auto-{prep,install,dispatch} Rust
//!   shims into the deps builder.

mod auto;

pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[
        wat::WatSource {
            path: "wat-telemetry-sqlite/telemetry/Sqlite.wat",
            source: include_str!("../wat/telemetry/Sqlite.wat"),
        },
    ];
    FILES
}

pub fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    auto::register(builder);
}
