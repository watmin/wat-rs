//! Arc 170 C2 — complete parametric surfaces (Gaps 1+2): a concrete type satisfies a PARAMETRIC
//! surface param, and a parametric-surface method's return instantiates from the receiver's args.
//!
//! WHY: the arc-170 C2 parametric-surface support landed half-built — it handled the two *concrete*
//! cases (a concrete satisfier's `<Type>/<method>` scheme resolves the surface `<T>`) but neither
//! *abstract* one. The `Dialable`-checker (the `bracket/uses` consumer) surfaced both gaps:
//!  - **Gap 2** (`check.rs` surface-method call): `(Dialable/coord d)` where `d` is an ABSTRACT
//!    `Dialable<Echo::Op,Echo::Reply>` param produced the raw `Address'<S,R>` (uninstantiated),
//!    because no concrete satisfier scheme exists for an abstract receiver. Now the surface's own
//!    `<T>` params instantiate from the receiver's concrete args → `Address'<Echo::Op,Echo::Reply>`.
//!  - **Gap 1** (`assignable`): a concrete `echo'::Handle` did not satisfy a `Dialable<Echo::Op,
//!    Echo::Reply>` PARAM — `assignable` had no `(concrete-Path actual, parametric-surface expected)`
//!    rule. Now it consults the full-args extend-type edge (nature-floor-checked).
//!
//! Zero regression: both are new guarded branches; monomorphic surfaces + concrete-satisfier
//! resolution are byte-for-byte unchanged. SOUNDNESS, not permissiveness: `echo'::Handle` satisfies
//! ONLY `Dialable<Echo::Op,Echo::Reply>`, never `Dialable<Kv::Op,Kv::Reply>` — the negative proves it.
//!
//! 1. `abstract_parametric_surface_param_and_coord_freeze_clean` — the positive (= the RED gate):
//!    the abstract-`Dialable` fn + a raw `echo'::Handle` caller freezes clean.
//! 2. `wrong_parametric_surface_param_is_compile_error` — the soundness proof: an `echo'::Handle`
//!    passed where `Dialable<Kv…>` is wanted is a located `TypeMismatch` (structural).

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn abstract_parametric_surface_param_and_coord_freeze_clean() {
    startup_from_file("tests/types/probe_arc170_parametric_surface_param_ok.wat").expect(
        "abstract Dialable<S,R> path must freeze clean: (Dialable/coord d) resolves \
         Address'<Echo::Op,Echo::Reply> (Gap 2) and a raw echo'::Handle assigns to the \
         Dialable<Echo::Op,Echo::Reply> param (Gap 1)",
    );
}

#[test]
fn wrong_parametric_surface_param_is_compile_error() {
    let err =
        startup_from_file("tests/types/probe_arc170_parametric_surface_param_wrong_param.wat.bad")
            .expect_err("echo'::Handle passed where Dialable<Kv…> is wanted must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // The Gap-1 edge is an EXACT-args match, so an echo handle does NOT satisfy the Kv
    // Dialable — the swap-gate holds.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, .. }
            if expected == ":wat::capability::Dialable<probe::Kv::Op,probe::Kv::Reply>"
            && got == ":probe::echo::Handle<wat::kernel::Wire>");
}
