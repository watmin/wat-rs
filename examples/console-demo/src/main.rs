//! Run with: `cargo run -p console-demo`
//!
//! Wat source at `wat/main.wat` renders a tiny domain enum five
//! ways (EDN / NoTagEdn / Json / NoTagJson / Pretty) and emits
//! each line through the ambient `:wat::kernel::println` /
//! `eprintln` ops. Each domain event becomes one parseable line;
//! nothing free-form crosses the boundary. Arc 170 slice 1f-η —
//! Console driver retired; ambient stdio replaces the handle-
//! plumbed surface entirely.

wat::main! {}
