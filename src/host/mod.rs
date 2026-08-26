//! `host` — the surface a Rust program uses to run wat.
//!
//! Three members: `run_program` + the `wat::main!` macro's runtime
//! half ([`entry`]), the `Guest` facade for embedding wat as a guest
//! in a Rust process ([`guest`]), and `.wat` test-suite discovery,
//! invocation, and the reporting contract ([`test_runner`]).
//!
//! No re-exports here — reach members via `crate::host::entry::…` /
//! `crate::host::guest::…` / `crate::host::test_runner::…`, or the
//! retargeted crate-root re-exports in `lib.rs`.

pub mod entry;
pub mod guest;
pub mod test_runner;
