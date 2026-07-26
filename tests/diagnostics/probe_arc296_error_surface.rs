//! Arc 296 — `:wat::core::Error` stdlib surface probe.
//!
//! Proves three things:
//! (a) `:wat::core::Error` is declared in core.wat and startup boots cleanly.
//! (b) A user `defrecord` satisfying the surface may be passed to a
//!     `[e <- :wat::core::Error]` param (structural satisfaction) and
//!     field-accessed via `:wat::core::Error/message` (arc 293.4d field
//!     accessor on a surface-typed receiver).
//! (c) `edn::write`→`edn::read` round-trips the error record without error.
//!
//! RED at HEAD: `:wat::core::Error` is unknown → startup fails (UnknownCallee
//! or unresolved field-type reference inside `:probe::BadInput`'s defrecord).
//!
//! GREEN after `wat/core.wat` adds the `defsurface :wat::core::Error` form.

use wat::freeze::call_beside_value;

#[test]
fn error_surface_declares_and_record_satisfies_and_round_trips() {
    // (a) startup boots — proves :wat::core::Error is in the type registry.
    // (b) :probe::BadInput (defrecord) passes a [e <- :wat::core::Error] param.
    // (c) edn::write→edn::read round-trip inside :user::main doesn't raise.
    let _result = call_beside_value(file!(), ":user::main")
        .unwrap_or_else(|e| panic!("(:user::main) raised a runtime error: {e:?}"));
}
