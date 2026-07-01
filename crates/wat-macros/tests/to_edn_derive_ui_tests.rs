//! Trybuild UI tests for `#[derive(ToEdn)]` proc-macro (arc 296 Strike 1 + 2a).
//!
//! ## Strike 1 (shape constraints)
//!
//! 1. `ui_to_edn_rejects_struct.rs` — struct input rejected at proc-macro level.
//! 2. `ui_to_edn_rejects_tuple_variant.rs` — tuple variant rejected at proc-macro level.
//!
//! ## Strike 2a (attribute DSL grammar)
//!
//! 3. `ui_to_edn_dsl_via_inline_expr.rs` — `via = xs.join(", ")` (inline
//!    expression where a bare path is required) → compile_error!.
//! 4. `ui_to_edn_dsl_key_not_litstr.rs` — `key = 123` (non-LitStr value) →
//!    compile_error!.
//! 5. `ui_to_edn_dsl_bogus_key.rs` — `bogus = "x"` (unknown directive) →
//!    compile_error! naming the allowed set.
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

    // ── Strike 1: shape constraints ──────────────────────────────────────────
    // compile-fail: struct input → "supports enums only"
    t.compile_fail("tests/ui/ui_to_edn_rejects_struct.rs");
    // compile-fail: tuple variant → "does not support tuple variants"
    t.compile_fail("tests/ui/ui_to_edn_rejects_tuple_variant.rs");

    // ── Strike 2a: attribute DSL grammar ─────────────────────────────────────
    // compile-fail: via = xs.join(", ") — inline expression forbidden
    t.compile_fail("tests/ui/ui_to_edn_dsl_via_inline_expr.rs");
    // compile-fail: key = 123 — non-LitStr value forbidden
    t.compile_fail("tests/ui/ui_to_edn_dsl_key_not_litstr.rs");
    // compile-fail: bogus = "x" — unknown directive
    t.compile_fail("tests/ui/ui_to_edn_dsl_bogus_key.rs");
}
