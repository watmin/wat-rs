//! Arc 170 — the capability-circuit crown jewel: the wrong-service COMPILE error.
//!
//! WHY: a service handle dialed as the WRONG service must be a located `TypeMismatch` at
//! `wat --check` time — never a runtime "peer closed". This is carried by the auto-emitted
//! typed `:wat::capability::Dialable<S,R>` coordinate (W1): `defservice` now auto-emits, per
//! service, that its `<fqdn>::Handle` satisfies the baked parametric surface
//! `:wat::capability::Dialable<S,R>` with `(coord [self] -> :wat::kernel::Address<S,R>)`. So
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

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

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
    let err = startup_from_file("tests/services/probe_arc170_wrong_service_compile_error.wat.bad")
        .expect_err("wrong-service coord must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // kv handle's coord ascribed to an Echo address → the ann-form rejects it.
    // rune:lint(no-inlined-wat) — the expected/got strings below are golden COMPARISON
    // text for a TypeMismatch's rendered fields, never a wat world/driver; they happen to be
    // reader-parseable now only because the checker's error renderer emits real `(Head :- [args])`
    // syntax instead of the retired unparseable `Head<a,b>` pseudo-syntax (that is the whole point
    // of this stone). Nothing here builds or runs a wat program from this string.
    // STONE-defservice-emits-the-binder (arc 109) — same call site, re-rendered: the
    // checker stopped minting `Head<a,b>` (a spelling the reader now refuses) and emits
    // the surviving `(Head :- [args])` form instead.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, .. }
            if expected == "(:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply])"
            && got == "(:wat::kernel::Address :- [:probe::Kv::Op :probe::Kv::Reply])");
}

#[test]
fn swapped_colocation_tuple_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc170_wrong_service_colocation.wat.bad")
        .expect_err("swapped tuple must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // swapped Tuple vs the field-ordered (Echo, Kv) contract at the downstream consumer.
    // STONE-defservice-emits-the-binder (arc 109) — same call site, re-rendered: the
    // checker stopped minting `Head<a,b>` (a spelling the reader now refuses) and emits
    // the surviving `(Head :- [args])` form instead (colon-stripped inside the Tuple, per
    // `format_type_inner`'s existing nested-element convention — unchanged by this stone).
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, .. }
            if expected == ":((wat::kernel::Address :- [probe::Echo::Op probe::Echo::Reply]),\
                              (wat::kernel::Address :- [probe::Kv::Op probe::Kv::Reply]))"
            && got == ":((wat::kernel::Address :- [probe::Kv::Op probe::Kv::Reply]),\
                         (wat::kernel::Address :- [probe::Echo::Op probe::Echo::Reply]))");
}
