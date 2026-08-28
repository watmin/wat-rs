//! Arc 278 #55 (S3b+S4) slice one — EXPECTATIONS row 8: `:undefined` is mandatory on the
//! fallback-carrying rete op. NOT load-bearing (EXPECTATIONS marks rows 3-6 and 14 load-bearing;
//! this row is covered on a best-effort basis).
//!
//! ⚠ DEVIATION FROM THE PREDICTED SHAPE, named plainly: EXPECTATIONS predicted the omission
//! would surface as `kwargs-lower: missing argument :undefined` — the error text a kwargs-
//! lowering DEFMACRO would produce. `:wat::rete::i64::+` is NOT implemented as a kwargs
//! defmacro (that would need a `wat/` file, out of this slice's scope — see the brief's own
//! supporting fact: "a kwargs surface is a defmacro lowering to a positional prime, so a
//! keyword argument never reaches an intrinsic"). It is a plain 4-ary positional Rust
//! intrinsic instead, with the literal keyword `:undefined` as a mandatory 3rd-slot marker
//! inspected directly on the raw AST (`runtime.rs`'s `dispatch_rete_op`). Omitting an argument
//! therefore fails as an ordinary arity mismatch against the registered 4-param `TypeScheme`,
//! not the predicted kwargs-lower message. The SUBSTANCE — the call fails to check when the
//! marker+fallback pair is omitted — holds; the exact wording does not.
//!
//! Run: cargo test --release --test rete

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

#[test]
fn omitting_the_undefined_marker_and_fallback_fails_to_check() {
    let result = startup_from_file("tests/rete/probe_arc278_55_slice_one_undefined_mandatory.wat");
    wat::assert_startup_error!(result, check
        CheckErrorKind::ArityMismatch { callee, expected, got }
            if callee == ":wat::rete::i64::+"
            && *expected == 4
            && *got == 3
    );
}
