//! `wat-telemetry` — telemetry primitives for wat.
//!
//! Arc 096. Owns the `:wat::telemetry::*` namespace. Subsumes the
//! pre-arc `wat-measure` crate (which retired in slice 3) and the
//! substrate's old `:wat::std::telemetry::*` namespace.
//!
//! What ships under `:wat::telemetry::*`:
//!
//! - **Service<E,G>** — generic queue-fronted destination service
//!   with paired-channel batch+ack discipline (arc 089 + arc 095).
//! - **WorkUnit + Event + scope HOF** — measurement-scope state +
//!   the events it ships at scope-close. Folded in from the
//!   retired wat-measure crate.
//! - **uuid::v4** — backward-compat alias under `:wat::telemetry::*`
//!   that delegates to the typed `:wat::core::Uuid/v4` constructor
//!   (arc 206 slice 3 retired the duplicate Rust shim; arc 207 slice 3
//!   retargeted to the typed constructor; new code uses `:wat::core::Uuid/v4` directly).
//! - **Tag, Tags, SinkHandles** — the type aliases consumers
//!   reference at every measurement scope.
//!
//! Arc 170 slice 1f-η retired the Console / ConsoleLogger wrappers
//! that previously fronted the former Console stdio service (retired).
//! With the ambient stdio trio + runtime orchestrator in place,
//! producers call `:wat::kernel::println` / `eprintln` directly;
//! structured-format dispatch lives in user code (see
//! `examples/console-demo` for the canonical ambient-stdio pattern).
//!
//! # Two-part contract
//!
//! - [`wat_sources`] — the baked `.wat` files (Service, types,
//!   Event, uuid, WorkUnit, WorkUnitLog).
//! - [`register`] — wires the WorkUnit thread-owned cell Rust shim
//!   into the deps builder. (UUID minting no longer needs a telemetry
//!   shim — `:wat::core::Uuid/v4` is the typed substrate primitive per arc 207.)

pub mod workunit;

/// wat source files this crate contributes. Order matters —
/// Service.wat declares the protocol typealiases (Handle<E>,
/// DriverPair<E>, AckTx, AckRx, etc.) that types.wat and the rest
/// reference.
///
/// Arc 170 slice 1f-η — Console.wat + ConsoleLogger.wat retired.
/// Producers needing structured stdio call `:wat::kernel::println`
/// / `eprintln` ambiently; structured-format dispatch is a wat-
/// level concern living in user code (see `examples/console-demo`).
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[
        // Service.wat first — the protocol types every other file
        // forward-references through typealiases.
        wat::WatSource {
            path: "wat-telemetry/telemetry/Service.wat",
            source: include_str!("../wat/telemetry/Service.wat"),
        },
        // types.wat — Tag, Tags, SinkHandles (the user-facing aliases).
        wat::WatSource {
            path: "wat-telemetry/telemetry/types.wat",
            source: include_str!("../wat/telemetry/types.wat"),
        },
        // Event.wat — the substrate-defined enum for measurement
        // events. Tags references via the alias from types.wat.
        wat::WatSource {
            path: "wat-telemetry/telemetry/Event.wat",
            source: include_str!("../wat/telemetry/Event.wat"),
        },
        // uuid.wat — :wat::telemetry::uuid::v4 backward-compat alias
        // that delegates to :wat::core::Uuid/v4 (arc 207 slice 3).
        wat::WatSource {
            path: "wat-telemetry/telemetry/uuid.wat",
            source: include_str!("../wat/telemetry/uuid.wat"),
        },
        // WorkUnit.wat — measurement-scope state surface. Depends
        // on Tags (types.wat) + the WorkUnit Rust shim
        // (:rust::telemetry::WorkUnit registered by workunit.rs).
        wat::WatSource {
            path: "wat-telemetry/telemetry/WorkUnit.wat",
            source: include_str!("../wat/telemetry/WorkUnit.wat"),
        },
        // WorkUnitLog.wat — producer-side Log emitter. Closure over
        // (handle, caller, now-fn); ships Event::Log rows through
        // Service<Event,_>. Depends on WorkUnit (for namespace /
        // uuid / tags accessors at emit time) + Event types.
        wat::WatSource {
            path: "wat-telemetry/telemetry/WorkUnitLog.wat",
            source: include_str!("../wat/telemetry/WorkUnitLog.wat"),
        },
    ];
    FILES
}

/// Registrar — wires the `:rust::telemetry::WorkUnit` thread-owned
/// cell shim into the deps builder. UUID minting retired from Rust shim
/// in arc 206 slice 3; `:wat::telemetry::uuid::v4` now delegates to the
/// typed `:wat::core::Uuid/v4` constructor at the wat layer (arc 207 slice 3).
pub fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    workunit::register(builder);
}
