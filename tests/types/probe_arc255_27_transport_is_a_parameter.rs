//! Stone 255.27 (C-b5) — a transport is an ordinary type parameter (arc 255).
//!
//! Each `.wat.bad` row was ACCEPTED (rc=0) on the pre-stone binary (`576200c59`) by a transport
//! special case in `src/check.rs` that this stone deleted:
//!
//! - `letter_as_shared` / `shared_as_letter` — `assignable`'s same-head arm that admitted a
//!   declared letter against `Transport.Shared`/`Transport.Wire` (either direction) without
//!   binding it (`transport_param_instantiates`, the D2 site).
//! - `record_short` / `record_long` — `unify`'s `is_transport_slot` n vs n+1 arms (and
//!   `assignable`'s ±1 "missing transport slot" arm above them): a same-head type one argument
//!   short, or one transport-shaped argument long, was the same type.
//!
//! A correct checker refuses each with `ReturnTypeMismatch` naming both types exactly.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const DIR: &str = "tests/types/probe_arc255_27_transport_is_a_parameter";

fn refused(suffix: &str, got_ty: &str, expected_ty: &str) {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must fail check"));
    let StartupError::Check(CheckErrors(errs)) = err else {
        panic!("{path}: expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::k" && expected == expected_ty && got == got_ty);
}

// rune:lint(no-inlined-wat) — every `"(:probe::R :- [...])"` literal below is golden COMPARISON
// text for the refusal's rendered `expected`/`got` fields; nothing here builds or runs a program
// from a string — every program is a `.wat.bad` fixture.
#[test]
fn a_declared_letter_is_not_shared() {
    refused(
        "letter_as_shared",
        "(:probe::R :- [:T])",
        "(:probe::R :- [:wat::kernel::Transport.Shared])",
    );
}

#[test]
fn wire_is_not_a_declared_letter() {
    refused(
        "shared_as_letter",
        "(:probe::R :- [:wat::kernel::Transport.Wire])",
        "(:probe::R :- [:T])",
    );
}

#[test]
fn a_type_one_argument_short_is_not_the_full_one() {
    refused(
        "record_short",
        "(:probe::R :- [:wat::core::i64])",
        "(:probe::R :- [:wat::core::i64 :wat::kernel::Transport.Wire])",
    );
}

#[test]
fn a_type_one_transport_argument_long_is_not_the_short_one() {
    refused(
        "record_long",
        "(:probe::R :- [:wat::core::i64 :wat::kernel::Transport.Shared])",
        "(:probe::R :- [:wat::core::i64])",
    );
}
