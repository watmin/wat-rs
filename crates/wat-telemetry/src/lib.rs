//! `wat-telemetry` — telemetry primitives for wat.
//!
//! Arc 096. Owns the `:wat::telemetry::*` namespace. Slice 1 ships
//! the queue-fronted Service<E,G> shell + the Console driver +
//! ConsoleLogger; slices 2-3 fold WorkUnit, Event, and the
//! sqlite-backed sink (the latter into a sibling
//! `wat-telemetry-sqlite` crate).
//!
//! Pre-arc-096, telemetry plumbing lived under
//! `:wat::std::telemetry::*` in the substrate and `:wat::measure::*`
//! in a separate `wat-measure` crate. The arc-096 recognition:
//! measurement IS telemetry; the split was artificial. One
//! namespace, one crate, one mental model.
//!
//! # Two-part contract (per CONVENTIONS.md "publishable wat crate")
//!
//! - [`wat_sources`] — the baked `.wat` files containing the
//!   `:wat::telemetry::*` typealiases, structs, enums, and
//!   wat-side functions (Service<E,G>, Console driver,
//!   ConsoleLogger).
//! - [`register`] — registers any Rust shims into the deps
//!   builder. Slice 1 has no shims (everything is wat-only);
//!   slice 2 adds the WorkUnit thread-owned shim from the
//!   retired wat-measure crate.

/// wat source files this crate contributes. Order matters —
/// Service.wat declares the protocol typealiases that
/// Console.wat and ConsoleLogger.wat reference.
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[
        // Service.wat first — declares the protocol typealiases
        // (Handle<E>, DriverPair<E>, HandlePool<E>, etc.) that
        // Console.wat (the dispatcher factory) and
        // ConsoleLogger.wat reference.
        wat::WatSource {
            path: "wat-telemetry/telemetry/Service.wat",
            source: include_str!("../wat/telemetry/Service.wat"),
        },
        // Console.wat — the dispatcher factory + Console::Dispatcher<E>
        // typealias. Wraps the SUBSTRATE'S `:wat::std::service::Console::*`
        // driver (paired-channel mini-TCP from arc 089 slice 5);
        // the driver itself stays in the substrate as a generic
        // service-pattern reference, NOT moved by arc 096.
        wat::WatSource {
            path: "wat-telemetry/telemetry/Console.wat",
            source: include_str!("../wat/telemetry/Console.wat"),
        },
        wat::WatSource {
            path: "wat-telemetry/telemetry/ConsoleLogger.wat",
            source: include_str!("../wat/telemetry/ConsoleLogger.wat"),
        },
    ];
    FILES
}

/// Registrar — wires this crate's Rust shims into the process's
/// `rust_deps::RustDepsRegistry`. Slice 1 ships no Rust shims;
/// the function is a no-op stub for now.
pub fn register(_builder: &mut wat::rust_deps::RustDepsBuilder) {
    // Slice 2 adds WorkUnit's shim::register call here.
}
