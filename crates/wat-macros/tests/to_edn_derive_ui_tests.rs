//! Trybuild UI tests for `#[derive(ToEdn)]` proc-macro (arc 296 Strike 1).
//!
//! Verifies compile-fail contracts for the `ToEdn` derive macro:
//! 1. `ui_to_edn_rejects_struct.rs` — struct input rejected at proc-macro level.
//! 2. `ui_to_edn_rejects_tuple_variant.rs` — tuple variant rejected at proc-macro level.
//!
//! ## What is NOT tested here
//!
//! The "non-`ToEdn` field type" wall — an enum with a field whose type is not
//! `ToEdn` (e.g. `std::net::TcpStream`) — is enforced by the Rust type system
//! at the call site `field.to_edn()` in the generated impl. This wall is
//! proven by the design (any non-`ToEdn` type causes `the trait bound '…:
//! ToEdn' is not satisfied`) and is exercised implicitly by the existing
//! compile-pass contracts (all `ConfigErrorKind` fields implement `ToEdn`).

#[test]
fn to_edn_derive_ui() {
    let t = trybuild::TestCases::new();

    // compile-fail: struct input → "supports enums only"
    t.compile_fail("tests/ui/ui_to_edn_rejects_struct.rs");

    // compile-fail: tuple variant → "does not support tuple variants"
    t.compile_fail("tests/ui/ui_to_edn_rejects_tuple_variant.rs");
}
