//! 255.5 — `also_accept_type` on annotation/return slots.
//! A known type in a type slot must rewrite; the same name as a call head must not.

use wat::freeze::{startup_beside, startup_from_file};

#[test]
fn known_types_in_annotation_and_return_slots_check() {
    assert!(
        startup_beside(file!()).is_ok(),
        "wat/WatAST, wat.core/i64, wat.time/Instant in :- / -> slots must check"
    );
}

#[test]
fn type_in_call_position_still_unresolved() {
    let err = startup_from_file("tests/resolve/probe_arc255_5_position_signal.wat.bad")
        .expect_err("a type used as a call head must fail");
    let msg = format!("{err}");
    // rune:lint(loose-assert) — the diagnostic carries a file:line span that
    // varies with the fixture path; pin the two stable tokens of the refusal.
    assert!(
        msg.contains("UnresolvedReference") && msg.contains("Instant"),
        "expected UnresolvedReference naming Instant for (wat.time/Instant) as a call, got:\n{msg}"
    );
}
