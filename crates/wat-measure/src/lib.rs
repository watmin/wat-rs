//! `wat-measure` — measurement primitives for wat.
//!
//! Arc 091. Slice 2 ships `:wat::measure::uuid::v4` — a fresh
//! canonical-hyphenated UUID per call. Every measurement scope's
//! identity comes from here; later slices add the `WorkUnit` shape
//! that owns the uuid and the counters/durations attached to it.
//!
//! # Two-part contract (publishable wat crate, per CONVENTIONS.md)
//!
//! - [`wat_sources`] — wat source files (the baked `.wat` wrappers).
//! - [`register`] — Rust shim dispatch + scheme + symbol registration
//!   into the `RustDepsBuilder`.
//!
//! # Using wat-measure from a Rust binary crate
//!
//! ```text
//! // Cargo.toml
//! [dependencies]
//! wat         = { path = "../wat-rs" }
//! wat-measure = { path = "../wat-rs/crates/wat-measure" }
//!
//! // main.rs
//! wat::main! {
//!     source: include_str!("program.wat"),
//!     deps: [wat_measure],
//! }
//!
//! // program.wat
//! (:wat::core::define (:user::main
//!                      (stdin  :wat::io::IOReader)
//!                      (stdout :wat::io::IOWriter)
//!                      (stderr :wat::io::IOWriter)
//!                      -> :())
//!   (:wat::core::let* (((id :String) (:wat::measure::uuid::v4)))
//!     ()))
//! ```
//!
//! # The minting path
//!
//! The shim at `:rust::measure::uuid::v4` calls
//! `wat_edn::new_uuid_v4()` (arc 092) and renders to canonical
//! 8-4-4-4-12 hyphenated hex via `Uuid::to_string`. wat-edn owns
//! the UUID concept end-to-end; wat-measure consumes it.
//!
//! No second `uuid` pin in the workspace.

pub mod shim;
pub mod workunit;

/// wat source files this crate contributes. Slice 2 returns one
/// file under `wat/measure/uuid.wat` — the wat-side wrapper that
/// re-exposes the `:rust::measure::uuid::v4` shim under
/// `:wat::measure::uuid::v4`. Composed in by `wat::main!` /
/// `wat::test!` / `wat::compose_and_run`.
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[
        // types.wat first — Tag + Tags typealiases that WorkUnit.wat
        // and the future Event types reference. Earlier files in
        // wat_sources() are registered before later ones, so
        // typealiases declared here are visible everywhere
        // downstream without explicit load!.
        wat::WatSource {
            path: "wat-measure/measure/types.wat",
            source: include_str!("../wat/measure/types.wat"),
        },
        wat::WatSource {
            path: "wat-measure/measure/uuid.wat",
            source: include_str!("../wat/measure/uuid.wat"),
        },
        wat::WatSource {
            path: "wat-measure/measure/WorkUnit.wat",
            source: include_str!("../wat/measure/WorkUnit.wat"),
        },
    ];
    FILES
}

/// Registrar — wires this crate's Rust shims into the process's
/// `rust_deps::RustDepsRegistry`. Forwards to per-module registers
/// (the v4 free-function shim + the WorkUnit `#[wat_dispatch]`
/// type).
pub fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    shim::register(builder);
    workunit::register(builder);
}
