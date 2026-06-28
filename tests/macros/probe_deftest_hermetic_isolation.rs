//! Arc 170 slice 3 Gap G — isolation contract probes for `deftest-hermetic`.
//!
//! Four probes verify the strict-isolation property that `deftest-hermetic`
//! provides: the parent's frozen symbol table is UNTOUCHED by any prelude
//! content. Only the test function entry point (`:name`) is registered at
//! the parent's top level; everything in the prelude lives exclusively in
//! the child's sandboxed world.
//!
//! All four probe programs are merged in the co-located fixture:
//! probe_deftest_hermetic_isolation.wat (slurped via startup_beside(file!())).
//! Each test loads the same world and asserts on specific symbols/types.
//!
//! Probe 1: Parent symbol table has no prelude struct accessors.
//! Probe 2: Two deftest-hermetic calls with same FQDN prelude types — no collision.
//! Probe 3: Test fn is in parent; prelude struct is NOT.
//! Probe 4: make-deftest-hermetic with define prelude freezes cleanly;
//!          parent has test fns but NOT prelude helpers.

use wat::freeze::startup_beside;

// ─── Probe 1 — parent symbol table untouched by prelude struct ───────────────

/// `deftest-hermetic` with a prelude that declares a struct type.
///
/// The struct declaration is INSIDE the `(:wat::core::forms ...)` passed to
/// `run-sandboxed-hermetic-ast` — it is quoted AST data, not a live
/// top-level declaration. The outer freeze pipeline (step 5 `register_types`,
/// step 6 `register_defines`, step 6a `register_struct_methods`) never sees
/// the struct form. The parent's `sym.functions` does NOT receive
/// `:test::g::IsolatedType/new`.
///
/// Demonstrates: parent's frozen symbol table is UNTOUCHED by prelude
/// struct declarations. Strict isolation holds.
#[test]
fn probe_parent_has_no_prelude_struct_accessors() {
    let world = startup_beside(file!()).expect("freeze should succeed");

    // Test fn IS registered in parent (the test runner can find it).
    assert!(
        world.symbols().get(":test::g::my-hermetic-test").is_some(),
        ":test::g::my-hermetic-test not in parent — test runner can't find it"
    );

    // Prelude struct accessors are NOT in parent (strict isolation).
    assert!(
        world.symbols().get(":test::g::IsolatedType/new").is_none(),
        ":test::g::IsolatedType/new is in parent — isolation violated"
    );
    assert!(
        world.symbols().get(":test::g::IsolatedType/field").is_none(),
        ":test::g::IsolatedType/field is in parent — isolation violated"
    );

    // Prelude type is NOT in parent's TypeEnv (strict isolation).
    assert!(
        world.types().get(":test::g::IsolatedType").is_none(),
        ":test::g::IsolatedType is in parent TypeEnv — isolation violated"
    );
}

// ─── Probe 2 — cross-test prelude isolation: same FQDN, no collision ────────

/// Two `deftest-hermetic` calls in the same file each declare a struct with
/// the SAME FQDN (`:test::g::SharedName`) in their prelude.
///
/// Under the old forms-based mechanism, both structs are inside separate
/// `forms` calls — neither is registered in the parent's TypeEnv. There is
/// no collision between the two declarations at parent freeze time.
/// Each test runs in its own hermetic child world where the struct is
/// independently declared.
///
/// Demonstrates: `deftest-hermetic` preludes are independent — no shared
/// parent-side type registry entry, no cross-test contamination.
#[test]
fn probe_cross_test_prelude_isolation_same_fqdn_no_collision() {
    // Both tests freeze cleanly — no collision in parent TypeEnv.
    let world = startup_beside(file!()).expect("freeze should succeed");

    // Both test fns are registered in parent (test runner finds them).
    assert!(
        world.symbols().get(":test::g::first-hermetic-test").is_some(),
        ":test::g::first-hermetic-test not in parent"
    );
    assert!(
        world.symbols().get(":test::g::second-hermetic-test").is_some(),
        ":test::g::second-hermetic-test not in parent"
    );

    // SharedName is in NEITHER test's parent world (both preludes are isolated).
    assert!(
        world.types().get(":test::g::SharedName").is_none(),
        ":test::g::SharedName leaked into parent TypeEnv from one of the preludes"
    );
    assert!(
        world.symbols().get(":test::g::SharedName/new").is_none(),
        ":test::g::SharedName/new leaked into parent sym from one of the preludes"
    );
}

// ─── Probe 3 — test fn visible in parent; prelude content invisible ──────────

/// The test fn entry point (`:test::g::visible-test`) IS registered at
/// the parent's top level — the test runner needs it to discover and
/// invoke the test. The prelude's struct (`:test::g::HiddenStruct`) and
/// any helper defines are NOT registered in the parent's world.
///
/// This is the core of the strict-isolation contract: exactly ONE thing
/// crosses the parent/child boundary at the parent level — the test fn's
/// registration. Everything else stays in the child.
#[test]
fn probe_test_fn_visible_prelude_content_invisible() {
    let world = startup_beside(file!()).expect("freeze should succeed");

    // Test fn CROSSES the boundary (registered in parent).
    assert!(
        world.symbols().get(":test::g::visible-test").is_some(),
        ":test::g::visible-test not registered — test runner can't find it"
    );

    // Prelude struct does NOT cross the boundary.
    assert!(
        world.types().get(":test::g::HiddenStruct").is_none(),
        ":test::g::HiddenStruct in parent TypeEnv — isolation violated"
    );
    assert!(
        world.symbols().get(":test::g::HiddenStruct/new").is_none(),
        ":test::g::HiddenStruct/new in parent sym — isolation violated"
    );
    assert!(
        world.symbols().get(":test::g::HiddenStruct/x").is_none(),
        ":test::g::HiddenStruct/x in parent sym — isolation violated"
    );

    // Prelude helper define does NOT cross the boundary.
    assert!(
        world.symbols().get(":test::g::hidden-helper").is_none(),
        ":test::g::hidden-helper in parent sym — isolation violated"
    );
}

// ─── Probe 4 — make-deftest-hermetic with define prelude freezes cleanly ────

/// `make-deftest-hermetic` factory generates a configured `deftest-hermetic`
/// variant. The default-prelude contains `define` forms (helper functions).
/// At parent freeze time, these defines are inside `forms` — not live code.
/// The parent's sym does NOT have the prelude helpers.
///
/// At child runtime, the prelude forms go through `startup_from_forms` in
/// the hermetic child subprocess, where `register_defines` (step 6) correctly
/// processes the `define` forms. The helpers are available to the test body
/// inside the child's world.
///
/// This probe mirrors the structure of
/// `wat-tests/kernel/services/ambient-stdio.wat` — the primary real-world
/// consumer of `make-deftest-hermetic`. It verifies that a prelude with
/// helper `define` forms compiles cleanly and achieves parent isolation.
#[test]
fn probe_make_deftest_hermetic_define_prelude_parent_isolated() {
    let world = startup_beside(file!()).expect("freeze should succeed");

    // The test fn IS registered in parent (generated by make-deftest-hermetic).
    assert!(
        world.symbols().get(":test::g::using-make-deftest-hermetic").is_some(),
        ":test::g::using-make-deftest-hermetic not in parent"
    );

    // The prelude helper define is NOT in parent (strict isolation).
    // run-inner was declared in the default-prelude, which is inside forms —
    // never registered in the parent's symbol table.
    assert!(
        world.symbols().get(":test::g::run-inner").is_none(),
        ":test::g::run-inner is in parent sym — isolation violated"
    );
}
