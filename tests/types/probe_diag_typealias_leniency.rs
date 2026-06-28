//! Arc 255 banked disconfirming gate (surfaced at 214.8.2 scoring): the
//! checker ACCEPTS a defstruct field typed with an UNDECLARED typealias
//! keyword — the TYPE-keyword sibling of the fresh-var leniency that hid
//! `+'2` (the undefined-leaf dark class arc 255 kills). Empirical incident:
//! the 8.2 stdin rebirth DELETED the load-bearing `:wat::kernel::ThreadId`
//! typealias and every gate stayed green; only the orchestrator's read of
//! the deletion diff caught it.
//!
//! RED by design (the panic on Ok is the gate); #[ignore]'d so the suite
//! stays truthful about known work. Arc 255 un-ignores it: when undeclared
//! type keywords become check errors, the Err arm makes this GREEN.

use wat::freeze::startup_from_file;

#[test]
#[ignore = "arc 255 banked gate: undeclared field-type keywords are accepted LENIENTLY today (the +'2 dark class, type-keyword flavor); un-ignore when 255 makes them check errors"]
fn probe_undeclared_field_type_keyword_rejected_or_lenient() {
    let result = startup_from_file(
        "tests/types/probe_diag_typealias_leniency_check_bad.wat",
    );
    match result {
        Ok(_) => panic!("LENIENT: undeclared field-type keyword accepted silently"),
        Err(e) => println!("STRICT: rejected with: {}", e),
    }
}
