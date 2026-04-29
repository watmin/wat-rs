//! Run with: `cargo run -p console-demo`
//!
//! Wat source at `wat/main.wat` wires the substrate's telemetry sink
//! to stdout via EDN-per-line. Each domain event becomes one
//! parseable EDN line; nothing free-form crosses the boundary.

wat::main! {
    deps: [wat_telemetry],
}
