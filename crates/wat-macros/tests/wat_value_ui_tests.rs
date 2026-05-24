//! Trybuild UI tests for `#[wat_value]` proc-macro.
//!
//! Verifies compile-fail and compile-pass contracts from sub-DESIGN
//! DESIGN-STONE-233.2.l.md. Each fixture lives in `tests/ui/`.
//!
//! Contracts tested:
//! 1. `ui_wat_value_rejects_box_self.rs` — `Box<Self>` field rejected
//! 2. `ui_wat_value_rejects_arc_self.rs` — `Arc<Self>` field rejected
//! 3. `ui_wat_value_rejects_self_direct.rs` — `Self` field directly rejected
//! 4. `ui_wat_value_accepts_opt_in.rs` — opt-in with reason string compiles
//! 5. `ui_wat_value_rejects_alias_bypass.rs` — alias bypass is documented
//!    limitation (Decision 1 of sub-DESIGN); fixture is compile-pass (alias
//!    bypasses syntactic scan), documented with clear comment.
//!
//! Note on alias bypass: per DESIGN-STONE-233.2.l.md Decision 1, the macro
//! uses a pure syntactic scan and does NOT resolve type aliases. A type alias
//! like `type BoxedValue = Box<Value>` used in a field syntactically appears
//! as a single-segment path not matching `Box`, so it bypasses the seal.
//! This is the known limitation. The `ui_wat_value_rejects_alias_bypass.rs`
//! fixture documents this honestly (compile-pass, with a comment explaining
//! the limitation and the recommended workaround: explicit opt-in with reason).

#[test]
fn wat_value_ui() {
    let t = trybuild::TestCases::new();

    // Compile-fail: Box<Self> field rejected.
    t.compile_fail("tests/ui/ui_wat_value_rejects_box_self.rs");

    // Compile-fail: Arc<Self> field rejected.
    t.compile_fail("tests/ui/ui_wat_value_rejects_arc_self.rs");

    // Compile-fail: Self field directly rejected.
    t.compile_fail("tests/ui/ui_wat_value_rejects_self_direct.rs");

    // Compile-pass: opt-in with non-empty reason string compiles.
    t.pass("tests/ui/ui_wat_value_accepts_opt_in.rs");

    // Compile-pass: type alias bypass — documented limitation per Decision 1.
    // The alias form syntactically bypasses the seal (known limitation).
    // This fixture documents the behavior honestly rather than asserting rejection.
    t.pass("tests/ui/ui_wat_value_rejects_alias_bypass.rs");
}
