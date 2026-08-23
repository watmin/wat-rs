//! Arc 170 C2 candidate D — the BODILESS extend-type mechanism that unblocked the typed
//! service-handle surface (`TypedCapability<S,R>`), promoted from the disconfirming probes
//! proven this session (scratchpad/probe-v-bodiless.wat, probe-v-swap.wat).
//!
//! WHY: the first C2-D attempt walled on re-declaring `coord`/`grant`/`revoke` in a THIRD
//! per-service extend-type (`DuplicateDefine` on the flat `<Type>/<method>` registration key).
//! The fix: the third auto-emit is BODILESS — no method bodies — registering only the
//! satisfaction EDGE (assignability); runtime dispatch serves the methods off the SAME
//! Handle's existing method bodies via the flat key regardless of which surface named the call.
//!
//! 1. `bodiless_edge_freezes_clean` — a bodiless extend-type registers the edge without
//!    colliding; a raw Handle is assignable to the combined-surface param, and both
//!    `grant`/`coord` resolve through it.
//! 2. `bodiless_edge_is_per_service_swap_is_compile_error` — the edge is PARAMETRIC per
//!    service, not a blanket escape hatch: a wrong-service handle is a located `TypeMismatch`.
//!
//! The real auto-emitted `:wat::capability::TypedCapability<S,R>` mechanism (wat/capability.wat
//! + wat/service.wat's bodiless typedcap-extend) is covered end-to-end by
//!   `probe_arc170_w2a_kwargs_check_mint*` (the checker) and `probe_arc170_c2_mixed_macro*` /
//!   `probe_arc170_c2_strike1_mixed` (the full N-service grant+dial runtime); this file isolates
//!   just the bodiless-edge mechanism itself, with a hand-defined LOCAL surface mirroring the
//!   proven probe shape verbatim.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn bodiless_edge_freezes_clean() {
    startup_from_file("tests/services/probe_arc170_c2_d_bodiless_edge_ok.wat").expect(
        "a bodiless extend-type should register the TypedCapability satisfaction edge without \
         re-declaring methods (no DuplicateDefine); a raw Handle is assignable, and both \
         grant/coord resolve through the combined surface at runtime",
    );
}

#[test]
fn bodiless_edge_is_per_service_swap_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc170_c2_d_bodiless_edge.wat.bad")
        .expect_err("a kv handle bound to an Echo-typed TypedCapability param must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
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
            if expected == "(:probe::TypedCapability :- [:probe::Echo::Op :probe::Echo::Reply])"
            && got == "(:probe::kv::Handle :- [:wat::kernel::Wire])");
}
