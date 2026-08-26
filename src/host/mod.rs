//! `host` — the surface a Rust program uses to run wat.
//!
//! Three members: `compose_and_run` + the `wat::main!` macro's runtime
//! half ([`compose`]), the `Harness` facade for embedding wat as a guest
//! in a Rust process ([`harness`]), and `.wat` test-suite discovery,
//! invocation, and the reporting contract ([`test_runner`]).
//!
//! No re-exports here — reach members via `crate::host::compose::…` /
//! `crate::host::harness::…` / `crate::host::test_runner::…`, or the
//! retargeted crate-root re-exports in `lib.rs`.

pub mod compose;
pub mod harness;
pub mod test_runner;
