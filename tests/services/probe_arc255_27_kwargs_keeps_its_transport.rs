//! Arc 255 Stone 255.27 (C-b5) — **the kwargs check keeps a handle's transport.**
//!
//! 255.21's witness (`wat-scripts/scratch-pad/255-21-kwargs-transport-lost-at-impl.wat`, moved
//! here as its header instructed): a thread handle passed through the generated
//! `::kwargs-check` lost its transport at `$impl`, because `assignable`'s
//! `transport_param_instantiates` arm admitted `Transport.Shared` against the fresh `T0` without
//! binding it. With the arm deleted the variable binds, so the thread handle's coord is Shared,
//! and a Wire claim is refused.
//!
//! The refusal was ACCEPTED (rc=0) on the pre-stone binary (`576200c59`).
//!
//! Run: cargo test --release -p wat --test services -- arc255_27

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const ACCEPTED: &str = "tests/services/probe_arc255_27_kwargs_keeps_its_transport.wat";
const REFUSED: &str = "tests/services/probe_arc255_27_kwargs_keeps_its_transport.wat.bad";

fn echo_addr(transport: &str) -> String {
    // rune:lint(no-inlined-wat) — golden COMPARISON text for a ReturnTypeMismatch's rendered
    // `expected`/`got` fields, never a wat world/driver; nothing builds or runs a program from it.
    format!("(:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply {transport}])")
}

#[test]
fn arc255_27_kwargs_thread_handle_is_shared() {
    startup_from_file(ACCEPTED)
        .expect("a thread handle through the kwargs check keeps its Shared transport");
}

#[test]
fn arc255_27_kwargs_thread_handle_claimed_wire_is_refused() {
    let err = startup_from_file(REFUSED)
        .expect_err("a thread handle's coord claimed as Wire must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    let (want_expected, want_got) = (
        echo_addr(":wat::kernel::Transport.Wire"),
        echo_addr(":wat::kernel::Transport.Shared"),
    );
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::kwargs-thread-handle-claimed-wire"
            && *expected == want_expected && *got == want_got);
}
