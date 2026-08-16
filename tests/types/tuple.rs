//! Integration tests for arc 165 slice 1 — `:wat::core::tuple`
//! (lowercase) renamed to `:wat::core::Tuple` (PascalCase) as the
//! canonical spelling everywhere internal.
//!
//! ## Background
//!
//! Prior to arc 165, `Value::Tuple.type_name()` returned the bare
//! lowercase `"tuple"` (missing the FQDN prefix entirely), and the
//! eval-dispatch arm key + head-field writes also used lowercase.
//! Arc 165 slice 1 aligns all storage sites to PascalCase, completing
//! the arc 109 slice 1f vec→Vector playbook for the one remaining
//! lowercase container head.
//!
//! ## Pattern 2 poison shape (arc 109 slice 1g)
//!
//! The check.rs Pattern 2 poison at lines 3901-3914 STAYS — its
//! callee match key remains `:wat::core::tuple` (the retired legacy
//! spelling being poisoned). It emits `TypeMismatch` redirecting to
//! `:wat::core::Tuple`. Arc 165 closes the storage gap: the redirect
//! target now matches the storage canonical form.
//!
//! ## Tuple type-position syntax note
//!
//! In wat source, the TUPLE TYPE is written `:(T,U,V)` (comma-
//! separated bare type paths, no leading `:` on inner args). This is
//! the type-position form. `(:wat::core::Tuple ...)` is the CTOR
//! (expression-position form). These are distinct; return-type
//! declarations use `:(T,U,V)`.
//!
//! ## Test shapes
//!
//! - Positive tests use `startup_ok` to assert clean type-check + freeze.
//! - Negative tests use `startup_err` + substring assertions to verify
//!   specific error variants surface.

use wat::freeze::startup_from_file;

/// Asserts the given fixture starts up cleanly. Panics with the
/// diagnostic on failure.
fn startup_ok(path: &str) {
    if let Err(e) = startup_from_file(path) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

/// Asserts the given fixture fails at startup. Returns the
/// Debug-formatted error string for substring assertions.
fn startup_err(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// --- 1. Canonical PascalCase constructor works --------------------------

#[test]
fn tuple_pascal_canonical_works() {
    startup_ok("tests/types/tuple_pascal_canonical.wat");
}

// --- 2. Legacy lowercase triggers Pattern 2 poison ----------------------

#[test]
fn legacy_tuple_lowercase_redirects_via_pattern2_poison() {
    let err = startup_err("tests/types/tuple_legacy_lowercase.wat.bad");
    wat::assert_edn_matches_file!(err, "tuple__legacy_tuple_lowercase_redirects_via_pattern2_poison.edn", "legacy lowercase tuple redirects via Pattern 2 poison: TypeMismatch + Retirement remedy");
}

// --- 3. Tuple in function return position type-checks clean -------------

#[test]
fn tuple_in_function_return_position() {
    startup_ok("tests/types/tuple_in_return_position.wat");
}

// --- 4. type_name returns FQDN PascalCase: runtime shape matches --------

#[test]
fn type_name_returns_fqdn_pascal() {
    startup_ok("tests/types/tuple_type_name_fqdn_pascal.wat");
}
