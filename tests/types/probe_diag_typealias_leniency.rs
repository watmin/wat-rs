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
#[ignore = "RE-POINTED arc 255 Stone P3 (2026-08-28): re-measured — still LENIENT (unchanged). But \
            the stated blocker was FALSE: this is not 255's undefined-CALL-head class (walk.rs:268, \
            resolve's blanket-accept). Traced to `parse_type_expr`/`parse_type_node` (src/types.rs \
            ~4907+): an ANNOTATION-position type keyword is only syntax-parsed into TypeExpr::Path, \
            never checked against TypeEnv for existence — no `UnknownType`/`UndeclaredType` error \
            kind exists in src/types/error.rs at all. Documented, with three required exemptions \
            (type params in scope, :wat::core::Value the deliberate top, forward references) and an \
            explicitly UNMEASURED blast radius, in \
            docs/arc/2026/04/109-kill-std/NOTE-type-annotation-names-unchecked.md — which extends \
            278's landed DESIGN-STONE-query-type-safe.md part 3 (call-position validation) to the \
            annotation position, and is owned by 109/278, NOT 255. Check by re-reading that NOTE's \
            'Blast radius — UNMEASURED' section for whether the corpus walk has since been run."]
fn probe_undeclared_field_type_keyword_rejected_or_lenient() {
    let result = startup_from_file(
        "tests/types/probe_diag_typealias_leniency_check.wat.bad",
    );
    match result {
        Ok(_) => panic!("LENIENT: undeclared field-type keyword accepted silently"),
        Err(e) => println!("STRICT: rejected with: {}", e),
    }
}
