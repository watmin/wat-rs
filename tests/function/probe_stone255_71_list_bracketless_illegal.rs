//! Arc 255 STONE 71 (THE WALL) — `List`'s `:- [T…]` bracket, optional since 255.70, is
//! MANDATORY now (the same as its six siblings). Disconfirming proof: before this stone,
//! `(wat.type/List 1 2 3)` checked AND ran (255.70's own `list-bracketless-still-runs` STOP-2
//! guard); after, it is a located `MalformedForm` check error naming the offending head and the
//! fix.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn bracketless_list_call_is_a_compile_error() {
    let err = startup_from_file("tests/function/probe_stone255_71_list_bracketless_illegal.wat.bad")
        .expect_err("a bracket-less `List` constructor call must fail check under THE WALL");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::List"
            && reason.contains("List")
            && reason.contains("wat.type/List :- [T"));
}
