//! Trybuild UI tests for `#[derive(ToEdn)]` proc-macro (arc 296).
//!
//! These tests moved from `wat-macros` to `wat-to-edn-derive` (stone A) because
//! the derive macro itself moved crates. The test files use `use wat_to_edn_derive::ToEdn`
//! instead of `use wat_macros::ToEdn`.
//!
//! ## Shape constraints
//!
//! 1. `ui_to_edn_rejects_struct.rs` — tuple/unit struct input rejected at proc-macro
//!    level. STOP-4 note: named-field structs ARE now supported (the original test
//!    used a named-field struct and is updated to a tuple struct which is still rejected).
//! 2. `ui_to_edn_rejects_tuple_variant.rs` — keyless single-field tuple variant
//!    rejected (must have `#[to_edn(key = "…")]`).
//!
//! ## Attribute DSL grammar
//!
//! 3. `ui_to_edn_dsl_via_inline_expr.rs` — `via = xs.join(", ")` forbidden.
//! 4. `ui_to_edn_dsl_key_not_litstr.rs` — `key = 123` (non-LitStr) forbidden.
//! 5. `ui_to_edn_dsl_bogus_key.rs` — unknown directive name → compile_error!.

#[test]
fn to_edn_derive_ui() {
    let t = trybuild::TestCases::new();

    // ── Shape constraints ────────────────────────────────────────────────────
    // compile-fail: tuple struct → "supports named-field structs only"
    t.compile_fail("tests/ui/ui_to_edn_rejects_struct.rs");
    // compile-fail: keyless single-field tuple variant
    t.compile_fail("tests/ui/ui_to_edn_rejects_tuple_variant.rs");

    // ── Attribute DSL grammar ─────────────────────────────────────────────────
    // compile-fail: via = xs.join(", ") — inline expression forbidden
    t.compile_fail("tests/ui/ui_to_edn_dsl_via_inline_expr.rs");
    // compile-fail: key = 123 — non-LitStr value forbidden
    t.compile_fail("tests/ui/ui_to_edn_dsl_key_not_litstr.rs");
    // compile-fail: bogus = "x" — unknown directive
    t.compile_fail("tests/ui/ui_to_edn_dsl_bogus_key.rs");
}
