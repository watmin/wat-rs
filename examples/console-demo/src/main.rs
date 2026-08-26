//! Run with: `cargo run -p console-demo`
//!
//! Wat source at `wat/main.wat` emits five domain events — three
//! `:Buy`/`:Sell` events and two `:CircuitBreak` events — each
//! through the ambient `:wat::kernel::println` op. Each domain
//! event becomes one EDN-encoded, parseable stdout line; nothing
//! free-form crosses the boundary, and nothing is ever written to
//! stderr (`eprintln` is wat's PANIC channel, not a second print).
//! Arc 170 slice 1f-η — Console driver retired; ambient stdio
//! replaces the handle-plumbed surface entirely.

wat::main! {}
