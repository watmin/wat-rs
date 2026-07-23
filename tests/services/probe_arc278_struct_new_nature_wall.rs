//! Arc 278 item-c Strike B — the `struct-new` NATURE WALL.
//!
//! WHY: `:wat::core::struct-new` mints a `Nature::Struct` aggregate. Before this wall,
//! `struct-new :wat::kernel::Failure` compiled clean even though `Failure` is `Nature::Record`
//! (arc 293.W.2b — a crash cause crosses the wire, pure EDN) — producing a wrong-nature value
//! that the Record accessor `Failure/message` can't read back. Strike A gave the corpus the one
//! canonical message-only constructor (`:wat::kernel::message-only-failure`); this wall makes the
//! WRONG construction path a located compile error instead of a silent pass, transitively
//! disallowing `struct-new` on ANY record-natured (or enum) type, not just Failure.
//!
//! This is the disconfirming proof: before the wall this form compiled (and built a wrong-nature
//! value); after, it's a located `MalformedForm` check error naming the offending type.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn struct_new_on_record_natured_failure_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc278_struct_new_nature_wall.wat.bad")
        .expect_err("struct-new on a record-natured type (:wat::kernel::Failure) must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::struct-new"
            && reason.contains("wat::kernel::Failure")
            && reason.contains("record-natured"));
}
