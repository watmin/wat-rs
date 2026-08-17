//! Arc 170 W2a/C2-D — auto-mint `<fqdn>::kwargs-check` at the kwargs-defn codegen site.
//!
//! WHY: `defn`'s kwargs branch (`wat/core.wat:876`) now ALSO auto-mints a fourth form,
//! `<fqdn>::kwargs-check` — a kwargs fn whose `Peer'<S,R>` field types are head-swapped
//! to `TypedCapability<S,R>` (data-typed fields pass through untouched; arc 170 C2
//! candidate D). This wards the AUTO-mint directly: a correct kwargs call (raw handles)
//! to the auto-minted checker freezes clean; a swapped one is a located `TypeMismatch`
//! naming the expected `TypedCapability<S,R>` and the got concrete `Handle` type.
//!
//! 1. `w2a_kwargs_check_mint_ok_freezes_clean` — the positive control.
//! 2. `w2a_kwargs_check_mint_swap_is_compile_error` — the swap, structurally asserted.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn w2a_kwargs_check_mint_ok_freezes_clean() {
    startup_from_file("tests/services/probe_arc170_w2a_kwargs_check_mint_ok.wat").expect(
        "correct kwargs call (raw handles) to the auto-minted :probe::enrich::kwargs-check \
         should freeze clean: the head-swapped TypedCapability<S,R> field types resolve the \
         RIGHT service via the bodiless auto-emitted edge",
    );
}

#[test]
fn w2a_kwargs_check_mint_swap_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc170_w2a_kwargs_check_mint_swap.wat.bad")
        .expect_err("swapped handles at the auto-minted checker must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // kv's RAW handle bound to the :echo kwarg (TypedCapability<Echo...>) — the
    // auto-minted checker's head-swapped field type rejects it. `got` names the
    // concrete Handle type (the receiver), not a TypedCapability-wrapped name.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, .. }
            if expected == ":wat::capability::TypedCapability<probe::Echo::Op,probe::Echo::Reply>"
            && got == ":probe::kv::Handle<wat::kernel::Wire>");
}
