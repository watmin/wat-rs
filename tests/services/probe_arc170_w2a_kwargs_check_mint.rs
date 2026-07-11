//! Arc 170 W2a — auto-mint `<fqdn>::kwargs-check` at the kwargs-defn codegen site.
//!
//! WHY: `defn`'s kwargs branch (`wat/core.wat:876`) now ALSO auto-mints a fourth form,
//! `<fqdn>::kwargs-check` — a kwargs fn whose `Peer'<S,R>` field types are head-swapped
//! to `Address'<S,R>` (data-typed fields pass through untouched). This wards the
//! AUTO-mint directly: a correct kwargs call to the auto-minted checker freezes clean;
//! a swapped one is a located `TypeMismatch` on the two `Address'` types.
//!
//! 1. `w2a_kwargs_check_mint_ok_freezes_clean` — the positive control.
//! 2. `w2a_kwargs_check_mint_swap_is_compile_error` — the swap, structurally asserted.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn w2a_kwargs_check_mint_ok_freezes_clean() {
    startup_from_file("tests/services/probe_arc170_w2a_kwargs_check_mint_ok.wat").expect(
        "correct kwargs call to the auto-minted :probe::enrich::kwargs-check should freeze \
         clean: the head-swapped Address'<S,R> field types resolve the RIGHT service",
    );
}

#[test]
fn w2a_kwargs_check_mint_swap_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc170_w2a_kwargs_check_mint_swap.wat.bad")
        .expect_err("swapped handles at the auto-minted checker must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    match &errs[0].kind {
        CheckErrorKind::TypeMismatch { expected, got, .. } => {
            // kv handle's coord ascribed to the :echo kwarg (Address'<Echo...>) — the
            // auto-minted checker's head-swapped field type rejects it.
            assert_eq!(expected, ":wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>");
            assert_eq!(got, ":wat::kernel::Address'<probe::Kv::Op,probe::Kv::Reply>");
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}
