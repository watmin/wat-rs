//! FM 2-bis probe for Stone 241.13 — `:wat::core::define-dispatch` HARD CUT.
//!
//! Stone 241.13 retires `:wat::core::define-dispatch` (arc 146's dispatch-by-arity+type
//! entity kind). `:wat::core::defclause` (Stone 237.2 SHIPPED `bdd9eb6c`) is the
//! surviving dispatch entity kind. ALL substrate scaffolding for define-dispatch is
//! DELETED: src/dispatch.rs (445 lines), DispatchRegistry plumbing across check.rs +
//! freeze.rs + runtime.rs + resolve.rs, special_forms.rs entry, freeze.rs walker arms.
//!
//! THE DOCTRINE: HARD CUT is total. No "infrastructure stays empty so it's fine" framing.
//!
//! HEAD-disconfirmation map (both contracts FAIL at HEAD):
//! - C01: `:wat::core::define-dispatch` HARD-CUT-rejected at startup ⇒ FAILS at HEAD
//! - C02: rejection error carries structured retirement remedy naming defclause ⇒ FAILS at HEAD
//!
//! Post-stone: both contracts PASS.

use wat::freeze::startup_from_file;

// ─── C01: :wat::core::define-dispatch HARD-CUT-rejected at startup ─────────────

#[test]
fn contract_01_define_dispatch_hard_cut_rejected() {
    // A well-formed define-dispatch decl must be HARD-CUT-rejected at startup.
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone13_define_dispatch_hard_cut_bad.wat",
    );
    assert!(
        result.is_err(),
        "`:wat::core::define-dispatch` must be HARD-CUT-rejected post-stone; got Ok"
    );
}

// ─── C02: rejection carries structured retirement remedy naming defclause ──────

#[test]
fn contract_02_rejection_remedy_names_defclause() {
    let result = startup_from_file(
        "tests/wat_lang/probe_arc241_stone13_define_dispatch_hard_cut_bad.wat",
    );
    let msg = format!("{}", result.unwrap_err());
    assert_eq!(
        msg,
        "check:\n1 type-check error(s):\n  - tests/wat_lang/probe_arc241_stone13_define_dispatch_hard_cut_bad.wat:6:2: malformed :wat::core::define-dispatch form: ':wat::core::define-dispatch' is retired (Stone 241.13); use ':wat::core::defclause' instead\n  did you mean: :wat::core::defclause [replaces a retired form]\n",
        "retirement remedy must name :wat::core::defclause with [replaces a retired form]"
    );
}
