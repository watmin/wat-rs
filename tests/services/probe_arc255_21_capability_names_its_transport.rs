//! Arc 255 Stone 255.21 (C-b1b) — **the capability names its transport.**
//!
//! `Dialable` and `TypedCapability` are `:- [S R T]`, `coord` returns `(Address :- [S R T])`, and
//! defservice's edges bind the Handle's own transport letter (`(Handle :- [… T])` extends
//! `(Dialable :- [Op Reply T])`); 255.23 binds that `T` from the receiver. So a thread handle's
//! `coord` is a Shared address and a process handle's a Wire one — never either.
//!
//! Every refusal below was ACCEPTED (rc=0) on the pre-stone binary (`997d726ec`), when `coord`
//! returned the 2-argument `(Address :- [S R])` and the missing-slot arm admitted either transport.
//!
//! Run: cargo test --release -p wat --test services -- arc255_21

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const ACCEPTED: &str = "tests/services/probe_arc255_21_coord_names_its_transport.wat";
const REFUSED: &str = "tests/services/probe_arc255_21_coord_claims_either_transport.wat.bad";

const SHARED: &str = ":wat::kernel::Transport.Shared";
const WIRE: &str = ":wat::kernel::Transport.Wire";

/// (function, the transport it claimed, the transport its handle actually has)
const REFUSALS: &[(&str, &str, &str)] = &[
    (":probe::thread-coord-claimed-wire", WIRE, SHARED),
    (":probe::process-coord-claimed-shared", SHARED, WIRE),
    (":probe::thread-typedcap-coord-claimed-wire", WIRE, SHARED),
    (":probe::process-typedcap-coord-claimed-shared", SHARED, WIRE),
];

fn echo_addr(transport: &str) -> String {
    // rune:lint(no-inlined-wat) — golden COMPARISON text for a ReturnTypeMismatch's rendered
    // `expected`/`got` fields, never a wat world/driver; nothing builds or runs a program from it.
    format!("(:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply {transport}])")
}

#[test]
fn arc255_21_twins_are_accepted() {
    startup_from_file(ACCEPTED).expect(
        "a thread handle's coord is Shared and a process handle's is Wire, through both \
         Dialable and TypedCapability",
    );
}

#[test]
fn arc255_21_a_claimed_transport_is_refused() {
    let err = startup_from_file(REFUSED)
        .expect_err("a coord claimed at the other transport must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    for &(func, claimed, actual) in REFUSALS {
        let (want_expected, want_got) = (echo_addr(claimed), echo_addr(actual));
        wat::assert_check_error_present!(errs,
            CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
                if function == func && *expected == want_expected && *got == want_got);
    }
}
