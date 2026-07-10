//! Arc 170 — the capability-circuit crown jewel: the wrong-service COMPILE error.
//!
//! WHY: a service handle dialed as the WRONG service must be a located `TypeMismatch` at
//! `wat --check` time — never a runtime "peer closed". This is carried by the auto-emitted
//! typed `:wat::capability::Dialable<S,R>` coordinate (W1): `defservice` now auto-emits, per
//! service, that its `<fqdn>::Handle` satisfies the baked parametric surface
//! `:wat::capability::Dialable<S,R>` with `(coord [self] -> :wat::kernel::Address'<S,R>)`. So
//! `(:wat::capability::Dialable/coord eh)` types as `Address'<Echo...>` and
//! `(:wat::capability::Dialable/coord kvh)` types as `Address'<Kv...>` — ascribing a kv
//! handle's coord to an Echo address is the wrong-service swap, caught by the checker.
//!
//! Commits `c49d2a4b`/`4bb4ee34` shipped the auto-emit mechanism with no committed test —
//! this file wards it. There is no hand-written `defsurface Dialable`/`extend-type` anywhere
//! in the fixtures below: the surface is baked (`wat/capability.wat`) and auto-emitted
//! (`wat/service.wat`); declaring it by hand DUPLICATE-DEFINEs.
//!
//! 1. `correct_service_coord_compiles` — the positive control: ascribing the echo handle's
//!    coord to an Echo address freezes clean.
//! 2. `wrong_service_coord_is_compile_error` — ascribing the KV handle's coord to an Echo
//!    address is a located `TypeMismatch` (expected `Address'<Echo...>`, got `Address'<Kv...>`).
//! 3. `swapped_colocation_tuple_is_compile_error` — the co-location angle: a `Tuple` of typed
//!    coords carries a field-ordered contract through a `let`; a SWAPPED tuple is rejected at
//!    the downstream consumer with a located `TypeMismatch` (not `DuplicateDefine`).

use wat::freeze::startup_from_file;

#[test]
fn correct_service_coord_compiles() {
    startup_from_file("tests/services/probe_arc170_wrong_service_compile_error_ok.wat")
        .expect(
            "correctly-typed coord (echo handle -> Echo address) should freeze clean: the \
             auto-emitted Dialable<S,R> coordinate resolves the RIGHT service",
        );
}

#[test]
fn wrong_service_coord_is_compile_error() {
    let err = format!(
        "{:?}",
        startup_from_file("tests/services/probe_arc170_wrong_service_compile_error.wat.bad")
            .expect_err("wrong-service coord must fail check")
    );
    assert!(
        err.contains("TypeMismatch") && err.contains("Echo") && err.contains("Kv"),
        "expected a TypeMismatch naming Echo and Kv (kv handle's coord ascribed to an Echo address), got: {err}"
    );
}

#[test]
fn swapped_colocation_tuple_is_compile_error() {
    let err = format!(
        "{:?}",
        startup_from_file("tests/services/probe_arc170_wrong_service_colocation.wat.bad")
            .expect_err("swapped tuple must fail check")
    );
    assert!(
        err.contains("TypeMismatch") && err.contains("Echo") && err.contains("Kv"),
        "expected a TypeMismatch (swapped Tuple vs the field-ordered Echo/Kv contract), got: {err}"
    );
}
